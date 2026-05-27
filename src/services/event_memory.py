import json
import math
import sqlite3
from datetime import datetime
from pathlib import Path
from typing import Literal

from src.config import settings
from src.models.event import Event


def _db_path(user_id: str) -> Path:
    return settings.data_dir / user_id / "events.db"


def _get_conn(user_id: str) -> sqlite3.Connection:
    db_path = _db_path(user_id)
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row
    conn.execute(
        """CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            emotions TEXT NOT NULL DEFAULT '[]',
            importance REAL NOT NULL,
            event_type TEXT,
            strength REAL NOT NULL DEFAULT 1.0,
            stability REAL NOT NULL DEFAULT 30.0,
            recall_count INTEGER NOT NULL DEFAULT 0,
            last_recalled_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )"""
    )
    conn.execute(
        """CREATE TABLE IF NOT EXISTS conversation_turns (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_msg TEXT NOT NULL,
            ai_msg TEXT NOT NULL,
            summary TEXT,
            emotions TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL
        )"""
    )
    # FTS5 全文搜索索引（跨会话检索）
    conn.execute(
        """CREATE VIRTUAL TABLE IF NOT EXISTS turn_search USING fts5(
            summary,
            user_msg,
            content='conversation_turns',
            content_rowid='id'
        )"""
    )
    # 洞察积累表（结构化的用户模式认知）
    conn.execute(
        """CREATE TABLE IF NOT EXISTS insights (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category TEXT NOT NULL,
            content TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0.5,
            source TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )"""
    )
    # 定性观察表（v0.3 替代八维权重）
    conn.execute(
        """CREATE TABLE IF NOT EXISTS observations (
            id TEXT PRIMARY KEY,
            date TEXT NOT NULL,
            content TEXT NOT NULL,
            category TEXT,
            source TEXT,
            created_at TEXT NOT NULL
        )"""
    )
    # 主题表
    conn.execute(
        """CREATE TABLE IF NOT EXISTS topics (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            first_mentioned TEXT,
            last_mentioned TEXT,
            mention_count INTEGER DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )"""
    )
    # 主题关联表
    conn.execute(
        """CREATE TABLE IF NOT EXISTS topic_links (
            topic_id TEXT NOT NULL,
            item_id TEXT NOT NULL,
            item_type TEXT NOT NULL,
            PRIMARY KEY (topic_id, item_id, item_type)
        )"""
    )
    # 项目档案表
    conn.execute(
        """CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT DEFAULT 'active',
            start_date TEXT,
            end_date TEXT,
            event_ids TEXT DEFAULT '[]',
            summary TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )"""
    )
    # 成长线表
    conn.execute(
        """CREATE TABLE IF NOT EXISTS growth_lines (
            id TEXT PRIMARY KEY,
            dimension TEXT NOT NULL,
            records TEXT DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )"""
    )
    # 向后兼容：为旧数据库添加新列
    for col, col_def in [
        ("strength", "REAL NOT NULL DEFAULT 1.0"),
        ("stability", "REAL NOT NULL DEFAULT 30.0"),
        ("recall_count", "INTEGER NOT NULL DEFAULT 0"),
        ("last_recalled_at", "TEXT"),
    ]:
        try:
            conn.execute(f"ALTER TABLE events ADD COLUMN {col} {col_def}")
        except sqlite3.OperationalError:
            pass  # 列已存在

    conn.commit()
    return conn


def init_db(user_id: str) -> None:
    """初始化用户的事件数据库"""
    _get_conn(user_id).close()


def add_event(user_id: str, event: Event) -> None:
    """添加事件，根据 importance 设置初始稳定性"""
    event.stability = settings.forget_base_stability * (0.5 + event.importance)
    conn = _get_conn(user_id)
    conn.execute(
        "INSERT INTO events (id, content, emotions, importance, event_type, "
        "strength, stability, recall_count, created_at, updated_at) "
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (
            event.id,
            event.content,
            json.dumps(event.emotions),
            event.importance,
            event.event_type,
            event.strength,
            event.stability,
            event.recall_count,
            event.created_at.isoformat(),
            event.updated_at.isoformat(),
        ),
    )
    conn.commit()
    conn.close()


def query_events(
    user_id: str,
    limit: int = 20,
    min_importance: float = 0.0,
    event_type: str | None = None,
    start_date: str | None = None,
    end_date: str | None = None,
    min_strength: float = 0.0,
) -> list[Event]:
    """查询事件，自动计算遗忘后的实际强度"""
    conn = _get_conn(user_id)
    conditions = ["importance >= ?"]
    params: list = [min_importance]

    if event_type:
        conditions.append("event_type = ?")
        params.append(event_type)
    if start_date:
        conditions.append("created_at >= ?")
        params.append(start_date)
    if end_date:
        conditions.append("created_at <= ?")
        params.append(end_date)

    where = " AND ".join(conditions)
    params.append(limit)

    rows = conn.execute(
        f"SELECT * FROM events WHERE {where} ORDER BY created_at DESC LIMIT ?",
        params,
    ).fetchall()
    conn.close()

    events = []
    for r in rows:
        strength = _compute_strength(
            created_at=r["created_at"],
            stability=r["stability"] if r["stability"] else 30.0,
        )
        if strength < min_strength:
            continue
        events.append(Event(
            id=r["id"],
            content=r["content"],
            emotions=json.loads(r["emotions"]),
            importance=r["importance"],
            event_type=r["event_type"],
            strength=strength,
            stability=r["stability"] if r["stability"] else 30.0,
            recall_count=r["recall_count"] if r["recall_count"] else 0,
            last_recalled_at=datetime.fromisoformat(r["last_recalled_at"]) if r["last_recalled_at"] else None,
            created_at=datetime.fromisoformat(r["created_at"]),
            updated_at=datetime.fromisoformat(r["updated_at"]),
        ))
    return events


def delete_event(user_id: str, event_id: str) -> bool:
    conn = _get_conn(user_id)
    cursor = conn.execute("DELETE FROM events WHERE id = ?", (event_id,))
    conn.commit()
    deleted = cursor.rowcount > 0
    conn.close()
    return deleted


# ============ 遗忘曲线 ============

def _compute_strength(created_at: str, stability: float) -> float:
    """Ebbinghaus 遗忘曲线: strength = e^(-t/S)
    t = 距创建的天数, S = 稳定天数
    """
    try:
        created = datetime.fromisoformat(created_at)
        days_elapsed = (datetime.now() - created).total_seconds() / 86400
        return math.exp(-days_elapsed / stability)
    except (ValueError, TypeError):
        return 1.0


def record_recall(user_id: str, event_id: str) -> None:
    """复述效应：被检索到的事件稳定性增加 50%"""
    conn = _get_conn(user_id)
    row = conn.execute(
        "SELECT stability, recall_count FROM events WHERE id = ?",
        (event_id,),
    ).fetchone()
    if row:
        new_stability = row["stability"] * (1 + settings.forget_recall_boost)
        new_count = (row["recall_count"] or 0) + 1
        conn.execute(
            "UPDATE events SET stability=?, recall_count=?, last_recalled_at=?, updated_at=? WHERE id=?",
            (new_stability, new_count, datetime.now().isoformat(),
             datetime.now().isoformat(), event_id),
        )
        conn.commit()
    conn.close()


def decay_all_events(user_id: str) -> int:
    """重新计算所有事件的强度。返回低于阈值的事件数"""
    conn = _get_conn(user_id)
    rows = conn.execute("SELECT id, created_at, stability FROM events").fetchall()
    count = 0
    for r in rows:
        strength = _compute_strength(r["created_at"], r["stability"])
        conn.execute(
            "UPDATE events SET strength=?, updated_at=? WHERE id=?",
            (strength, datetime.now().isoformat(), r["id"]),
        )
        if strength < settings.forget_min_strength:
            count += 1
    conn.commit()
    conn.close()
    return count


def cleanup_forgotten_events(user_id: str, min_strength: float | None = None) -> int:
    """删除强度低于阈值的事件，返回删除数量"""
    threshold = min_strength or settings.forget_min_strength
    conn = _get_conn(user_id)
    rows = conn.execute("SELECT id, created_at, stability FROM events").fetchall()
    to_delete = []
    for r in rows:
        strength = _compute_strength(r["created_at"], r["stability"])
        if strength < threshold:
            to_delete.append(r["id"])
    for eid in to_delete:
        conn.execute("DELETE FROM events WHERE id = ?", (eid,))
    conn.commit()
    conn.close()
    return len(to_delete)


# ============ 人格快照 ============



# ============ 跨会话搜索（FTS5） ============

def save_conversation_turn(
    user_id: str, user_msg: str, ai_msg: str,
    summary: str | None = None, emotions: list[str] | None = None,
) -> None:
    """存储每轮对话摘要，供跨会话搜索"""
    conn = _get_conn(user_id)
    now = datetime.now().isoformat()
    conn.execute(
        "INSERT INTO conversation_turns (user_msg, ai_msg, summary, emotions, created_at) "
        "VALUES (?, ?, ?, ?, ?)",
        (user_msg, ai_msg, summary or "", json.dumps(emotions or []), now),
    )
    # 同步写入 FTS5 索引
    turn_id = conn.execute("SELECT last_insert_rowid()").fetchone()[0]
    fts_summary = summary or user_msg[:200]
    conn.execute(
        "INSERT INTO turn_search (rowid, summary, user_msg) VALUES (?, ?, ?)",
        (turn_id, fts_summary, user_msg[:500]),
    )
    conn.commit()
    conn.close()


def search_conversations(user_id: str, query: str, limit: int = 5) -> list[dict]:
    """FTS5 全文搜索历史对话"""
    conn = _get_conn(user_id)
    rows = conn.execute(
        """SELECT ct.id, ct.summary, ct.emotions, ct.created_at
        FROM conversation_turns ct
        JOIN turn_search ts ON ct.id = ts.rowid
        WHERE turn_search MATCH ?
        ORDER BY rank
        LIMIT ?""",
        (query, limit),
    ).fetchall()
    conn.close()
    return [
        {
            "id": r["id"],
            "summary": r["summary"],
            "emotions": json.loads(r["emotions"]),
            "created_at": r["created_at"],
        }
        for r in rows
    ]


# ============ 洞察积累 ============

def save_insight(
    user_id: str, category: str, content: str,
    confidence: float = 0.5, source: str = "",
) -> None:
    """
    存储结构化洞察。
    category: emotion_pattern | relationship | behavior | value | growth
    """
    conn = _get_conn(user_id)
    now = datetime.now().isoformat()
    conn.execute(
        "INSERT INTO insights (category, content, confidence, source, created_at, updated_at) "
        "VALUES (?, ?, ?, ?, ?, ?)",
        (category, content, confidence, source, now, now),
    )
    conn.commit()
    conn.close()


def get_insights(
    user_id: str, category: str | None = None, limit: int = 20
) -> list[dict]:
    """查询洞察，按置信度降序"""
    conn = _get_conn(user_id)
    if category:
        rows = conn.execute(
            "SELECT * FROM insights WHERE category = ? ORDER BY confidence DESC, created_at DESC LIMIT ?",
            (category, limit),
        ).fetchall()
    else:
        rows = conn.execute(
            "SELECT * FROM insights ORDER BY confidence DESC, created_at DESC LIMIT ?",
            (limit,),
        ).fetchall()
    conn.close()
    return [
        {
            "id": r["id"],
            "category": r["category"],
            "content": r["content"],
            "confidence": r["confidence"],
            "source": r["source"],
            "created_at": r["created_at"],
        }
        for r in rows
    ]


def update_insight_confidence(user_id: str, insight_id: int, new_confidence: float) -> None:
    """更新洞察置信度（被后续观察验证时提升）"""
    conn = _get_conn(user_id)
    conn.execute(
        "UPDATE insights SET confidence = ?, updated_at = ? WHERE id = ?",
        (new_confidence, datetime.now().isoformat(), insight_id),
    )
    conn.commit()
    conn.close()


# ============ 定性观察（v0.3 替代八维权重） ============

def add_observation(
    user_id: str, obs_id: str, date: str, content: str,
    category: str | None = None, source: str = "reflection",
) -> None:
    """添加一条定性观察"""
    conn = _get_conn(user_id)
    conn.execute(
        "INSERT INTO observations (id, date, content, category, source, created_at) "
        "VALUES (?, ?, ?, ?, ?, ?)",
        (obs_id, date, content, category, source, datetime.now().isoformat()),
    )
    conn.commit()
    conn.close()


def query_observations(
    user_id: str,
    category: str | None = None,
    limit: int = 20,
) -> list[dict]:
    """查询定性观察，按日期降序"""
    conn = _get_conn(user_id)
    if category:
        rows = conn.execute(
            "SELECT * FROM observations WHERE category = ? ORDER BY date DESC LIMIT ?",
            (category, limit),
        ).fetchall()
    else:
        rows = conn.execute(
            "SELECT * FROM observations ORDER BY date DESC LIMIT ?",
            (limit,),
        ).fetchall()
    conn.close()
    return [
        {
            "id": r["id"],
            "date": r["date"],
            "content": r["content"],
            "category": r["category"],
            "source": r["source"],
            "created_at": r["created_at"],
        }
        for r in rows
    ]


# ============ 主题系统 ============

def add_topic(
    user_id: str, topic_id: str, name: str,
    description: str | None = None, date_str: str | None = None,
) -> None:
    """创建新主题"""
    now = datetime.now().isoformat()
    conn = _get_conn(user_id)
    conn.execute(
        "INSERT INTO topics (id, name, description, first_mentioned, last_mentioned, "
        "mention_count, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        (topic_id, name, description or "", date_str or now, date_str or now, 1, now, now),
    )
    conn.commit()
    conn.close()


def query_topics(user_id: str, limit: int = 50) -> list[dict]:
    """查询所有主题，按最近提及排序"""
    conn = _get_conn(user_id)
    rows = conn.execute(
        "SELECT * FROM topics ORDER BY last_mentioned DESC LIMIT ?",
        (limit,),
    ).fetchall()
    conn.close()
    return [
        {
            "id": r["id"],
            "name": r["name"],
            "description": r["description"],
            "first_mentioned": r["first_mentioned"],
            "last_mentioned": r["last_mentioned"],
            "mention_count": r["mention_count"],
        }
        for r in rows
    ]


def get_topic(user_id: str, topic_id: str) -> dict | None:
    """获取单个主题"""
    conn = _get_conn(user_id)
    row = conn.execute("SELECT * FROM topics WHERE id = ?", (topic_id,)).fetchone()
    conn.close()
    if not row:
        return None
    return {
        "id": row["id"],
        "name": row["name"],
        "description": row["description"],
        "first_mentioned": row["first_mentioned"],
        "last_mentioned": row["last_mentioned"],
        "mention_count": row["mention_count"],
    }


def get_topic_by_name(user_id: str, name: str) -> dict | None:
    """按名称精确查找主题"""
    conn = _get_conn(user_id)
    row = conn.execute("SELECT * FROM topics WHERE name = ?", (name,)).fetchone()
    conn.close()
    if not row:
        return None
    return {
        "id": row["id"],
        "name": row["name"],
        "description": row["description"],
        "first_mentioned": row["first_mentioned"],
        "last_mentioned": row["last_mentioned"],
        "mention_count": row["mention_count"],
    }


def update_topic(user_id: str, topic_id: str, **kwargs) -> None:
    """更新主题字段"""
    conn = _get_conn(user_id)
    sets = []
    vals = []
    for k, v in kwargs.items():
        sets.append(f"{k} = ?")
        vals.append(v)
    vals.append(datetime.now().isoformat())
    vals.append(topic_id)
    conn.execute(
        f"UPDATE topics SET {', '.join(sets)}, updated_at = ? WHERE id = ?",
        vals,
    )
    conn.commit()
    conn.close()


def link_topic(user_id: str, topic_id: str, item_id: str, item_type: str) -> None:
    """关联主题和事件/观察"""
    conn = _get_conn(user_id)
    try:
        conn.execute(
            "INSERT INTO topic_links (topic_id, item_id, item_type) VALUES (?, ?, ?)",
            (topic_id, item_id, item_type),
        )
        conn.commit()
    except sqlite3.IntegrityError:
        pass  # 已存在
    conn.close()


def get_topic_links(user_id: str, topic_id: str) -> list[dict]:
    """获取主题下的所有关联项"""
    conn = _get_conn(user_id)
    rows = conn.execute(
        "SELECT * FROM topic_links WHERE topic_id = ?",
        (topic_id,),
    ).fetchall()
    conn.close()
    return [
        {"topic_id": r["topic_id"], "item_id": r["item_id"], "item_type": r["item_type"]}
        for r in rows
    ]


def get_item_topics(user_id: str, item_id: str, item_type: str) -> list[dict]:
    """获取事件/观察关联的所有主题"""
    conn = _get_conn(user_id)
    rows = conn.execute(
        "SELECT t.* FROM topics t JOIN topic_links tl ON t.id = tl.topic_id "
        "WHERE tl.item_id = ? AND tl.item_type = ?",
        (item_id, item_type),
    ).fetchall()
    conn.close()
    return [
        {"id": r["id"], "name": r["name"], "description": r["description"]}
        for r in rows
    ]


# ============ 项目档案 ============

def add_project(
    user_id: str, project_id: str, title: str,
    description: str | None = None, event_ids: list[str] | None = None,
    start_date: str | None = None, end_date: str | None = None,
) -> None:
    """创建项目"""
    now = datetime.now().isoformat()
    conn = _get_conn(user_id)
    conn.execute(
        "INSERT INTO projects (id, title, description, status, start_date, end_date, "
        "event_ids, summary, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (project_id, title, description or "", "active",
         start_date or now, end_date or now,
         json.dumps(event_ids or []), "", now, now),
    )
    conn.commit()
    conn.close()


def query_projects(user_id: str, status: str | None = None, limit: int = 50) -> list[dict]:
    """查询项目列表"""
    conn = _get_conn(user_id)
    if status:
        rows = conn.execute(
            "SELECT * FROM projects WHERE status = ? ORDER BY updated_at DESC LIMIT ?",
            (status, limit),
        ).fetchall()
    else:
        rows = conn.execute(
            "SELECT * FROM projects ORDER BY updated_at DESC LIMIT ?",
            (limit,),
        ).fetchall()
    conn.close()
    return [
        {
            "id": r["id"],
            "title": r["title"],
            "description": r["description"],
            "status": r["status"],
            "start_date": r["start_date"],
            "end_date": r["end_date"],
            "event_ids": json.loads(r["event_ids"]),
            "summary": r["summary"],
        }
        for r in rows
    ]


def get_project(user_id: str, project_id: str) -> dict | None:
    """获取单个项目"""
    conn = _get_conn(user_id)
    row = conn.execute("SELECT * FROM projects WHERE id = ?", (project_id,)).fetchone()
    conn.close()
    if not row:
        return None
    return {
        "id": row["id"],
        "title": row["title"],
        "description": row["description"],
        "status": row["status"],
        "start_date": row["start_date"],
        "end_date": row["end_date"],
        "event_ids": json.loads(row["event_ids"]),
        "summary": row["summary"],
        "created_at": row["created_at"],
        "updated_at": row["updated_at"],
    }


def update_project(user_id: str, project_id: str, **kwargs) -> None:
    """更新项目字段"""
    conn = _get_conn(user_id)
    sets = []
    vals = []
    for k, v in kwargs.items():
        if k == "event_ids" and isinstance(v, list):
            v = json.dumps(v)
        sets.append(f"{k} = ?")
        vals.append(v)
    vals.append(datetime.now().isoformat())
    vals.append(project_id)
    conn.execute(
        f"UPDATE projects SET {', '.join(sets)}, updated_at = ? WHERE id = ?",
        vals,
    )
    conn.commit()
    conn.close()


# ============ 成长线 ============

def add_growth_line(user_id: str, gl_id: str, dimension: str) -> None:
    """创建成长线"""
    now = datetime.now().isoformat()
    conn = _get_conn(user_id)
    conn.execute(
        "INSERT INTO growth_lines (id, dimension, records, created_at, updated_at) "
        "VALUES (?, ?, ?, ?, ?)",
        (gl_id, dimension, "[]", now, now),
    )
    conn.commit()
    conn.close()


def query_growth_lines(user_id: str, limit: int = 50) -> list[dict]:
    """查询所有成长线"""
    conn = _get_conn(user_id)
    rows = conn.execute(
        "SELECT * FROM growth_lines ORDER BY updated_at DESC LIMIT ?",
        (limit,),
    ).fetchall()
    conn.close()
    return [
        {
            "id": r["id"],
            "dimension": r["dimension"],
            "records": json.loads(r["records"]),
        }
        for r in rows
    ]


def get_growth_line(user_id: str, dimension: str) -> dict | None:
    """按维度查找成长线"""
    conn = _get_conn(user_id)
    row = conn.execute(
        "SELECT * FROM growth_lines WHERE dimension = ?",
        (dimension,),
    ).fetchone()
    conn.close()
    if not row:
        return None
    return {
        "id": row["id"],
        "dimension": row["dimension"],
        "records": json.loads(row["records"]),
    }


def update_growth_line_records(user_id: str, gl_id: str, records: list[dict]) -> None:
    """更新成长线记录"""
    conn = _get_conn(user_id)
    conn.execute(
        "UPDATE growth_lines SET records = ?, updated_at = ? WHERE id = ?",
        (json.dumps(records, ensure_ascii=False), datetime.now().isoformat(), gl_id),
    )
    conn.commit()
    conn.close()
