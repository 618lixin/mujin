use std::path::Path;

use super::config::AiConfig;
use super::notes::AppError;
use super::types::{ChatMessage, CoreMemory, CoreMemoryResponse, CoreMemoryStats, MemoryPatch};

// ─── Core Memory ──────────────────────────────────────────────────────────

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

fn user_dir(base_dir: &Path, user_id: &str) -> Result<std::path::PathBuf, AppError> {
    validate_user_id(user_id)?;
    Ok(base_dir.join(user_id))
}

fn profile_path(base_dir: &Path, user_id: &str) -> Result<std::path::PathBuf, AppError> {
    Ok(user_dir(base_dir, user_id)?.join("user_profile.md"))
}

fn notes_path(base_dir: &Path, user_id: &str) -> Result<std::path::PathBuf, AppError> {
    Ok(user_dir(base_dir, user_id)?.join("companion_notes.md"))
}

pub fn load_core_memory(
    base_dir: &Path,
    user_id: &str,
    config: &AiConfig,
) -> Result<CoreMemory, AppError> {
    let profile_file = profile_path(base_dir, user_id)?;
    let notes_file = notes_path(base_dir, user_id)?;

    let profile_content = if profile_file.exists() {
        std::fs::read_to_string(&profile_file).unwrap_or_default()
    } else {
        String::new()
    };

    let notes_content = if notes_file.exists() {
        std::fs::read_to_string(&notes_file).unwrap_or_default()
    } else {
        String::new()
    };

    Ok(CoreMemory {
        profile_content,
        notes_content,
        profile_max_chars: config.profile_max_chars,
        notes_max_chars: config.notes_max_chars,
    })
}

pub fn save_core_memory(
    base_dir: &Path,
    user_id: &str,
    memory: &CoreMemory,
) -> Result<(), AppError> {
    let dir = user_dir(base_dir, user_id)?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::new("io", format!("Failed to create user dir: {e}")))?;

    std::fs::write(profile_path(base_dir, user_id)?, &memory.profile_content)
        .map_err(|e| AppError::new("io", format!("Failed to write profile: {e}")))?;

    std::fs::write(notes_path(base_dir, user_id)?, &memory.notes_content)
        .map_err(|e| AppError::new("io", format!("Failed to write notes: {e}")))?;

    Ok(())
}

pub fn patch_core_memory(
    base_dir: &Path,
    user_id: &str,
    config: &AiConfig,
    patch: &MemoryPatch,
) -> Result<CoreMemory, AppError> {
    let mut memory = load_core_memory(base_dir, user_id, config)?;

    let current = if patch.target == "profile" {
        &memory.profile_content
    } else {
        &memory.notes_content
    };

    let max_chars = if patch.target == "profile" {
        memory.profile_max_chars
    } else {
        memory.notes_max_chars
    };

    let new_content = match patch.action.as_str() {
        "add" => {
            if current.is_empty() {
                patch.content.clone()
            } else {
                format!("{}\n{}", current.trim(), patch.content)
            }
        }
        "replace" => {
            let old_text = patch.old_text.as_deref().unwrap_or("");
            if old_text.is_empty() || !current.contains(old_text) {
                return Err(AppError::new(
                    "memory",
                    format!("old_text '{}' not found in content", old_text),
                ));
            }
            current.replace(old_text, &patch.content)
        }
        "remove" => {
            let old_text = patch.old_text.as_deref().unwrap_or("");
            if old_text.is_empty() || !current.contains(old_text) {
                return Err(AppError::new(
                    "memory",
                    format!("old_text '{}' not found in content", old_text),
                ));
            }
            current.replace(old_text, "").trim().to_string()
        }
        _ => {
            return Err(AppError::new(
                "memory",
                format!("Unknown action: {}", patch.action),
            ));
        }
    };

    if new_content.len() > max_chars as usize {
        return Err(AppError::new(
            "memory",
            format!(
                "Content exceeds limit: {}/{max_chars} chars",
                new_content.len()
            ),
        ));
    }

    if patch.target == "profile" {
        memory.profile_content = new_content;
    } else {
        memory.notes_content = new_content;
    }

    save_core_memory(base_dir, user_id, &memory)?;
    Ok(memory)
}

pub fn format_memory_for_prompt(memory: &CoreMemory) -> String {
    let profile_pct = (memory.profile_pct() * 100.0) as i32;
    let notes_pct = (memory.notes_pct() * 100.0) as i32;

    let mut blocks = Vec::new();

    if !memory.profile_content.is_empty() {
        blocks.push(format!(
            "{}\nUSER PROFILE (用户画像) [{}% — {}/{} chars]\n{}\n{}",
            "=".repeat(48),
            profile_pct,
            memory.profile_usage(),
            memory.profile_max_chars,
            memory.profile_content,
            "=".repeat(48),
        ));
    }

    if !memory.notes_content.is_empty() {
        blocks.push(format!(
            "{}\nCOMPANION NOTES (AI 笔记) [{}% — {}/{} chars]\n{}\n{}",
            "=".repeat(48),
            notes_pct,
            memory.notes_usage(),
            memory.notes_max_chars,
            memory.notes_content,
            "=".repeat(48),
        ));
    }

    blocks.join("\n\n")
}

pub fn build_core_memory_response(memory: &CoreMemory) -> CoreMemoryResponse {
    CoreMemoryResponse {
        profile: CoreMemoryStats {
            content: memory.profile_content.clone(),
            chars: memory.profile_usage(),
            max_chars: memory.profile_max_chars,
            pct: memory.profile_pct(),
            near_limit: memory.profile_near_limit(),
        },
        notes: CoreMemoryStats {
            content: memory.notes_content.clone(),
            chars: memory.notes_usage(),
            max_chars: memory.notes_max_chars,
            pct: memory.notes_pct(),
            near_limit: memory.notes_near_limit(),
        },
    }
}

// ─── Chat History ─────────────────────────────────────────────────────────

fn history_path(base_dir: &Path, user_id: &str) -> Result<std::path::PathBuf, AppError> {
    Ok(user_dir(base_dir, user_id)?.join("history.json"))
}

pub fn load_history(
    base_dir: &Path,
    user_id: &str,
    max_turns: u32,
) -> Result<Vec<ChatMessage>, AppError> {
    let path = history_path(base_dir, user_id)?;
    if !path.exists() {
        return Ok(vec![]);
    }

    let data = std::fs::read_to_string(&path)
        .map_err(|e| AppError::new("io", format!("Failed to read history: {e}")))?;

    let all: Vec<ChatMessage> = serde_json::from_str(&data).unwrap_or_default();
    let max_msgs = (max_turns as usize) * 2;
    if all.len() > max_msgs {
        Ok(all[all.len() - max_msgs..].to_vec())
    } else {
        Ok(all)
    }
}

pub fn save_history(
    base_dir: &Path,
    user_id: &str,
    messages: &[ChatMessage],
    max_turns: u32,
) -> Result<(), AppError> {
    let dir = user_dir(base_dir, user_id)?;
    std::fs::create_dir_all(&dir).ok();

    let max_msgs = (max_turns as usize) * 2;
    let to_save = if messages.len() > max_msgs {
        &messages[messages.len() - max_msgs..]
    } else {
        messages
    };

    let path = history_path(base_dir, user_id)?;
    let json = serde_json::to_string_pretty(to_save)
        .map_err(|e| AppError::new("parse", format!("Failed to serialize history: {e}")))?;
    std::fs::write(path, json)
        .map_err(|e| AppError::new("io", format!("Failed to write history: {e}")))?;

    Ok(())
}

// ─── Turn Counter ─────────────────────────────────────────────────────────

fn turn_counter_path(base_dir: &Path, user_id: &str) -> Result<std::path::PathBuf, AppError> {
    Ok(user_dir(base_dir, user_id)?.join("turn_counter.json"))
}

pub fn load_turn_counter(base_dir: &Path, user_id: &str) -> Result<u32, AppError> {
    let path = turn_counter_path(base_dir, user_id)?;
    if !path.exists() {
        return Ok(0);
    }
    let data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_default())
            .unwrap_or_default();
    Ok(data.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as u32)
}

pub fn save_turn_counter(base_dir: &Path, user_id: &str, count: u32) -> Result<(), AppError> {
    let dir = user_dir(base_dir, user_id)?;
    std::fs::create_dir_all(&dir).ok();
    let path = turn_counter_path(base_dir, user_id)?;
    let json = serde_json::json!({ "count": count }).to_string();
    std::fs::write(path, json)
        .map_err(|e| AppError::new("io", format!("Failed to write turn counter: {e}")))?;
    Ok(())
}

// ─── Last Activity ────────────────────────────────────────────────────────

fn last_activity_path(base_dir: &Path, user_id: &str) -> Result<std::path::PathBuf, AppError> {
    Ok(user_dir(base_dir, user_id)?.join("last_activity.json"))
}

pub fn save_last_activity(base_dir: &Path, user_id: &str) -> Result<(), AppError> {
    let dir = user_dir(base_dir, user_id)?;
    std::fs::create_dir_all(&dir).ok();
    let path = last_activity_path(base_dir, user_id)?;
    let now = chrono::Utc::now().to_rfc3339();
    let json = serde_json::json!({ "last_active_at": now }).to_string();
    std::fs::write(path, json)
        .map_err(|e| AppError::new("io", format!("Failed to write last activity: {e}")))?;
    Ok(())
}

pub fn load_last_activity(base_dir: &Path, user_id: &str) -> Result<Option<String>, AppError> {
    let path = last_activity_path(base_dir, user_id)?;
    if !path.exists() {
        return Ok(None);
    }
    let data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_default())
            .unwrap_or_default();
    Ok(data
        .get("last_active_at")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()))
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("gc_test_memory_{id}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_load_save_core_memory() {
        let dir = test_dir();
        let config = AiConfig::default();

        let memory = CoreMemory {
            profile_content: "测试用户画像".to_string(),
            notes_content: "AI 笔记内容".to_string(),
            ..CoreMemory::default()
        };

        save_core_memory(&dir, "user1", &memory).unwrap();
        let loaded = load_core_memory(&dir, "user1", &config).unwrap();
        assert_eq!(loaded.profile_content, "测试用户画像");
        assert_eq!(loaded.notes_content, "AI 笔记内容");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rejects_path_traversal_user_id() {
        let dir = test_dir();
        let config = AiConfig::default();

        let err = load_core_memory(&dir, "../outside", &config).unwrap_err();
        assert_eq!(err.code, "invalidUserId");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_patch_add() {
        let dir = test_dir();
        let config = AiConfig::default();

        let memory = CoreMemory {
            profile_content: String::new(),
            notes_content: String::new(),
            ..CoreMemory::default()
        };
        save_core_memory(&dir, "user2", &memory).unwrap();

        let patch = MemoryPatch {
            action: "add".to_string(),
            target: "profile".to_string(),
            content: "用户喜欢编程".to_string(),
            old_text: None,
        };
        let result = patch_core_memory(&dir, "user2", &config, &patch).unwrap();
        assert_eq!(result.profile_content, "用户喜欢编程");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_patch_replace() {
        let dir = test_dir();
        let config = AiConfig::default();

        let memory = CoreMemory {
            profile_content: "用户喜欢 Python".to_string(),
            notes_content: String::new(),
            ..CoreMemory::default()
        };
        save_core_memory(&dir, "user3", &memory).unwrap();

        let patch = MemoryPatch {
            action: "replace".to_string(),
            target: "profile".to_string(),
            content: "Rust".to_string(),
            old_text: Some("Python".to_string()),
        };
        let result = patch_core_memory(&dir, "user3", &config, &patch).unwrap();
        assert_eq!(result.profile_content, "用户喜欢 Rust");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_patch_capacity_exceeded() {
        let dir = test_dir();
        let config = AiConfig {
            profile_max_chars: 10,
            ..AiConfig::default()
        };

        let memory = CoreMemory {
            profile_content: "short".to_string(),
            notes_content: String::new(),
            profile_max_chars: 10,
            notes_max_chars: 800,
        };
        save_core_memory(&dir, "user4", &memory).unwrap();

        let patch = MemoryPatch {
            action: "add".to_string(),
            target: "profile".to_string(),
            content: "this is way too long content".to_string(),
            old_text: None,
        };
        let result = patch_core_memory(&dir, "user1", &config, &patch);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("exceeds limit"));
    }

    #[test]
    fn test_history_trim() {
        let dir = test_dir();

        let messages: Vec<ChatMessage> = (0..50)
            .flat_map(|i| {
                vec![
                    ChatMessage {
                        role: "user".to_string(),
                        content: format!("msg {i}"),
                    },
                    ChatMessage {
                        role: "assistant".to_string(),
                        content: format!("reply {i}"),
                    },
                ]
            })
            .collect();

        save_history(&dir, "user1", &messages, 20).unwrap();
        let loaded = load_history(&dir, "user1", 5).unwrap(); // only last 5 turns = 10 messages
        assert_eq!(loaded.len(), 10);
    }

    #[test]
    fn test_turn_counter() {
        let dir = test_dir();
        assert_eq!(load_turn_counter(&dir, "user1").unwrap(), 0);
        save_turn_counter(&dir, "user1", 42).unwrap();
        assert_eq!(load_turn_counter(&dir, "user1").unwrap(), 42);
    }

    #[test]
    fn test_format_memory_for_prompt() {
        let memory = CoreMemory {
            profile_content: "测试画像".to_string(),
            notes_content: "测试笔记".to_string(),
            ..CoreMemory::default()
        };
        let prompt = format_memory_for_prompt(&memory);
        assert!(prompt.contains("USER PROFILE"));
        assert!(prompt.contains("COMPANION NOTES"));
        assert!(prompt.contains("测试画像"));
        assert!(prompt.contains("测试笔记"));
    }

    #[test]
    fn test_format_empty_memory() {
        let memory = CoreMemory {
            profile_content: String::new(),
            notes_content: String::new(),
            ..CoreMemory::default()
        };
        let prompt = format_memory_for_prompt(&memory);
        assert!(prompt.is_empty());
    }
}
