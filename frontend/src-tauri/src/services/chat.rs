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
use super::types::{ChatMessage, Event, PostChatResult, QueryEventsParams};

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

// ─── Memory Retrieval ─────────────────────────────────────────────────────

/// Escape user input for safe FTS5 query usage.
/// Strips FTS5 special characters. FTS5 treats consecutive CJK characters
/// as an implicit phrase — no need for explicit quote wrapping.
fn escape_fts5_query(input: &str) -> String {
    input
        .chars()
        .filter(|c| !matches!(c, '"' | '*' | '(' | ')' | '-' | '^'))
        .take(120)
        .collect()
}

/// Select an emoji for the event type badge.
fn event_type_emoji(event_type: &Option<String>) -> &'static str {
    match event_type.as_deref() {
        Some("milestone") => "🏔",
        Some("conflict") => "⚡",
        Some("decision") => "🎯",
        Some("emotion") => "💭",
        _ => "📌",
    }
}

/// Format retrieved memories into a compact prompt block.
fn format_retrieved_memories(
    fts5_turns: &[super::types::ConversationTurn],
    related_events: &[Event],
) -> Option<String> {
    if fts5_turns.is_empty() && related_events.is_empty() {
        return None;
    }

    let mut block = String::from("--- 相关历史记忆 ---\n");

    if !related_events.is_empty() {
        block.push_str("过往相关事件：\n");
        for event in related_events {
            let date: String = event.created_at.chars().take(10).collect();
            let emoji = event_type_emoji(&event.event_type);
            let emotions_str = if event.emotions.is_empty() {
                String::new()
            } else {
                format!(" [{}]", event.emotions.join(", "))
            };
            block.push_str(&format!(
                "- [{}] {} {}{}\n",
                date, emoji, event.content, emotions_str
            ));
        }
        block.push('\n');
    }

    if !fts5_turns.is_empty() {
        block.push_str("相关过往对话：\n");
        for turn in fts5_turns {
            let date: String = turn.created_at.chars().take(10).collect();
            let summary = truncate_for_display(&turn.summary, 150);
            block.push_str(&format!("- [{}] {}\n", date, summary));
        }
    }

    block.push_str("---");
    Some(block)
}

fn truncate_for_display(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(max_chars).collect::<String>())
    }
}

/// Retrieve memories relevant to the user's current message.
///
/// Uses two sources:
/// 1. LIKE-based keyword search on conversation_turns (works with Chinese,
///    unlike FTS5's unicode61 tokenizer which treats CJK characters as one blob)
/// 2. Recent high-importance events (≤ 30 days, importance ≥ 0.5, limit 5)
///
/// Returns `None` when no relevant memories are found.
fn retrieve_memories(
    db: &DbState,
    user_id: &str,
    user_message: &str,
) -> Result<Option<String>, AppError> {
    // Extract meaningful keywords: take up to first 20 chars for LIKE search.
    // For CJK text, 2-char substrings give good recall.
    let keyword: String = user_message.chars().take(20).collect();

    // ── LIKE search on conversation history (FTS5 doesn't handle Chinese well) ──
    let like_turns = db
        .search_conversations_like(user_id, &keyword, 5)
        .unwrap_or_default();

    // ── Recent important events ──
    let now = chrono::Utc::now();
    let thirty_days_ago = now - chrono::Duration::days(30);
    let related_events = db
        .query_events(
            user_id,
            &QueryEventsParams {
                limit: 5,
                min_importance: 0.5,
                start_date: Some(thirty_days_ago.to_rfc3339()),
                end_date: Some(now.to_rfc3339()),
                ..Default::default()
            },
        )
        .unwrap_or_default();

    Ok(format_retrieved_memories(&like_turns, &related_events))
}

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
    db: &DbState,
    user_id: &str,
    user_message: &str,
    config: &AiConfig,
) -> Result<(Vec<ChatMessage>, Vec<ChatMessage>), AppError> {
    let retrieved_memories = retrieve_memories(db, user_id, user_message)?;

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
    let (messages, history) = prepare_chat(base_dir, db, user_id, user_message, config).await?;

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

    // Create DbState for memory retrieval + background task
    let db_for_task = DbState::new(base_dir.clone());

    // Prepare messages (includes memory retrieval via FTS5 + events)
    let (messages, history) =
        prepare_chat(&base_dir, &db_for_task, &user_id, &message, &config).await?;

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

    // ── Memory retrieval unit tests ──────────────────────────────────────

    #[test]
    fn test_escape_fts5_query_removes_special_chars() {
        let input = "今天\"面试\"怎么样* (还不错) - 开心^";
        let escaped = escape_fts5_query(input);
        // Special FTS5 chars should be stripped
        assert!(!escaped.contains('"'));
        assert!(!escaped.contains('*'));
        assert!(!escaped.contains('('));
        assert!(!escaped.contains(')'));
        assert!(!escaped.contains('-'));
        assert!(!escaped.contains('^'));
    }

    #[test]
    fn test_escape_fts5_query_truncates_long_input() {
        let input: String = std::iter::repeat("你好").take(100).collect::<String>();
        let escaped = escape_fts5_query(&input);
        // Should be truncated to 120 chars, plus 2 quote chars
        assert!(
            escaped.chars().count() <= 122,
            "got {} chars",
            escaped.chars().count()
        );
    }

    #[test]
    fn test_event_type_emoji_variants() {
        assert_eq!(event_type_emoji(&Some("milestone".into())), "🏔");
        assert_eq!(event_type_emoji(&Some("conflict".into())), "⚡");
        assert_eq!(event_type_emoji(&Some("decision".into())), "🎯");
        assert_eq!(event_type_emoji(&Some("emotion".into())), "💭");
        assert_eq!(event_type_emoji(&None), "📌");
        assert_eq!(event_type_emoji(&Some("unknown".into())), "📌");
    }

    #[test]
    fn test_truncate_for_display_short_text() {
        assert_eq!(truncate_for_display("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_for_display_long_text() {
        let long = "你好".repeat(80); // 160 CJK chars
        let result = truncate_for_display(&long, 150);
        assert!(result.chars().count() <= 153); // 150 + "…"
        assert!(result.ends_with('…'));
    }

    #[test]
    fn test_format_retrieved_memories_empty() {
        let result = format_retrieved_memories(&[], &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_format_retrieved_memories_with_events_and_turns() {
        use super::super::types::{ConversationTurn, Event};

        let events = vec![Event {
            id: "evt1".into(),
            content: "用户参加了面试".into(),
            emotions: vec!["anxiety".into(), "hope".into()],
            importance: 0.8,
            event_type: Some("milestone".into()),
            strength: 1.0,
            stability: 30.0,
            recall_count: 0,
            last_recalled_at: None,
            created_at: "2026-05-25T10:00:00Z".into(),
            updated_at: "2026-05-25T10:00:00Z".into(),
        }];

        let turns = vec![ConversationTurn {
            id: 1,
            summary: "用户聊到找工作的事情".into(),
            emotions: vec!["anxiety".into()],
            created_at: "2026-05-20T08:00:00Z".into(),
        }];

        let result = format_retrieved_memories(&turns, &events);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text.contains("相关历史记忆"));
        assert!(text.contains("过往相关事件"));
        assert!(text.contains("参加了面试"));
        assert!(text.contains("🏔"));
        assert!(text.contains("[anxiety, hope]"));
        assert!(text.contains("相关过往对话"));
        assert!(text.contains("找工作"));
    }

    #[test]
    fn test_format_retrieved_memories_only_events() {
        use super::super::types::Event;

        let events = vec![Event {
            id: "evt1".into(),
            content: "测试事件".into(),
            emotions: vec![],
            importance: 0.6,
            event_type: None,
            strength: 1.0,
            stability: 30.0,
            recall_count: 0,
            last_recalled_at: None,
            created_at: "2026-05-15T00:00:00Z".into(),
            updated_at: "2026-05-15T00:00:00Z".into(),
        }];

        let result = format_retrieved_memories(&[], &events);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text.contains("过往相关事件"));
        assert!(!text.contains("相关过往对话"));
        assert!(text.contains("📌")); // no event_type → default emoji
    }

    #[test]
    fn test_format_retrieved_memories_only_turns() {
        use super::super::types::ConversationTurn;

        let turns = vec![ConversationTurn {
            id: 2,
            summary: "一段很长的摘要".into(),
            emotions: vec![],
            created_at: "2026-05-18T12:00:00Z".into(),
        }];

        let result = format_retrieved_memories(&turns, &[]);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(!text.contains("过往相关事件"));
        assert!(text.contains("相关过往对话"));
    }

    #[test]
    fn test_retrieve_memories_integration() {
        // Setup temp DB
        let dir = std::env::temp_dir().join("gc_test_chat_retrieve2");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("user1")).unwrap();
        std::fs::write(dir.join("user1/user_profile.md"), "").unwrap();

        let db = DbState::new(dir.clone());
        db.init_db("user1").unwrap();

        // Save conversation turns via DbState (uses LIKE search, not FTS5)
        db.save_conversation_turn(
            "user1",
            "我今天去面试了",
            "面试怎么样？",
            Some("用户参加面试"),
            &["anxiety".to_string()],
        )
        .unwrap();

        db.save_conversation_turn(
            "user1",
            "最近工作压力很大",
            "能具体说说吗",
            Some("工作压力讨论"),
            &["anxiety".to_string()],
        )
        .unwrap();

        // ── Test 1: LIKE search finds Chinese keyword ──
        let like_results = db
            .search_conversations_like("user1", "工作压力", 5)
            .unwrap();
        assert!(
            !like_results.is_empty(),
            "LIKE should find Chinese keyword '工作压力'"
        );

        // ── Test 2: retrieve_memories high-level function ──
        let result = retrieve_memories(&db, "user1", "工作压力").unwrap();
        assert!(
            result.is_some(),
            "retrieve_memories should find memories about work"
        );
        let text = result.unwrap();
        assert!(text.contains("相关历史记忆"), "got: {}", text);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
