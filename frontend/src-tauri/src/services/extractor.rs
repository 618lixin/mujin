use reqwest::Client;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use super::config::AiConfig;
use super::database::DbState;
use super::llm::call_cheap_llm;
use super::notes::AppError;
use super::types::{ChatMessage, EmotionResult, Event, Topic};

// ─── Constants ────────────────────────────────────────────────────────────

pub const VALID_EMOTIONS: &[&str] = &[
    "joy",
    "sadness",
    "anger",
    "anxiety",
    "fear",
    "surprise",
    "disgust",
    "calm",
    "overwhelm",
    "hope",
];

pub const VALID_EVENT_TYPES: &[&str] = &["conflict", "milestone", "emotion", "decision"];

// ─── Extraction Prompt ────────────────────────────────────────────────────

const EMOTION_EXTRACTION_PROMPT: &str = r#"分析以下对话，提取情绪和事件信息。

用户消息：{user_message}
AI 回复：{ai_reply}

请以 JSON 格式输出（不要输出其他内容）：
{{
  "emotions": ["从 {valid_emotions} 中选择，可多个"],
  "event_type": "conflict/milestone/emotion/decision 或 null",
  "importance": 0.0到1.0之间的浮点数,
  "summary": "一句话摘要",
  "topics": ["从对话中识别的主题标签，如'职业选择'、'和父亲的关系'、'考研'、'健身'等，最多3个，日常闲聊为空数组"]
}}

评分标准：
- importance 0.0~0.3：日常闲聊
- importance 0.3~0.6：有情绪但非重大事件
- importance 0.6~0.8：明确的事件或强烈情绪
- importance 0.8~1.0：人生重大变化或情绪崩溃

主题识别规则：
- 只提取对话中明确涉及的主题，不要推测
- 主题应该是用户生活中持续出现的关注点或经历
- 如果用户换了工作，主题可以是"职业变化"或具体公司名
"#;

// ─── Public API ───────────────────────────────────────────────────────────

pub async fn extract_emotion(
    client: &Client,
    config: &AiConfig,
    user_message: &str,
    ai_reply: &str,
) -> Result<EmotionResult, AppError> {
    let valid_emotions_str = VALID_EMOTIONS.join("/");
    let prompt = EMOTION_EXTRACTION_PROMPT
        .replace("{user_message}", user_message)
        .replace("{ai_reply}", ai_reply)
        .replace("{valid_emotions}", &valid_emotions_str);

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let response = match call_cheap_llm(client, config, &messages, 0.1, 256).await {
        Ok(r) => r,
        Err(_) => return Ok(EmotionResult::empty()),
    };

    Ok(parse_emotion_response(&response))
}

#[tauri::command]
pub async fn quick_extract(
    app: AppHandle,
    user_id: String,
    note_id: String,
    title: String,
    content: String,
) -> Result<serde_json::Value, AppError> {
    let text = content.trim();
    if text.is_empty() {
        return Ok(serde_json::json!({
            "extracted": false,
            "reason": "empty",
        }));
    }

    let base_dir = super::notes::default_base_dir()?;
    let config = super::config::load_ai_config(&base_dir)?;
    let client = {
        let llm_state = app.state::<Mutex<Client>>();
        let guard = llm_state
            .lock()
            .map_err(|e| AppError::new("state", format!("Failed to lock LLM client: {e}")))?;
        guard.clone()
    };

    let prompt_text = format_quick_note_for_extraction(&title, text);
    let emotion = extract_emotion(&client, &config, &prompt_text, "").await?;
    if !should_persist_quick_event(&emotion) {
        return Ok(serde_json::json!({
            "extracted": false,
            "reason": "below_threshold",
            "emotion": emotion,
        }));
    }

    let db = app.state::<DbState>();
    let now = chrono::Utc::now().to_rfc3339();
    let event_id = Uuid::new_v4().to_string()[..8].to_string();
    let event = Event {
        id: event_id.clone(),
        content: quick_event_summary(&emotion, &title, text),
        emotions: emotion.emotions.clone(),
        importance: emotion.importance,
        event_type: emotion.event_type.clone(),
        strength: 1.0,
        stability: config.forget_base_stability * (0.5 + emotion.importance),
        recall_count: 0,
        last_recalled_at: None,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    db.add_event(&user_id, &event, config.forget_base_stability)?;
    link_quick_note_topics(&db, &user_id, &event_id, &emotion.topics)?;

    Ok(serde_json::json!({
        "extracted": true,
        "eventId": event_id,
        "noteId": note_id,
        "emotion": emotion,
    }))
}

// ─── Parsing ──────────────────────────────────────────────────────────────

fn parse_emotion_response(response: &str) -> EmotionResult {
    let text = strip_codeblock(response.trim());

    let data: serde_json::Value = match serde_json::from_str(&text) {
        Ok(d) => d,
        Err(_) => return EmotionResult::empty(),
    };

    let emotions = data
        .get("emotions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .filter(|e| VALID_EMOTIONS.contains(&e.as_str()))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let event_type = data
        .get("event_type")
        .and_then(|v| v.as_str())
        .filter(|s| VALID_EVENT_TYPES.contains(&(*s).to_lowercase().as_str()))
        .map(String::from);

    let importance = data
        .get("importance")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);

    let summary = data
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let topics = data
        .get("topics")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    EmotionResult {
        emotions,
        event_type,
        importance,
        summary,
        topics,
    }
}

fn should_persist_quick_event(emotion: &EmotionResult) -> bool {
    emotion.importance >= 0.6 && emotion.event_type.is_some() && !emotion.summary.trim().is_empty()
}

fn format_quick_note_for_extraction(title: &str, content: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        content.to_string()
    } else {
        format!("{title}\n\n{content}")
    }
}

fn quick_event_summary(emotion: &EmotionResult, title: &str, content: &str) -> String {
    let summary = emotion.summary.trim();
    if !summary.is_empty() {
        return summary.to_string();
    }

    let fallback = if title.trim().is_empty() {
        content.trim()
    } else {
        title.trim()
    };
    fallback.chars().take(160).collect()
}

fn link_quick_note_topics(
    db: &DbState,
    user_id: &str,
    event_id: &str,
    topics: &[String],
) -> Result<(), AppError> {
    for topic_name in topics {
        let topic_name = topic_name.trim();
        if topic_name.is_empty() {
            continue;
        }

        if let Some(existing) = db.get_topic_by_name(user_id, topic_name)? {
            let now = chrono::Utc::now().to_rfc3339();
            db.update_topic(
                user_id,
                &existing.id,
                Some(&now),
                Some(existing.mention_count + 1),
                None,
            )?;
            db.link_topic(user_id, &existing.id, event_id, "event")?;
        } else {
            let topic = Topic {
                id: Uuid::new_v4().to_string()[..8].to_string(),
                name: topic_name.to_string(),
                description: String::new(),
                first_mentioned: None,
                last_mentioned: None,
                mention_count: 1,
            };
            db.add_topic(user_id, &topic)?;
            db.link_topic(user_id, &topic.id, event_id, "event")?;
        }
    }

    Ok(())
}

/// Strip markdown codeblock wrapping from response.
fn strip_codeblock(text: &str) -> String {
    if text.starts_with("```") {
        let without_first_line = text.splitn(2, '\n').nth(1).unwrap_or(text);
        if let Some(end) = without_first_line.rfind("```") {
            without_first_line[..end].trim().to_string()
        } else {
            without_first_line.trim().to_string()
        }
    } else {
        text.to_string()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_response() {
        let json = r#"{"emotions": ["joy", "hope"], "event_type": "milestone", "importance": 0.8, "summary": "用户提到面试成功", "topics": ["职业变化"]}"#;
        let result = parse_emotion_response(json);
        assert_eq!(result.emotions, vec!["joy", "hope"]);
        assert_eq!(result.event_type, Some("milestone".to_string()));
        assert!((result.importance - 0.8).abs() < f64::EPSILON);
        assert_eq!(result.topics, vec!["职业变化"]);
    }

    #[test]
    fn test_filter_invalid_emotions() {
        let json = r#"{"emotions": ["joy", "invalid_emotion", "calm"], "event_type": null, "importance": 0.5, "summary": "test"}"#;
        let result = parse_emotion_response(json);
        assert_eq!(result.emotions, vec!["joy", "calm"]);
    }

    #[test]
    fn test_filter_invalid_event_type() {
        let json = r#"{"emotions": [], "event_type": "unknown_type", "importance": 0.5, "summary": "test"}"#;
        let result = parse_emotion_response(json);
        assert!(result.event_type.is_none());
    }

    #[test]
    fn test_importance_clamped_high() {
        let json = r#"{"emotions": [], "event_type": null, "importance": 1.5, "summary": "test"}"#;
        let result = parse_emotion_response(json);
        assert!((result.importance - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_importance_clamped_low() {
        let json = r#"{"emotions": [], "event_type": null, "importance": -0.5, "summary": "test"}"#;
        let result = parse_emotion_response(json);
        assert!((result.importance - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_strip_codeblock() {
        let input = "```json\n{\"emotions\": []}\n```";
        let stripped = strip_codeblock(input);
        assert_eq!(stripped, "{\"emotions\": []}");
    }

    #[test]
    fn test_strip_codeblock_no_wrapping() {
        let input = "{\"emotions\": []}";
        let stripped = strip_codeblock(input);
        assert_eq!(stripped, "{\"emotions\": []}");
    }

    #[test]
    fn test_graceful_degradation() {
        let result = parse_emotion_response("this is not json at all");
        assert!(result.emotions.is_empty());
        assert!(result.event_type.is_none());
        assert_eq!(result.importance, 0.0);
    }

    #[test]
    fn test_quick_event_threshold() {
        let important = EmotionResult {
            emotions: vec!["hope".to_string()],
            event_type: Some("milestone".to_string()),
            importance: 0.7,
            summary: "Started a new project".to_string(),
            topics: vec![],
        };
        assert!(should_persist_quick_event(&important));

        let casual = EmotionResult {
            importance: 0.7,
            summary: "Had tea".to_string(),
            ..EmotionResult::empty()
        };
        assert!(!should_persist_quick_event(&casual));
    }

    #[test]
    fn test_format_quick_note_for_extraction() {
        assert_eq!(
            format_quick_note_for_extraction("Decision", "I chose Rust"),
            "Decision\n\nI chose Rust"
        );
        assert_eq!(
            format_quick_note_for_extraction(" ", "Only body"),
            "Only body"
        );
    }

    #[test]
    fn test_empty_json() {
        let result = parse_emotion_response("{}");
        assert!(result.emotions.is_empty());
        assert!(result.event_type.is_none());
        assert_eq!(result.importance, 0.0);
        assert!(result.summary.is_empty());
    }
}
