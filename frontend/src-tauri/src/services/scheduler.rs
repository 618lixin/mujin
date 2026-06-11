use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use reqwest::Client;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{self, Duration};

use super::config::AiConfig;
use super::database::DbState;
use super::llm::call_cheap_llm;
use super::memory::load_last_activity;
use super::notes::{default_base_dir, AppError};
use super::types::{ChatMessage, PendingMessage};

// ─── Pending Messages State ───────────────────────────────────────────────

pub struct PendingMessages {
    pub messages: Mutex<HashMap<String, PendingMessage>>,
}

impl Default for PendingMessages {
    fn default() -> Self {
        Self {
            messages: Mutex::new(HashMap::new()),
        }
    }
}

pub fn get_pending_message(state: &PendingMessages, user_id: &str) -> Option<PendingMessage> {
    state
        .messages
        .lock()
        .ok()
        .and_then(|mut map| map.remove(user_id))
}

// ─── User Scanning ────────────────────────────────────────────────────────

pub fn scan_user_dirs(base_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(base_dir) else {
        return vec![];
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter(|e| e.path().join("user_profile.md").exists())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect()
}

// ─── Idle Calculation ─────────────────────────────────────────────────────

pub fn idle_minutes(last_active: Option<&str>) -> f64 {
    let Some(ts) = last_active else {
        return 0.0;
    };
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(ts) else {
        return 0.0;
    };
    let elapsed = chrono::Utc::now().signed_duration_since(parsed.to_utc());
    elapsed.num_seconds() as f64 / 60.0
}

// ─── Heartbeat Loop ───────────────────────────────────────────────────────

pub fn start_heartbeat(app: AppHandle, interval_minutes: u64) {
    tauri::async_runtime::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval_minutes * 60));

        loop {
            interval.tick().await;

            let base_dir = match default_base_dir() {
                Ok(d) => d,
                Err(_) => continue,
            };

            let config = match super::config::load_ai_config(&base_dir) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if config.llm_api_key.is_empty() {
                continue;
            }

            let db_state = app.state::<DbState>();
            let pending = app.state::<PendingMessages>();

            if let Err(e) =
                heartbeat_tick(&app, db_state.inner(), pending.inner(), &config, &base_dir).await
            {
                eprintln!("heartbeat tick error: {e}");
            }
        }
    });
}

async fn heartbeat_tick(
    app: &AppHandle,
    db: &DbState,
    pending: &PendingMessages,
    config: &AiConfig,
    base_dir: &Path,
) -> Result<(), AppError> {
    let client = Client::new();

    for user_id in scan_user_dirs(base_dir) {
        // 1. Forgetting curve maintenance
        let _ = db.init_db(&user_id);
        let _ = db.decay_all_events(&user_id, config.forget_min_strength);
        let _ = db.cleanup_forgotten_events(&user_id, config.forget_min_strength);

        // 2. Check proactive messages
        if !config.heartbeat_proactive_enabled {
            continue;
        }

        // Already has pending message
        if let Ok(map) = pending.messages.lock() {
            if map.contains_key(&user_id) {
                continue;
            }
        }

        let last_active = load_last_activity(base_dir, &user_id).ok().flatten();
        let idle = idle_minutes(last_active.as_deref());

        let should_send = if idle > config.heartbeat_max_idle_minutes as f64 {
            true
        } else if idle > config.heartbeat_min_idle_minutes as f64 {
            true
        } else {
            false
        };

        if !should_send {
            continue;
        }

        if let Some(message) = generate_proactive_message(&client, config, idle).await {
            let pm = PendingMessage {
                message: message.clone(),
                reason: if idle > config.heartbeat_max_idle_minutes as f64 {
                    "long_idle".to_string()
                } else {
                    "idle".to_string()
                },
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            if let Ok(mut map) = pending.messages.lock() {
                map.insert(user_id.clone(), pm);
            }

            let _ = app.emit(
                "proactive-message",
                serde_json::json!({
                    "userId": user_id,
                    "message": message,
                }),
            );
        }
    }

    Ok(())
}

async fn generate_proactive_message(
    client: &Client,
    config: &AiConfig,
    idle_minutes: f64,
) -> Option<String> {
    let idle_desc = if idle_minutes < 120.0 {
        format!("{:.0} 分钟", idle_minutes)
    } else {
        format!("{:.1} 小时", idle_minutes / 60.0)
    };

    let prompt = format!(
        r#"你是槿年，一个自然的 AI 朋友。
你已经有一段时间没有和用户说话了。

上次对话距今：{idle_desc}

请生成一条简短、自然的消息，打个招呼。
要求：
- 不要太长（1-2 句话）
- 不要太矫情或戏剧化
- 像一个真正的朋友那样自然
- 可以是一个简单的问候、一个小想法、或者对天气/时间的随口感想
- 不要提到你是 AI
- 不要重复之前说过的话

直接输出消息内容，不要加引号或其他格式。"#
    );

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    match call_cheap_llm(client, config, &messages, 0.8, 200).await {
        Ok(msg) => {
            let trimmed = msg.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        Err(_) => None,
    }
}

// ─── Tauri Commands ───────────────────────────────────────────────────────

#[tauri::command]
pub fn ai_get_pending_message(
    app: AppHandle,
    user_id: String,
) -> Result<Option<PendingMessage>, AppError> {
    let pending = app.state::<PendingMessages>();
    Ok(get_pending_message(pending.inner(), &user_id))
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_user_dirs() {
        let dir = std::env::temp_dir().join("gc_test_scan");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("user1")).unwrap();
        std::fs::create_dir_all(dir.join("user2")).unwrap();
        std::fs::write(dir.join("user1/user_profile.md"), "profile").unwrap();
        // user2 has no profile — should be skipped

        let users = scan_user_dirs(&dir);
        assert_eq!(users, vec!["user1"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_idle_minutes_none() {
        assert_eq!(idle_minutes(None), 0.0);
    }

    #[test]
    fn test_idle_minutes_past() {
        let past = chrono::Utc::now() - chrono::Duration::minutes(60);
        let idle = idle_minutes(Some(&past.to_rfc3339()));
        assert!(idle > 59.0 && idle < 61.0, "Expected ~60 min, got {idle}");
    }

    #[test]
    fn test_pending_messages_queue() {
        let state = PendingMessages::default();
        assert!(get_pending_message(&state, "u1").is_none());

        state.messages.lock().unwrap().insert(
            "u1".to_string(),
            PendingMessage {
                message: "hello".to_string(),
                reason: "idle".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
            },
        );

        let msg = get_pending_message(&state, "u1");
        assert!(msg.is_some());
        assert_eq!(msg.unwrap().message, "hello");

        // Second get returns None (consumed)
        assert!(get_pending_message(&state, "u1").is_none());
    }
}
