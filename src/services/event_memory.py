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
