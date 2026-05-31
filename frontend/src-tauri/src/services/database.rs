use std::path::PathBuf;

use rusqlite::{params, Connection};
use serde_json;

use super::notes::AppError;
use super::types::{
    ConversationTurn, Event, GrowthLine, Insight, Observation, Project, QueryEventsParams, Topic,
    TopicLink,
};

// 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Database State 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌?

pub struct DbState {
    base_dir: PathBuf,
}

fn validate_user_id(user_id: &str) -> Result<(), AppError> {
    let valid = !user_id.is_empty()
        && user_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'));

    if valid {
        Ok(())
    } else {
        Err(AppError::new(
            "invalidUserId",
            "user_id may only contain ASCII letters, numbers, underscores, and hyphens",
        ))
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

impl DbState {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Open a connection to the user's database.
    fn conn(&self, user_id: &str) -> Result<Connection, AppError> {
        validate_user_id(user_id)?;
        let db_dir = self.base_dir.join(user_id);
        std::fs::create_dir_all(&db_dir)
            .map_err(|e| AppError::new("io", format!("Failed to create user dir: {e}")))?;
        let db_path = db_dir.join("events.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::new("db", format!("Failed to open database: {e}")))?;
        conn.execute_batch("PRAGMA journal_mode=WAL;").ok();
        Ok(conn)
    }

    fn conn_with_schema(&self, user_id: &str) -> Result<Connection, AppError> {
        let conn = self.conn(user_id)?;
        Self::ensure_schema(&conn)?;
        Ok(conn)
    }

    // 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Schema Initialization 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟?

    pub fn init_db(&self, user_id: &str) -> Result<(), AppError> {
        let conn = self.conn(user_id)?;
        Self::ensure_schema(&conn)
    }

    fn ensure_schema(conn: &Connection) -> Result<(), AppError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
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
            );

            CREATE TABLE IF NOT EXISTS conversation_turns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_msg TEXT NOT NULL,
                ai_msg TEXT NOT NULL,
                summary TEXT,
                emotions TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS turn_search USING fts5(
                summary,
                user_msg
            );

            CREATE TABLE IF NOT EXISTS insights (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                content TEXT NOT NULL,
                confidence REAL NOT NULL DEFAULT 0.5,
                source TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS observations (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                content TEXT NOT NULL,
                category TEXT,
                source TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS topics (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                first_mentioned TEXT,
                last_mentioned TEXT,
                mention_count INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS topic_links (
                topic_id TEXT NOT NULL,
                item_id TEXT NOT NULL,
                item_type TEXT NOT NULL,
                PRIMARY KEY (topic_id, item_id, item_type)
            );

            CREATE TABLE IF NOT EXISTS projects (
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
            );

            CREATE TABLE IF NOT EXISTS growth_lines (
                id TEXT PRIMARY KEY,
                dimension TEXT NOT NULL,
                records TEXT DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )
        .map_err(|e| AppError::new("db", format!("Failed to create tables: {e}")))?;

        // Backward-compatible migrations: add new columns if missing
        let migrations: &[(&str, &str)] = &[
            ("strength", "REAL NOT NULL DEFAULT 1.0"),
            ("stability", "REAL NOT NULL DEFAULT 30.0"),
            ("recall_count", "INTEGER NOT NULL DEFAULT 0"),
            ("last_recalled_at", "TEXT"),
        ];
        for (col, col_def) in migrations {
            let sql = format!("ALTER TABLE events ADD COLUMN {col} {col_def}");
            conn.execute_batch(&sql).ok(); // ignore "column already exists"
        }

        Ok(())
    }

    // 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Events 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?

    pub fn add_event(
        &self,
        user_id: &str,
        event: &Event,
        base_stability: f64,
    ) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let stability = base_stability * (0.5 + event.importance);
        conn.execute(
            "INSERT INTO events (id, content, emotions, importance, event_type,
             strength, stability, recall_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                event.id,
                event.content,
                serde_json::to_string(&event.emotions).unwrap_or_else(|_| "[]".to_string()),
                event.importance,
                event.event_type,
                event.strength,
                stability,
                event.recall_count,
                event.created_at,
                event.updated_at,
            ],
        )
        .map_err(|e| AppError::new("db", format!("Failed to add event: {e}")))?;
        Ok(())
    }

    pub fn query_events(
        &self,
        user_id: &str,
        params_spec: &QueryEventsParams,
    ) -> Result<Vec<Event>, AppError> {
        let conn = self.conn_with_schema(user_id)?;

        let mut conditions = vec!["importance >= ?1".to_string()];
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(params_spec.min_importance)];
        let mut idx = 2u32;

        if let Some(ref et) = params_spec.event_type {
            conditions.push(format!("event_type = ?{idx}"));
            param_values.push(Box::new(et.clone()));
            idx += 1;
        }
        if let Some(ref sd) = params_spec.start_date {
            conditions.push(format!("created_at >= ?{idx}"));
            param_values.push(Box::new(sd.clone()));
            idx += 1;
        }
        if let Some(ref ed) = params_spec.end_date {
            conditions.push(format!("created_at <= ?{idx}"));
            param_values.push(Box::new(ed.clone()));
            idx += 1;
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT * FROM events WHERE {where_clause} ORDER BY created_at DESC LIMIT ?{idx}"
        );
        param_values.push(Box::new(params_spec.limit as i64));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| AppError::new("db", format!("Failed to prepare query: {e}")))?;
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                let created_at: String = row.get("created_at")?;
                let stability: f64 = row.get("stability").unwrap_or(30.0);
                let strength = compute_strength(&created_at, stability);
                Ok(Event {
                    id: row.get("id")?,
                    content: row.get("content")?,
                    emotions: parse_json_array(
                        &row.get::<_, String>("emotions").unwrap_or_default(),
                    ),
                    importance: row.get("importance")?,
                    event_type: row.get("event_type")?,
                    strength,
                    stability,
                    recall_count: row.get("recall_count").unwrap_or(0),
                    last_recalled_at: row.get("last_recalled_at")?,
                    created_at: created_at.clone(),
                    updated_at: row.get("updated_at")?,
                })
            })
            .map_err(|e| AppError::new("db", format!("Failed to query events: {e}")))?;

        let mut events = Vec::new();
        for row in rows {
            let event =
                row.map_err(|e| AppError::new("db", format!("Failed to read event row: {e}")))?;
            if event.strength < params_spec.min_strength {
                continue;
            }
            events.push(event);
        }
        Ok(events)
    }

    pub fn delete_event(&self, user_id: &str, event_id: &str) -> Result<bool, AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let affected = conn
            .execute("DELETE FROM events WHERE id = ?1", params![event_id])
            .map_err(|e| AppError::new("db", format!("Failed to delete event: {e}")))?;
        Ok(affected > 0)
    }

    // 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Forgetting Curve 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾?

    pub fn record_recall(
        &self,
        user_id: &str,
        event_id: &str,
        recall_boost: f64,
    ) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let now = chrono::Utc::now().to_rfc3339();

        let stability: f64 = conn
            .query_row(
                "SELECT stability FROM events WHERE id = ?1",
                params![event_id],
                |row| row.get(0),
            )
            .unwrap_or(30.0);

        let recall_count: i64 = conn
            .query_row(
                "SELECT recall_count FROM events WHERE id = ?1",
                params![event_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let new_stability = stability * (1.0 + recall_boost);
        let new_count = recall_count + 1;

        conn.execute(
            "UPDATE events SET stability = ?1, recall_count = ?2, last_recalled_at = ?3, updated_at = ?4 WHERE id = ?5",
            params![new_stability, new_count, now, now, event_id],
        )
        .map_err(|e| AppError::new("db", format!("Failed to record recall: {e}")))?;

        Ok(())
    }

    pub fn decay_all_events(&self, user_id: &str, min_strength: f64) -> Result<usize, AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let now = chrono::Utc::now().to_rfc3339();

        let mut stmt = conn
            .prepare("SELECT id, created_at, stability FROM events")
            .map_err(|e| AppError::new("db", format!("Failed to prepare decay query: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>("id")?,
                    row.get::<_, String>("created_at")?,
                    row.get::<_, f64>("stability").unwrap_or(30.0),
                ))
            })
            .map_err(|e| AppError::new("db", format!("Failed to query events for decay: {e}")))?;

        let mut count = 0usize;
        for row in rows {
            let (id, created_at, stability) =
                row.map_err(|e| AppError::new("db", format!("Failed to read decay row: {e}")))?;
            let strength = compute_strength(&created_at, stability);
            conn.execute(
                "UPDATE events SET strength = ?1, updated_at = ?2 WHERE id = ?3",
                params![strength, now, id],
            )
            .map_err(|e| AppError::new("db", format!("Failed to update decay: {e}")))?;
            if strength < min_strength {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn cleanup_forgotten_events(
        &self,
        user_id: &str,
        min_strength: f64,
    ) -> Result<usize, AppError> {
        let conn = self.conn_with_schema(user_id)?;

        let mut stmt = conn
            .prepare("SELECT id, created_at, stability FROM events")
            .map_err(|e| AppError::new("db", format!("Failed to prepare cleanup query: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>("id")?,
                    row.get::<_, String>("created_at")?,
                    row.get::<_, f64>("stability").unwrap_or(30.0),
                ))
            })
            .map_err(|e| AppError::new("db", format!("Failed to query events for cleanup: {e}")))?;

        let mut to_delete = Vec::new();
        for row in rows {
            let (id, created_at, stability) =
                row.map_err(|e| AppError::new("db", format!("Failed to read cleanup row: {e}")))?;
            let strength = compute_strength(&created_at, stability);
            if strength < min_strength {
                to_delete.push(id);
            }
        }

        let count = to_delete.len();
        for id in &to_delete {
            conn.execute("DELETE FROM events WHERE id = ?1", params![id])
                .map_err(|e| {
                    AppError::new("db", format!("Failed to delete forgotten event: {e}"))
                })?;
        }
        Ok(count)
    }

    // 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Conversation Turns 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌?

    pub fn save_conversation_turn(
        &self,
        user_id: &str,
        user_msg: &str,
        ai_msg: &str,
        summary: Option<&str>,
        emotions: &[String],
    ) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let now = chrono::Utc::now().to_rfc3339();
        let summary_str = summary.unwrap_or("");
        let emotions_json = serde_json::to_string(emotions).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO conversation_turns (user_msg, ai_msg, summary, emotions, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_msg, ai_msg, summary_str, emotions_json, now],
        )
        .map_err(|e| AppError::new("db", format!("Failed to save conversation turn: {e}")))?;

        let turn_id: i64 = conn
            .query_row("SELECT last_insert_rowid()", [], |row| row.get(0))
            .map_err(|e| AppError::new("db", format!("Failed to get turn id: {e}")))?;

        let fts_summary = if summary_str.is_empty() {
            truncate_chars(user_msg, 200)
        } else {
            summary_str.to_string()
        };
        let fts_user_msg = truncate_chars(user_msg, 500);

        conn.execute(
            "INSERT INTO turn_search (rowid, summary, user_msg) VALUES (?1, ?2, ?3)",
            params![turn_id, fts_summary, fts_user_msg],
        )
        .map_err(|e| AppError::new("db", format!("Failed to index turn for FTS5: {e}")))?;

        Ok(())
    }

    pub fn search_conversations(
        &self,
        user_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ConversationTurn>, AppError> {
        let conn = self.conn_with_schema(user_id)?;

        // Search FTS5 index, then look up the corresponding conversation_turns row.
        // Since we insert with explicit rowid = turn_id, the FTS5 rowid matches.
        let mut stmt = conn
            .prepare(
                "SELECT ts.rowid as id, ts.summary, ct.emotions, ct.created_at
                 FROM turn_search ts
                 LEFT JOIN conversation_turns ct ON ct.id = ts.rowid
                 WHERE turn_search MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| AppError::new("db", format!("Failed to prepare search: {e}")))?;

        let rows = stmt
            .query_map(params![query, limit as i64], |row| {
                Ok(ConversationTurn {
                    id: row.get("id")?,
                    summary: row.get("summary").unwrap_or_default(),
                    emotions: parse_json_array(
                        &row.get::<_, String>("emotions").unwrap_or_default(),
                    ),
                    created_at: row.get("created_at").unwrap_or_default(),
                })
            })
            .map_err(|e| AppError::new("db", format!("Failed to search conversations: {e}")))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(
                row.map_err(|e| AppError::new("db", format!("Failed to read search result: {e}")))?,
            );
        }
        Ok(results)
    }

    // 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Insights 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾?

    /// Search conversation turns by keyword using LIKE (works with Chinese text
    /// where FTS5's unicode61 tokenizer fails to split CJK characters).
    pub fn search_conversations_like(
        &self,
        user_id: &str,
        keyword: &str,
        limit: usize,
    ) -> Result<Vec<ConversationTurn>, AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let pattern = format!("%{}%", keyword);

        let mut stmt = conn
            .prepare(
                "SELECT id, summary, emotions, created_at
                 FROM conversation_turns
                 WHERE summary LIKE ?1 OR user_msg LIKE ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| AppError::new("db", format!("Failed to prepare LIKE search: {e}")))?;

        let rows = stmt
            .query_map(params![pattern, limit as i64], |row| {
                Ok(ConversationTurn {
                    id: row.get("id")?,
                    summary: row.get("summary").unwrap_or_default(),
                    emotions: parse_json_array(
                        &row.get::<_, String>("emotions").unwrap_or_default(),
                    ),
                    created_at: row.get("created_at").unwrap_or_default(),
                })
            })
            .map_err(|e| AppError::new("db", format!("Failed to search by keyword: {e}")))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(
                row.map_err(|e| AppError::new("db", format!("Failed to read LIKE result: {e}")))?,
            );
        }
        Ok(results)
    }

    pub fn save_insight(
        &self,
        user_id: &str,
        category: &str,
        content: &str,
        confidence: f64,
        source: &str,
    ) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO insights (category, content, confidence, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![category, content, confidence, source, now, now],
        )
        .map_err(|e| AppError::new("db", format!("Failed to save insight: {e}")))?;
        Ok(())
    }

    pub fn get_insights(
        &self,
        user_id: &str,
        category: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Insight>, AppError> {
        let conn = self.conn_with_schema(user_id)?;

        let mut results = Vec::new();

        let map_insight = |row: &rusqlite::Row| -> rusqlite::Result<Insight> {
            Ok(Insight {
                id: row.get("id")?,
                category: row.get("category")?,
                content: row.get("content")?,
                confidence: row.get("confidence").unwrap_or(0.5),
                source: row.get("source").unwrap_or_default(),
                created_at: row.get("created_at").unwrap_or_default(),
            })
        };

        if let Some(cat) = category {
            let mut stmt = conn.prepare(
                "SELECT * FROM insights WHERE category = ?1 ORDER BY confidence DESC, created_at DESC LIMIT ?2"
            ).map_err(|e| AppError::new("db", format!("Failed to prepare insights query: {e}")))?;
            let rows = stmt
                .query_map(params![cat, limit as i64], map_insight)
                .map_err(|e| AppError::new("db", format!("Failed to query insights: {e}")))?;
            for row in rows {
                results.push(
                    row.map_err(|e| AppError::new("db", format!("Failed to read insight: {e}")))?,
                );
            }
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT * FROM insights ORDER BY confidence DESC, created_at DESC LIMIT ?1",
                )
                .map_err(|e| {
                    AppError::new("db", format!("Failed to prepare insights query: {e}"))
                })?;
            let rows = stmt
                .query_map(params![limit as i64], map_insight)
                .map_err(|e| AppError::new("db", format!("Failed to query insights: {e}")))?;
            for row in rows {
                results.push(
                    row.map_err(|e| AppError::new("db", format!("Failed to read insight: {e}")))?,
                );
            }
        }

        Ok(results)
    }

    // 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Observations 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹?

    pub fn add_observation(&self, user_id: &str, obs: &Observation) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        conn.execute(
            "INSERT INTO observations (id, date, content, category, source, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                obs.id,
                obs.date,
                obs.content,
                obs.category,
                obs.source,
                obs.created_at
            ],
        )
        .map_err(|e| AppError::new("db", format!("Failed to add observation: {e}")))?;
        Ok(())
    }

    pub fn query_observations(
        &self,
        user_id: &str,
        category: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Observation>, AppError> {
        let conn = self.conn_with_schema(user_id)?;

        let mut results = Vec::new();

        let map_obs = |row: &rusqlite::Row| -> rusqlite::Result<Observation> {
            Ok(Observation {
                id: row.get("id")?,
                date: row.get("date")?,
                content: row.get("content")?,
                category: row.get("category")?,
                source: row.get("source")?,
                created_at: row.get("created_at")?,
            })
        };

        if let Some(cat) = category {
            let mut stmt = conn
                .prepare(
                    "SELECT * FROM observations WHERE category = ?1 ORDER BY date DESC LIMIT ?2",
                )
                .map_err(|e| {
                    AppError::new("db", format!("Failed to prepare observations query: {e}"))
                })?;
            let rows = stmt
                .query_map(params![cat, limit as i64], map_obs)
                .map_err(|e| AppError::new("db", format!("Failed to query observations: {e}")))?;
            for row in rows {
                results.push(row.map_err(|e| {
                    AppError::new("db", format!("Failed to read observation: {e}"))
                })?);
            }
        } else {
            let mut stmt = conn
                .prepare("SELECT * FROM observations ORDER BY date DESC LIMIT ?1")
                .map_err(|e| {
                    AppError::new("db", format!("Failed to prepare observations query: {e}"))
                })?;
            let rows = stmt
                .query_map(params![limit as i64], map_obs)
                .map_err(|e| AppError::new("db", format!("Failed to query observations: {e}")))?;
            for row in rows {
                results.push(row.map_err(|e| {
                    AppError::new("db", format!("Failed to read observation: {e}"))
                })?);
            }
        }

        Ok(results)
    }

    // 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Topics 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?

    pub fn add_topic(&self, user_id: &str, topic: &Topic) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO topics (id, name, description, first_mentioned, last_mentioned,
             mention_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                topic.id,
                topic.name,
                topic.description,
                topic.first_mentioned.as_ref().unwrap_or(&now),
                topic.last_mentioned.as_ref().unwrap_or(&now),
                topic.mention_count,
                now,
                now,
            ],
        )
        .map_err(|e| AppError::new("db", format!("Failed to add topic: {e}")))?;
        Ok(())
    }

    pub fn query_topics(&self, user_id: &str, limit: usize) -> Result<Vec<Topic>, AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let mut stmt = conn
            .prepare("SELECT * FROM topics ORDER BY last_mentioned DESC LIMIT ?1")
            .map_err(|e| AppError::new("db", format!("Failed to prepare topics query: {e}")))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(Topic {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    description: row.get("description").unwrap_or_default(),
                    first_mentioned: row.get("first_mentioned")?,
                    last_mentioned: row.get("last_mentioned")?,
                    mention_count: row.get("mention_count").unwrap_or(1),
                })
            })
            .map_err(|e| AppError::new("db", format!("Failed to query topics: {e}")))?;

        let mut results = Vec::new();
        for row in rows {
            results
                .push(row.map_err(|e| AppError::new("db", format!("Failed to read topic: {e}")))?);
        }
        Ok(results)
    }

    pub fn get_topic_by_name(&self, user_id: &str, name: &str) -> Result<Option<Topic>, AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let mut stmt = conn
            .prepare("SELECT * FROM topics WHERE name = ?1")
            .map_err(|e| AppError::new("db", format!("Failed to prepare topic by name: {e}")))?;

        let result = stmt
            .query_row(params![name], |row| {
                Ok(Topic {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    description: row.get("description").unwrap_or_default(),
                    first_mentioned: row.get("first_mentioned")?,
                    last_mentioned: row.get("last_mentioned")?,
                    mention_count: row.get("mention_count").unwrap_or(1),
                })
            })
            .ok();

        Ok(result)
    }

    pub fn update_topic(
        &self,
        user_id: &str,
        topic_id: &str,
        last_mentioned: Option<&str>,
        mention_count: Option<u32>,
        description: Option<&str>,
    ) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let now = chrono::Utc::now().to_rfc3339();

        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(lm) = last_mentioned {
            sets.push("last_mentioned = ?".to_string());
            values.push(Box::new(lm.to_string()));
        }
        if let Some(mc) = mention_count {
            sets.push("mention_count = ?".to_string());
            values.push(Box::new(mc));
        }
        if let Some(d) = description {
            sets.push("description = ?".to_string());
            values.push(Box::new(d.to_string()));
        }

        if sets.is_empty() {
            return Ok(());
        }

        sets.push("updated_at = ?".to_string());
        values.push(Box::new(now));
        values.push(Box::new(topic_id.to_string()));

        let sql = format!("UPDATE topics SET {} WHERE id = ?", sets.join(", "));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            values.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, param_refs.as_slice())
            .map_err(|e| AppError::new("db", format!("Failed to update topic: {e}")))?;

        Ok(())
    }

    pub fn link_topic(
        &self,
        user_id: &str,
        topic_id: &str,
        item_id: &str,
        item_type: &str,
    ) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        conn.execute(
            "INSERT OR IGNORE INTO topic_links (topic_id, item_id, item_type) VALUES (?1, ?2, ?3)",
            params![topic_id, item_id, item_type],
        )
        .map_err(|e| AppError::new("db", format!("Failed to link topic: {e}")))?;
        Ok(())
    }

    pub fn get_topic_links(
        &self,
        user_id: &str,
        topic_id: &str,
    ) -> Result<Vec<TopicLink>, AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let mut stmt = conn
            .prepare("SELECT * FROM topic_links WHERE topic_id = ?1")
            .map_err(|e| {
                AppError::new("db", format!("Failed to prepare topic links query: {e}"))
            })?;

        let rows = stmt
            .query_map(params![topic_id], |row| {
                Ok(TopicLink {
                    topic_id: row.get("topic_id")?,
                    item_id: row.get("item_id")?,
                    item_type: row.get("item_type")?,
                })
            })
            .map_err(|e| AppError::new("db", format!("Failed to query topic links: {e}")))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(
                row.map_err(|e| AppError::new("db", format!("Failed to read topic link: {e}")))?,
            );
        }
        Ok(results)
    }

    // 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Projects 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾?

    pub fn add_project(&self, user_id: &str, project: &Project) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO projects (id, title, description, status, start_date, end_date,
             event_ids, summary, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                project.id,
                project.title,
                project.description,
                project.status,
                project.start_date.as_ref().unwrap_or(&now),
                project.end_date.as_ref().unwrap_or(&now),
                serde_json::to_string(&project.event_ids).unwrap_or_else(|_| "[]".to_string()),
                project.summary,
                now,
                now,
            ],
        )
        .map_err(|e| AppError::new("db", format!("Failed to add project: {e}")))?;
        Ok(())
    }

    pub fn query_projects(
        &self,
        user_id: &str,
        status: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Project>, AppError> {
        let conn = self.conn_with_schema(user_id)?;

        let sql = if status.is_some() {
            "SELECT * FROM projects WHERE status = ?1 ORDER BY updated_at DESC LIMIT ?2"
        } else {
            "SELECT * FROM projects ORDER BY updated_at DESC LIMIT ?1"
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| AppError::new("db", format!("Failed to prepare projects query: {e}")))?;

        let map_project = |row: &rusqlite::Row| -> rusqlite::Result<Project> {
            let event_ids_str: String = row.get("event_ids").unwrap_or_default();
            Ok(Project {
                id: row.get("id")?,
                title: row.get("title")?,
                description: row.get("description").unwrap_or_default(),
                status: row.get("status").unwrap_or_else(|_| "active".to_string()),
                start_date: row.get("start_date")?,
                end_date: row.get("end_date")?,
                event_ids: serde_json::from_str(&event_ids_str).unwrap_or_default(),
                summary: row.get("summary").unwrap_or_default(),
                created_at: row.get("created_at").unwrap_or_default(),
                updated_at: row.get("updated_at").unwrap_or_default(),
            })
        };

        let mut results = Vec::new();
        if let Some(s) = status {
            let rows = stmt
                .query_map(params![s, limit as i64], map_project)
                .map_err(|e| AppError::new("db", format!("Failed to query projects: {e}")))?;
            for row in rows {
                results.push(
                    row.map_err(|e| AppError::new("db", format!("Failed to read project: {e}")))?,
                );
            }
        } else {
            let rows = stmt
                .query_map(params![limit as i64], map_project)
                .map_err(|e| AppError::new("db", format!("Failed to query projects: {e}")))?;
            for row in rows {
                results.push(
                    row.map_err(|e| AppError::new("db", format!("Failed to read project: {e}")))?,
                );
            }
        }
        Ok(results)
    }

    pub fn get_project(
        &self,
        user_id: &str,
        project_id: &str,
    ) -> Result<Option<Project>, AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let result = conn
            .query_row(
                "SELECT * FROM projects WHERE id = ?1",
                params![project_id],
                |row| {
                    let event_ids_str: String = row.get("event_ids").unwrap_or_default();
                    Ok(Project {
                        id: row.get("id")?,
                        title: row.get("title")?,
                        description: row.get("description").unwrap_or_default(),
                        status: row.get("status").unwrap_or_else(|_| "active".to_string()),
                        start_date: row.get("start_date")?,
                        end_date: row.get("end_date")?,
                        event_ids: serde_json::from_str(&event_ids_str).unwrap_or_default(),
                        summary: row.get("summary").unwrap_or_default(),
                        created_at: row.get("created_at").unwrap_or_default(),
                        updated_at: row.get("updated_at").unwrap_or_default(),
                    })
                },
            )
            .ok();
        Ok(result)
    }

    pub fn update_project(
        &self,
        user_id: &str,
        project_id: &str,
        title: Option<&str>,
        description: Option<&str>,
        status: Option<&str>,
        event_ids: Option<&Vec<String>>,
        summary: Option<&str>,
    ) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let now = chrono::Utc::now().to_rfc3339();

        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(t) = title {
            sets.push("title = ?");
            values.push(Box::new(t.to_string()));
        }
        if let Some(d) = description {
            sets.push("description = ?");
            values.push(Box::new(d.to_string()));
        }
        if let Some(s) = status {
            sets.push("status = ?");
            values.push(Box::new(s.to_string()));
        }
        if let Some(eids) = event_ids {
            sets.push("event_ids = ?");
            values.push(Box::new(
                serde_json::to_string(eids).unwrap_or_else(|_| "[]".to_string()),
            ));
        }
        if let Some(s) = summary {
            sets.push("summary = ?");
            values.push(Box::new(s.to_string()));
        }

        if sets.is_empty() {
            return Ok(());
        }

        sets.push("updated_at = ?");
        values.push(Box::new(now));
        values.push(Box::new(project_id.to_string()));

        let sql = format!("UPDATE projects SET {} WHERE id = ?", sets.join(", "));
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            values.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, param_refs.as_slice())
            .map_err(|e| AppError::new("db", format!("Failed to update project: {e}")))?;

        Ok(())
    }

    // 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Growth Lines 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹?

    pub fn add_growth_line(
        &self,
        user_id: &str,
        id: &str,
        dimension: &str,
    ) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO growth_lines (id, dimension, records, created_at, updated_at)
             VALUES (?1, ?2, '[]', ?3, ?4)",
            params![id, dimension, now, now],
        )
        .map_err(|e| AppError::new("db", format!("Failed to add growth line: {e}")))?;
        Ok(())
    }

    pub fn query_growth_lines(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<GrowthLine>, AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let mut stmt = conn
            .prepare("SELECT * FROM growth_lines ORDER BY updated_at DESC LIMIT ?1")
            .map_err(|e| {
                AppError::new("db", format!("Failed to prepare growth lines query: {e}"))
            })?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                let records_str: String = row.get("records").unwrap_or_default();
                Ok(GrowthLine {
                    id: row.get("id")?,
                    dimension: row.get("dimension")?,
                    records: serde_json::from_str(&records_str).unwrap_or_default(),
                })
            })
            .map_err(|e| AppError::new("db", format!("Failed to query growth lines: {e}")))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(
                row.map_err(|e| AppError::new("db", format!("Failed to read growth line: {e}")))?,
            );
        }
        Ok(results)
    }

    pub fn get_growth_line(
        &self,
        user_id: &str,
        dimension: &str,
    ) -> Result<Option<GrowthLine>, AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let result = conn
            .query_row(
                "SELECT * FROM growth_lines WHERE dimension = ?1",
                params![dimension],
                |row| {
                    let records_str: String = row.get("records").unwrap_or_default();
                    Ok(GrowthLine {
                        id: row.get("id")?,
                        dimension: row.get("dimension")?,
                        records: serde_json::from_str(&records_str).unwrap_or_default(),
                    })
                },
            )
            .ok();
        Ok(result)
    }

    pub fn update_growth_line_records(
        &self,
        user_id: &str,
        gl_id: &str,
        records: &[serde_json::Value],
    ) -> Result<(), AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let now = chrono::Utc::now().to_rfc3339();
        let records_json = serde_json::to_string(records).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "UPDATE growth_lines SET records = ?1, updated_at = ?2 WHERE id = ?3",
            params![records_json, now, gl_id],
        )
        .map_err(|e| AppError::new("db", format!("Failed to update growth line: {e}")))?;
        Ok(())
    }

    // ─── Date-based Queries ────────────────────────────────────────────────

    /// Query events whose `created_at` falls within the given date (local day range).
    /// Query events whose `created_at` falls within the given UTC timestamp range.
    /// `utc_start` and `utc_end` should be ISO 8601 UTC strings (e.g. "2026-05-29T16:00:00Z").
    /// The caller is responsible for converting local date boundaries to UTC.
    pub fn query_events_by_date(
        &self,
        user_id: &str,
        utc_start: &str,
        utc_end: &str,
        limit: usize,
    ) -> Result<Vec<Event>, AppError> {
        let conn = self.conn_with_schema(user_id)?;

        let mut stmt = conn
            .prepare(
                "SELECT * FROM events WHERE created_at >= ?1 AND created_at <= ?2
                 ORDER BY created_at ASC LIMIT ?3",
            )
            .map_err(|e| AppError::new("db", format!("Failed to prepare events by date: {e}")))?;

        let rows = stmt
            .query_map(params![utc_start, utc_end, limit as i64], |row| {
                let created_at: String = row.get("created_at")?;
                let stability: f64 = row.get("stability").unwrap_or(30.0);
                let strength = compute_strength(&created_at, stability);
                Ok(Event {
                    id: row.get("id")?,
                    content: row.get("content")?,
                    emotions: parse_json_array(
                        &row.get::<_, String>("emotions").unwrap_or_default(),
                    ),
                    importance: row.get("importance")?,
                    event_type: row.get("event_type")?,
                    strength,
                    stability,
                    recall_count: row.get("recall_count").unwrap_or(0),
                    last_recalled_at: row.get("last_recalled_at")?,
                    created_at: created_at.clone(),
                    updated_at: row.get("updated_at")?,
                })
            })
            .map_err(|e| AppError::new("db", format!("Failed to query events by date: {e}")))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(
                row.map_err(|e| AppError::new("db", format!("Failed to read event row: {e}")))?,
            );
        }
        Ok(results)
    }

    /// Query conversation turns whose `created_at` falls within the given UTC timestamp range.
    /// `utc_start` and `utc_end` should be ISO 8601 UTC strings (e.g. "2026-05-29T16:00:00Z").
    /// The caller is responsible for converting local date boundaries to UTC.
    pub fn query_conversation_turns_by_date(
        &self,
        user_id: &str,
        utc_start: &str,
        utc_end: &str,
        limit: usize,
    ) -> Result<Vec<ConversationTurn>, AppError> {
        let conn = self.conn_with_schema(user_id)?;

        let mut stmt = conn
            .prepare(
                "SELECT id, summary, emotions, created_at FROM conversation_turns
                 WHERE created_at >= ?1 AND created_at <= ?2
                 ORDER BY created_at ASC LIMIT ?3",
            )
            .map_err(|e| {
                AppError::new(
                    "db",
                    format!("Failed to prepare turns by date: {e}"),
                )
            })?;

        let rows = stmt
            .query_map(params![utc_start, utc_end, limit as i64], |row| {
                Ok(ConversationTurn {
                    id: row.get("id")?,
                    summary: row.get("summary").unwrap_or_default(),
                    emotions: parse_json_array(
                        &row.get::<_, String>("emotions").unwrap_or_default(),
                    ),
                    created_at: row.get("created_at").unwrap_or_default(),
                })
            })
            .map_err(|e| {
                AppError::new("db", format!("Failed to query turns by date: {e}"))
            })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(
                row.map_err(|e| AppError::new("db", format!("Failed to read turn row: {e}")))?,
            );
        }
        Ok(results)
    }

    pub fn delete_growth_line(&self, user_id: &str, gl_id: &str) -> Result<bool, AppError> {
        let conn = self.conn_with_schema(user_id)?;
        let affected = conn
            .execute("DELETE FROM growth_lines WHERE id = ?1", params![gl_id])
            .map_err(|e| AppError::new("db", format!("Failed to delete growth line: {e}")))?;
        Ok(affected > 0)
    }
}

// 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Forgetting Curve 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹?

pub fn compute_strength(created_at: &str, stability: f64) -> f64 {
    let parsed = chrono::DateTime::parse_from_rfc3339(created_at)
        .map(|dt| dt.to_utc())
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%dT%H:%M:%S%.f")
                .map(|ndt| ndt.and_utc())
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%dT%H:%M:%S")
                .map(|ndt| ndt.and_utc())
        });

    let Ok(parsed) = parsed else {
        return 1.0;
    };

    let elapsed = chrono::Utc::now().signed_duration_since(parsed);
    let days_elapsed = elapsed.num_seconds() as f64 / 86400.0;
    (-days_elapsed / stability).exp()
}

// 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Helpers 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚?

fn parse_json_array(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
}

// 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺?Tests 闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜闂傚倷绀侀崯鍧楀储濠婂牆纾婚柟鍓х帛閻撳啴鏌涜箛鎿冩Ц濞存粓绠栧娲礃閹绘帒杈呴梺绋款儐閹瑰洭寮诲澶婄濠㈣泛锕ｆ竟鏇㈡⒒娴ｇ鏆遍柛妯荤矒瀹曟垿骞樼紒妯煎帗闂佺绻愰ˇ顖涚妤ｅ啯鈷戦柛鎰絻鐢劑鏌涚€ｎ偅宕岄柡灞界Ч瀹曟寰勬繝浣割棜

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> DbState {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("gc_test_db_{id}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        DbState::new(dir)
    }

    #[test]
    fn test_compute_strength_at_creation() {
        let now = chrono::Utc::now().to_rfc3339();
        let strength = compute_strength(&now, 30.0);
        assert!(
            strength > 0.99,
            "Fresh event should have strength ~1.0, got {strength}"
        );
    }

    #[test]
    fn test_compute_strength_old_event() {
        let past = "2026-01-01T00:00:00+00:00";
        let strength = compute_strength(past, 30.0);
        assert!(
            strength < 0.1,
            "Old event should have low strength, got {strength}"
        );
    }

    #[test]
    fn test_init_db_and_add_event() {
        let db = test_db();
        db.init_db("test_user").unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        let event = Event {
            id: "evt001".to_string(),
            content: "Had a great conversation".to_string(),
            emotions: vec!["joy".to_string()],
            importance: 0.8,
            event_type: Some("milestone".to_string()),
            strength: 1.0,
            stability: 30.0,
            recall_count: 0,
            last_recalled_at: None,
            created_at: now.clone(),
            updated_at: now,
        };

        db.add_event("test_user", &event, 30.0).unwrap();

        let events = db
            .query_events("test_user", &QueryEventsParams::default())
            .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "evt001");
        assert_eq!(events[0].emotions, vec!["joy"]);
    }

    #[test]
    fn test_query_events_initializes_schema() {
        let db = test_db();
        let events = db
            .query_events("test_user", &QueryEventsParams::default())
            .unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_rejects_path_traversal_user_id() {
        let db = test_db();
        let err = db
            .query_events("../outside", &QueryEventsParams::default())
            .unwrap_err();
        assert_eq!(err.code, "invalidUserId");
    }

    #[test]
    fn test_delete_event() {
        let db = test_db();
        db.init_db("test_user").unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        let event = Event {
            id: "evt_del".to_string(),
            content: "To be deleted".to_string(),
            emotions: vec![],
            importance: 0.5,
            event_type: None,
            strength: 1.0,
            stability: 30.0,
            recall_count: 0,
            last_recalled_at: None,
            created_at: now.clone(),
            updated_at: now,
        };
        db.add_event("test_user", &event, 30.0).unwrap();
        assert!(db.delete_event("test_user", "evt_del").unwrap());
        assert!(!db.delete_event("test_user", "evt_del").unwrap());
    }

    #[test]
    fn test_topic_crud() {
        let db = test_db();
        db.init_db("test_user").unwrap();

        let topic = Topic {
            id: "top001".to_string(),
            name: "career_choice".to_string(),
            description: "Career direction discussion".to_string(),
            first_mentioned: Some("2026-05-30T12:00:00+00:00".to_string()),
            last_mentioned: Some("2026-05-30T12:00:00+00:00".to_string()),
            mention_count: 1,
        };
        db.add_topic("test_user", &topic).unwrap();

        let found = db.get_topic_by_name("test_user", "career_choice").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "top001");

        let topics = db.query_topics("test_user", 50).unwrap();
        assert_eq!(topics.len(), 1);

        db.update_topic(
            "test_user",
            "top001",
            Some("2026-05-30T14:00:00+00:00"),
            Some(2),
            None,
        )
        .unwrap();

        let updated = db
            .get_topic_by_name("test_user", "career_choice")
            .unwrap()
            .unwrap();
        assert_eq!(updated.mention_count, 2);
    }

    #[test]
    fn test_topic_linking() {
        let db = test_db();
        db.init_db("test_user").unwrap();

        let topic = Topic {
            id: "top002".to_string(),
            name: "fitness".to_string(),
            description: String::new(),
            first_mentioned: None,
            last_mentioned: None,
            mention_count: 1,
        };
        db.add_topic("test_user", &topic).unwrap();
        db.link_topic("test_user", "top002", "evt001", "event")
            .unwrap();
        db.link_topic("test_user", "top002", "evt001", "event")
            .unwrap(); // idempotent

        let links = db.get_topic_links("test_user", "top002").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].item_id, "evt001");
    }

    #[test]
    fn test_project_crud() {
        let db = test_db();
        db.init_db("test_user").unwrap();

        let project = Project {
            id: "prj001".to_string(),
            title: "Learn Rust".to_string(),
            description: "Migrate to Tauri".to_string(),
            status: "active".to_string(),
            start_date: None,
            end_date: None,
            event_ids: vec!["evt001".to_string()],
            summary: String::new(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        db.add_project("test_user", &project).unwrap();

        let found = db.get_project("test_user", "prj001").unwrap();
        assert!(found.is_some());
        let p = found.as_ref().unwrap();
        assert_eq!(p.title, "Learn Rust");
        assert_eq!(p.event_ids, vec!["evt001"]);

        db.update_project(
            "test_user",
            "prj001",
            None,
            None,
            Some("completed"),
            None,
            None,
        )
        .unwrap();
        let updated = db.get_project("test_user", "prj001").unwrap().unwrap();
        assert_eq!(updated.status, "completed");
    }

    #[test]
    fn test_growth_lines() {
        let db = test_db();
        db.init_db("test_user").unwrap();

        db.add_growth_line("test_user", "gl001", "emotion_management")
            .unwrap();

        let lines = db.query_growth_lines("test_user", 50).unwrap();
        assert_eq!(lines.len(), 1);

        let found = db
            .get_growth_line("test_user", "emotion_management")
            .unwrap();
        assert!(found.is_some());

        let records = vec![serde_json::json!({"date": "2026-05-30", "note": "more stable"})];
        db.update_growth_line_records("test_user", "gl001", &records)
            .unwrap();

        let updated = db
            .get_growth_line("test_user", "emotion_management")
            .unwrap()
            .unwrap();
        assert_eq!(updated.records.len(), 1);

        assert!(db.delete_growth_line("test_user", "gl001").unwrap());
    }

    #[test]
    fn test_conversation_search() {
        let db = test_db();
        db.init_db("test_user").unwrap();

        // Use a single connection for both write and search to avoid FTS5 cross-conn issues
        let user_dir = db.base_dir.join("test_user");
        std::fs::create_dir_all(&user_dir).unwrap();
        let db_path = user_dir.join("events.db");
        let conn = Connection::open(&db_path).unwrap();

        // First, check if FTS5 is available
        let fts_ok: bool = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='turn_search'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap()
            > 0;
        assert!(fts_ok, "FTS5 table turn_search not created");

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO conversation_turns (user_msg, ai_msg, summary, emotions, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                "I had a job interview today",
                "That sounds great!",
                "user mentioned new job opportunity",
                "[\"hope\",\"anxiety\"]",
                now,
            ],
        )
        .unwrap();

        let turn_id: i64 = conn
            .query_row("SELECT last_insert_rowid()", [], |r| r.get(0))
            .unwrap();
        conn.execute(
            "INSERT INTO turn_search (rowid, summary, user_msg) VALUES (?1, ?2, ?3)",
            params![
                turn_id,
                "user mentioned new job opportunity",
                "I had a job interview today"
            ],
        )
        .unwrap();

        // Search with ASCII term first to verify FTS5 works
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM turn_search WHERE turn_search MATCH 'interview'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "FTS5 should find 'interview' but got {count}");

        drop(conn);

        let results = db
            .search_conversations("test_user", "interview", 5)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].emotions, vec!["hope", "anxiety"]);
    }

    #[test]
    fn test_save_conversation_turn_truncates_unicode_safely() {
        let db = test_db();
        let user_msg = "hello 😀".repeat(300);

        db.save_conversation_turn(
            "test_user",
            &user_msg,
            "I hear you.",
            None,
            &["calm".to_string()],
        )
        .unwrap();

        let results = db.search_conversations("test_user", "hello", 5).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_insights() {
        let db = test_db();
        db.init_db("test_user").unwrap();

        db.save_insight(
            "test_user",
            "emotion_pattern",
            "weekend anxiety",
            0.7,
            "reflection",
        )
        .unwrap();

        let insights = db
            .get_insights("test_user", Some("emotion_pattern"), 10)
            .unwrap();
        assert_eq!(insights.len(), 1);
        assert_eq!(insights[0].content, "weekend anxiety");
    }

    #[test]
    fn test_observations() {
        let db = test_db();
        db.init_db("test_user").unwrap();

        let obs = Observation {
            id: "obs001".to_string(),
            date: "2026-05-30".to_string(),
            content: "User stayed calmer during conflict".to_string(),
            category: Some("behavior".to_string()),
            source: Some("reflection".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        db.add_observation("test_user", &obs).unwrap();

        let results = db
            .query_observations("test_user", Some("behavior"), 10)
            .unwrap();
        assert_eq!(results.len(), 1);
    }
}
