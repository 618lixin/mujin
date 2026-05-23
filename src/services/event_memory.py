import json
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
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )"""
    )
    conn.execute(
        """CREATE TABLE IF NOT EXISTS personality_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            weights TEXT NOT NULL,
            summary TEXT,
            created_at TEXT NOT NULL
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
    conn.commit()
    return conn


def init_db(user_id: str) -> None:
    """初始化用户的事件数据库"""
    _get_conn(user_id).close()


def add_event(user_id: str, event: Event) -> None:
    conn = _get_conn(user_id)
    conn.execute(
        "INSERT INTO events (id, content, emotions, importance, event_type, created_at, updated_at) "
        "VALUES (?, ?, ?, ?, ?, ?, ?)",
        (
            event.id,
            event.content,
            json.dumps(event.emotions),
            event.importance,
            event.event_type,
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
) -> list[Event]:
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

    return [
        Event(
            id=r["id"],
            content=r["content"],
            emotions=json.loads(r["emotions"]),
            importance=r["importance"],
            event_type=r["event_type"],
            created_at=datetime.fromisoformat(r["created_at"]),
            updated_at=datetime.fromisoformat(r["updated_at"]),
        )
        for r in rows
    ]


def delete_event(user_id: str, event_id: str) -> bool:
    conn = _get_conn(user_id)
    cursor = conn.execute("DELETE FROM events WHERE id = ?", (event_id,))
    conn.commit()
    deleted = cursor.rowcount > 0
    conn.close()
    return deleted


def save_personality_snapshot(user_id: str, weights: dict, summary: str = "") -> None:
    """保存人格权重快照"""
    conn = _get_conn(user_id)
    conn.execute(
        "INSERT INTO personality_snapshots (weights, summary, created_at) VALUES (?, ?, ?)",
        (json.dumps(weights), summary, datetime.now().isoformat()),
    )
    conn.commit()
    conn.close()


def get_personality_snapshots(user_id: str, limit: int = 50) -> list[dict]:
    """查询人格权重历史"""
    conn = _get_conn(user_id)
    rows = conn.execute(
        "SELECT * FROM personality_snapshots ORDER BY created_at DESC LIMIT ?",
        (limit,),
    ).fetchall()
    conn.close()
    return [
        {
            "id": r["id"],
            "weights": json.loads(r["weights"]),
            "summary": r["summary"],
            "created_at": r["created_at"],
        }
        for r in rows
    ]


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
