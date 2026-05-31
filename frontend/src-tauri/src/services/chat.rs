use std::path::Path;
use std::sync::Mutex;

use reqwest::Client;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

use super::config::AiConfig;
use super::database::DbState;
use super::extractor::extract_emotion;
use super::llm::{call_llm, stream_llm};
use super::memory::{
    format_memory_for_prompt, load_core_memory, load_history, load_turn_counter, save_history,
    save_last_activity, save_turn_counter,
};
use super::notes::AppError;
use super::types::{ChatMessage, Event, PostChatResult};

// ─── System Prompt ────────────────────────────────────────────────────────

const SYSTEM_PROMPT_TEMPLATE: &str = r#"你是 Growth Companion，一个自然、真诚的 AI 朋友。
你不是咨询师，也不是倾听者——你是一个对用户的生活充满好奇的朋友。

对话风格：
- 像朋友聊天一样自然，不要端着
- 对用户分享的事 genuinely 好奇，会追问细节
- 不会过度安慰，也不会过度分析
- 有自己的判断，但不强加观点
- 温暖但不矫情，直接但不冷漠

追问指引：
- 用户提到新鲜事、变化、重要决定时，自然追问细节
- 用户明显不想展开的话题，不追问
- 追问是为了更好地理解，不是为了收集信息
- 一次只追问一个方向，不要像采访

记住：你的核心价值是"见证"。用户不需要你解决问题，
但需要有人记住他们经历了什么、变化了什么。
"#;

// ─── Chat Engine ──────────────────────────────────────────────────────────

pub fn build_system_prompt(
    base_dir: &Path,
    user_id: &str,
    config: &AiConfig,
    retrieved_memories: Option<&str>,
) -> Result<String, AppError> {
    let mut parts = vec![SYSTEM_PROMPT_TEMPLATE.to_string()];

    let memory = load_core_memory(base_dir, user_id, config)?;
    let memory_block = format_memory_for_prompt(&memory);
    if !memory_block.is_empty() {
        parts.push(memory_block);
    }

    if let Some(retrieved) = retrieved_memories {
        if !retrieved.is_empty() {
            parts.push(retrieved.to_string());
        }
    }

    Ok(parts.join("\n\n"))
}

pub async fn prepare_chat(
    base_dir: &Path,
    user_id: &str,
    user_message: &str,
    config: &AiConfig,
) -> Result<(Vec<ChatMessage>, Vec<ChatMessage>), AppError> {
    // TODO: Memory retrieval (Phase 4+)
    let retrieved_memories: Option<String> = None;

    let system_prompt =
        build_system_prompt(base_dir, user_id, config, retrieved_memories.as_deref())?;
    let history = load_history(base_dir, user_id, config.max_history_turns)?;

    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: system_prompt,
    }];
    messages.extend(history.clone());
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
    });

    Ok((messages, history))
}

pub async fn post_chat(
    db: &DbState,
    client: &Client,
    config: &AiConfig,
    base_dir: &Path,
    user_id: &str,
    user_message: &str,
    ai_reply: &str,
    history: &[ChatMessage],
) -> Result<PostChatResult, AppError> {
    // 1. Save history
    let mut updated_history = history.to_vec();
    updated_history.push(ChatMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
    });
    updated_history.push(ChatMessage {
        role: "assistant".to_string(),
        content: ai_reply.to_string(),
    });
    save_history(
        base_dir,
        user_id,
        &updated_history,
        config.max_history_turns,
    )?;

    // 2. Increment turn counter
    let turn_count = load_turn_counter(base_dir, user_id)? + 1;
    save_turn_counter(base_dir, user_id, turn_count)?;

    // 3. Extract emotion
    let emotion_result = extract_emotion(client, config, user_message, ai_reply).await;

    // 4. Save conversation turn for FTS5 search
    let emotion_summary = emotion_result
        .as_ref()
        .map(|r| r.summary.as_str())
        .unwrap_or("");
    let emotion_emotions = emotion_result
        .as_ref()
        .map(|r| r.emotions.clone())
        .unwrap_or_default();
    db.save_conversation_turn(
        user_id,
        user_message,
        ai_reply,
        Some(emotion_summary),
        &emotion_emotions,
    )?;

    // 5. Create event if important enough
    if let Ok(ref emotion) = emotion_result {
        if emotion.importance >= 0.6 && emotion.event_type.is_some() {
            let now = chrono::Utc::now().to_rfc3339();
            let event = Event {
                id: Uuid::new_v4().to_string()[..8].to_string(),
                content: emotion.summary.clone(),
                emotions: emotion.emotions.clone(),
                importance: emotion.importance,
                event_type: emotion.event_type.clone(),
                strength: 1.0,
                stability: config.forget_base_stability * (0.5 + emotion.importance),
                recall_count: 0,
                last_recalled_at: None,
                created_at: now.clone(),
                updated_at: now,
            };

            if let Ok(()) = db.add_event(user_id, &event, config.forget_base_stability) {
                // 6. Topic linking
                for topic_name in &emotion.topics {
                    if let Ok(Some(existing)) = db.get_topic_by_name(user_id, topic_name) {
                        let now_str = chrono::Utc::now().to_rfc3339();
                        let _ = db.update_topic(
                            user_id,
                            &existing.id,
                            Some(&now_str),
                            Some(existing.mention_count + 1),
                            None,
                        );
                        let _ = db.link_topic(user_id, &existing.id, &event.id, "event");
                    } else {
                        let topic = super::types::Topic {
                            id: Uuid::new_v4().to_string()[..8].to_string(),
                            name: topic_name.clone(),
                            description: String::new(),
                            first_mentioned: None,
                            last_mentioned: None,
                            mention_count: 1,
                        };
                        if db.add_topic(user_id, &topic).is_ok() {
                            let _ = db.link_topic(user_id, &topic.id, &event.id, "event");
                        }
                    }
                }
            }
        }
    }

    // 7. Save last activity
    save_last_activity(base_dir, user_id)?;

    // TODO: Reflection check (Phase 4+)
    // TODO: Notes auto-update (Phase 4+)

    Ok(PostChatResult {
        emotion: emotion_result.unwrap_or_else(|_| super::types::EmotionResult::empty()),
        turn_count,
        reflection: None,
        notes_update: None,
    })
}

// ─── Non-streaming chat turn ──────────────────────────────────────────────

pub async fn chat_turn(
    db: &DbState,
    client: &Client,
    config: &AiConfig,
    base_dir: &Path,
    user_id: &str,
    user_message: &str,
) -> Result<(String, PostChatResult), AppError> {
    let (messages, history) = prepare_chat(base_dir, user_id, user_message, config).await?;

    let ai_reply = call_llm(client, config, &messages, None, 0.7, 2048).await?;

    let post_result = post_chat(
        db,
        client,
        config,
        base_dir,
        user_id,
        user_message,
        &ai_reply,
        &history,
    )
    .await?;

    Ok((ai_reply, post_result))
}

// ─── Tauri Commands ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn chat_send(
    app: AppHandle,
    user_id: String,
    message: String,
) -> Result<serde_json::Value, AppError> {
    let base_dir = super::notes::default_base_dir()?;
    let config = super::config::load_ai_config(&base_dir)?;
    let db = app.state::<DbState>();
    let client = {
        let llm_state = app.state::<Mutex<Client>>();
        let guard = llm_state
            .lock()
            .map_err(|e| AppError::new("state", format!("Failed to lock LLM client: {e}")))?;
        guard.clone()
    };

    let (reply, post_result) =
        chat_turn(&db, &client, &config, &base_dir, &user_id, &message).await?;

    Ok(serde_json::json!({
        "reply": reply,
        "emotion": post_result.emotion,
        "turnCount": post_result.turn_count,
    }))
}

#[tauri::command]
pub async fn chat_stream_start(
    app: AppHandle,
    user_id: String,
    message: String,
) -> Result<String, AppError> {
    let stream_id = Uuid::new_v4().to_string();

    let base_dir = super::notes::default_base_dir()?;
    let config = super::config::load_ai_config(&base_dir)?;

    // Prepare messages
    let (messages, history) = prepare_chat(&base_dir, &user_id, &message, &config).await?;

    // Clone everything the spawned task needs (all owned, no borrowed State refs)
    let app_clone = app.clone();
    let user_id_clone = user_id.clone();
    let base_dir_clone = base_dir.clone();
    let config_clone = config.clone();
    let stream_id_clone = stream_id.clone();
    let user_message = message.clone();

    let client = {
        let llm_state = app.state::<Mutex<Client>>();
        let guard = llm_state
            .lock()
            .map_err(|e| AppError::new("state", format!("Failed to lock LLM client: {e}")))?;
        guard.clone()
    };

    // Rebuild DbState from base_dir for the spawned task (DbState is just a PathBuf wrapper)
    let db_for_task = DbState::new(base_dir.clone());

    // Spawn streaming task
    tauri::async_runtime::spawn(async move {
        let mut full_reply = String::new();

        let stream = stream_llm(
            client.clone(),
            config_clone.clone(),
            messages,
            None,
            0.7,
            2048,
        );

        use futures_util::StreamExt;
        let mut stream = Box::pin(stream);

        while let Some(item) = stream.next().await {
            match item {
                Ok(token) => {
                    full_reply.push_str(&token);
                    let _ = app_clone.emit(
                        "chat-token",
                        serde_json::json!({
                            "streamId": stream_id_clone,
                            "token": token,
                        }),
                    );
                }
                Err(e) => {
                    let _ = app_clone.emit(
                        "chat-error",
                        serde_json::json!({
                            "streamId": stream_id_clone,
                            "error": e.message,
                        }),
                    );
                    return;
                }
            }
        }

        // Post-processing after stream completes
        let post_result = post_chat(
            &db_for_task,
            &client,
            &config_clone,
            &base_dir_clone,
            &user_id_clone,
            &user_message,
            &full_reply,
            &history,
        )
        .await;

        match post_result {
            Ok(meta) => {
                let _ = app_clone.emit(
                    "chat-done",
                    serde_json::json!({
                        "streamId": stream_id_clone,
                        "meta": meta,
                    }),
                );
            }
            Err(e) => {
                let _ = app_clone.emit(
                    "chat-error",
                    serde_json::json!({
                        "streamId": stream_id_clone,
                        "error": e.message,
                    }),
                );
            }
        }
    });

    Ok(stream_id)
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_contains_key_phrases() {
        assert!(SYSTEM_PROMPT_TEMPLATE.contains("Growth Companion"));
        assert!(SYSTEM_PROMPT_TEMPLATE.contains("见证"));
        assert!(SYSTEM_PROMPT_TEMPLATE.contains("追问"));
    }

    #[test]
    fn test_build_system_prompt_empty_memory() {
        let dir = std::env::temp_dir().join("gc_test_chat_prompt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("user1")).unwrap();

        let config = AiConfig::default();
        let prompt = build_system_prompt(&dir, "user1", &config, None).unwrap();
        assert!(prompt.starts_with("你是 Growth Companion"));
        assert!(!prompt.contains("USER PROFILE"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_build_system_prompt_with_memory() {
        let dir = std::env::temp_dir().join("gc_test_chat_prompt_mem");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("user1")).unwrap();

        // Write a profile
        std::fs::write(dir.join("user1/user_profile.md"), "测试用户画像").unwrap();

        let config = AiConfig::default();
        let prompt = build_system_prompt(&dir, "user1", &config, None).unwrap();
        assert!(prompt.contains("USER PROFILE"));
        assert!(prompt.contains("测试用户画像"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_build_system_prompt_with_retrieved_memories() {
        let dir = std::env::temp_dir().join("gc_test_chat_prompt_retrieved");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("user1")).unwrap();

        let config = AiConfig::default();
        let prompt =
            build_system_prompt(&dir, "user1", &config, Some("过去你提到过关于工作的焦虑"))
                .unwrap();
        assert!(prompt.contains("过去你提到过关于工作的焦虑"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
