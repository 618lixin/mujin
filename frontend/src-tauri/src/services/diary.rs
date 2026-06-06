use std::path::Path;

use super::config::{load_ai_config, AiConfig};
use super::llm::call_llm;
use super::memory::load_core_memory;
use super::notes::{default_store, AppError, NoteStore, SaveNoteRequest};
use super::types::{ChatMessage, DiaryEntry, DiaryGenerateResult};

/// Maximum number of events/turns to include as source data for diary generation.
const MAX_SOURCE_EVENTS: usize = 100;
const MAX_SOURCE_TURNS: usize = 200;
const DIARY_TEMPERATURE: f64 = 0.2;

const DIARY_ANTI_HALLUCINATION_RULES: &str = r#"ANTI-HALLUCINATION RULES (highest priority):
- Write only facts that are explicitly present in same-day chat summaries or same-day user notes.
- Events, core memory, and related past memories are context only; they must not create new facts for today.
- Do not infer a concrete event from an emotion, topic, or memory.
- Do not invent causes, timelines, actions, dialogue, locations, decisions, outcomes, or relationship status.
- If a sentence cannot be traced to the supplied same-day chat or note material, omit it.
- If the same-day material is sparse, write a sparse diary. Do not compensate by adding plausible details.
- It is acceptable to say that the day has too little material to write much."#;

/// Validate a date string as YYYY-MM-DD.
fn validate_date(date: &str) -> Result<(), AppError> {
    if date.len() != 10 {
        return Err(AppError::new(
            "invalidDate",
            format!("Invalid date format: '{date}'. Expected YYYY-MM-DD"),
        ));
    }
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(AppError::new(
            "invalidDate",
            format!("Invalid date format: '{date}'. Expected YYYY-MM-DD"),
        ));
    }
    let year: i32 = parts[0]
        .parse()
        .map_err(|_| AppError::new("invalidDate", format!("Invalid year in date: '{date}'")))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| AppError::new("invalidDate", format!("Invalid month in date: '{date}'")))?;
    let day: u32 = parts[2]
        .parse()
        .map_err(|_| AppError::new("invalidDate", format!("Invalid day in date: '{date}'")))?;
    if !(2020..=2099).contains(&year) {
        return Err(AppError::new(
            "invalidDate",
            format!("Year out of range 2020-2099: '{date}'"),
        ));
    }
    if !(1..=12).contains(&month) {
        return Err(AppError::new(
            "invalidDate",
            format!("Month out of range 1-12: '{date}'"),
        ));
    }
    if !(1..=31).contains(&day) {
        return Err(AppError::new(
            "invalidDate",
            format!("Day out of range 1-31: '{date}'"),
        ));
    }
    // Basic month-day sanity checks
    if (month == 4 || month == 6 || month == 9 || month == 11) && day > 30 {
        return Err(AppError::new(
            "invalidDate",
            format!("Day 31 not valid for month {month}: '{date}'"),
        ));
    }
    if month == 2 {
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let max_day = if is_leap { 29 } else { 28 };
        if day > max_day {
            return Err(AppError::new(
                "invalidDate",
                format!("Day {day} not valid for February {year}: '{date}'"),
            ));
        }
    }
    Ok(())
}

/// Convert a local date string (YYYY-MM-DD) to a UTC timestamp range
/// that covers the entire local day. Returns (utc_start, utc_end) as ISO 8601
/// strings suitable for string comparison with DB timestamps.
///
/// Example for UTC+8: "2026-05-30" → ("2026-05-29T16:00:00Z", "2026-05-30T15:59:59Z")
fn local_date_to_utc_range(date: &str) -> (String, String) {
    use chrono::{Local, TimeZone, Utc};

    let naive_date =
        chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("date validated before call");
    let local_start = Local
        .from_local_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    let local_end = Local
        .from_local_datetime(&naive_date.and_hms_opt(23, 59, 59).unwrap())
        .unwrap();

    let utc_start = local_start.with_timezone(&Utc);
    let utc_end = local_end.with_timezone(&Utc);

    (
        utc_start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        utc_end.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    )
}

/// Get today's date in YYYY-MM-DD format (local time).
fn today_local() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d").to_string()
}

/// Find an existing diary note for the given user and date.
/// Returns Some(DiaryEntry) if found, None otherwise.
fn find_existing_diary(store: &NoteStore, date: &str) -> Result<Option<DiaryEntry>, AppError> {
    let notes = store.list_notes()?;
    for meta in &notes {
        if meta.category == "diary" && meta.title == date {
            let note = store.read_note(&meta.id)?;
            return Ok(Some(DiaryEntry {
                date: date.to_string(),
                note_id: note.id,
                title: note.title,
                content: note.content,
                created_at: note.created_at.to_rfc3339(),
                updated_at: note.updated_at.to_rfc3339(),
            }));
        }
    }
    Ok(None)
}

/// List all diary entries for the given user, up to `limit`.
fn list_diary_entries(store: &NoteStore, limit: usize) -> Result<Vec<DiaryEntry>, AppError> {
    let notes = store.list_notes()?;
    let mut entries = Vec::new();
    for meta in &notes {
        if meta.category == "diary" {
            entries.push(DiaryEntry {
                date: meta.title.clone(),
                note_id: meta.id.clone(),
                title: meta.title.clone(),
                content: String::new(), // content populated on read
                created_at: meta.created_at.to_rfc3339(),
                updated_at: meta.updated_at.to_rfc3339(),
            });
            if entries.len() >= limit {
                break;
            }
        }
    }
    Ok(entries)
}

/// Build the LLM prompt for diary generation.
fn build_diary_prompt(
    date: &str,
    events: &str,
    turns: &str,
    notes: &str,
    core_memory: &str,
    has_data: bool,
    related_memories: Option<&str>,
) -> String {
    if !has_data {
        return String::new();
    }

    let mut parts = Vec::new();

    parts.push(format!(
        "你是一个克制、准确的日记整理助手。请根据以下用户今天的事件、对话摘要和随手笔记，生成一篇中文日记。\n\
         \n\
         硬性要求：\n\
         - 日记使用 Markdown 格式。\n\
         - 以 `# {date}` 开头。\n\
         - 必须以“今天的对话摘要”和“今天的笔记”为主要依据；事件和过往记忆只作为辅助上下文。\n\
         - 可以做轻微整理和谨慎延伸，但不得编造未出现的人物、地点、情节、动作、对白或具体心理活动。\n\
         - 不要写成小说、故事或散文，不要添加戏剧化场景。\n\
         - 如果材料没有明确说发生了什么，只能写「记录里没有展开」「似乎」「可能」，不要补全细节。\n\
         - 如果数据不多，写短一些即可，不要凑字数；优先准确，其次才是文采。\n\
         - 控制在 600-1200 字左右。\n\
         \n\
         --- 日记内容 ---\n\
         \n\
         # {date}\n\
         \n\
         今天你..."
    ));

    parts.push(
        "写作风格要求（优先级最高）：\n\
         - 日记要自然像本人写下来的日记，不要像 AI 资料整理。\n\
         - 不要写成资料汇总、复盘报告或事实核查。\n\
         - 不要在正文里写“笔记里”。\n\
         - 不要在正文里写“对话里”。\n\
         - 不要在正文里写“重要事件里”。\n\
         - 不要在正文里写“记忆里”。\n\
         - 不要在正文里写“记录显示”“根据材料”。\n\
         - 不要把矛盾分析过程写出来；如果事件、笔记、记忆之间不一致，静默采用今天的笔记和对话，忽略不可靠的信息。\n\
         - 可以把过往记忆化成一句自然的背景感受，例如“之前一直悬着的那件事终于落地了”，但不要标日期、不要解释来源。\n\
         - 语气可以有一点松动、生活感和自嘲；允许短句，允许情绪上的停顿。\n\
         - 正文只输出日记成品，不输出分析、清单、证据判断或写作说明。"
            .to_string(),
    );

    parts.push(DIARY_ANTI_HALLUCINATION_RULES.to_string());

    if !core_memory.is_empty() {
        parts.push(format!("--- 用户背景 ---\n\n{}", core_memory));
    }

    if !events.is_empty() {
        parts.push(format!("--- 今天的重要事件 ---\n\n{}", events));
    }

    if !turns.is_empty() {
        parts.push(format!("--- 今天的对话摘要 ---\n\n{}", turns));
    }

    if !notes.is_empty() {
        parts.push(format!("--- 今天的笔记 ---\n\n{}", notes));
    }

    if let Some(related) = related_memories {
        if !related.is_empty() {
            parts.push(related.to_string());
        }
    }

    parts.push(format!("请根据以上信息，生成 # {date} 的日记正文："));

    parts.join("\n\n")
}

/// Format events for prompt inclusion.
fn format_events_for_prompt(events: &[super::types::Event]) -> String {
    if events.is_empty() {
        return String::new();
    }
    events
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let emotions = if e.emotions.is_empty() {
                String::new()
            } else {
                format!(" [情绪: {}]", e.emotions.join(", "))
            };
            format!(
                "{}. {}{} (重要性: {:.2})",
                i + 1,
                e.content,
                emotions,
                e.importance
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format conversation turns for prompt inclusion.
fn format_turns_for_prompt(turns: &[super::types::ConversationTurn]) -> String {
    if turns.is_empty() {
        return String::new();
    }
    turns
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if t.summary.is_empty() {
                format!("{}. (无摘要)", i + 1)
            } else {
                format!("{}. {}", i + 1, t.summary)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Filter notes created on the given local date, returning their contents.
/// Notes that are themselves diary entries are excluded to avoid circular sourcing.
fn query_notes_by_date(store: &NoteStore, date: &str) -> Result<Vec<(String, String)>, AppError> {
    use chrono::{Local, TimeZone, Utc};

    let naive_date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| AppError::new("invalidDate", format!("Failed to parse date '{date}': {e}")))?;

    let local_start = Local
        .from_local_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    let local_end = Local
        .from_local_datetime(&naive_date.and_hms_opt(23, 59, 59).unwrap())
        .unwrap();
    let utc_start = local_start.with_timezone(&Utc);
    let utc_end = local_end.with_timezone(&Utc);

    let notes = store.list_notes()?;
    let mut result: Vec<(String, String)> = Vec::new();
    for meta in &notes {
        // Skip diary entries themselves
        if meta.category == "diary" {
            continue;
        }
        if meta.created_at >= utc_start && meta.created_at <= utc_end {
            let note = store.read_note(&meta.id)?;
            result.push((meta.title.clone(), note.content));
        }
    }
    Ok(result)
}

/// Format user notes for prompt inclusion.
fn format_notes_for_prompt(notes: &[(String, String)]) -> String {
    if notes.is_empty() {
        return String::new();
    }
    notes
        .iter()
        .enumerate()
        .map(|(i, (title, content))| {
            // Truncate very long notes to keep prompt manageable
            let preview = if content.chars().count() > 800 {
                format!(
                    "{}…（以下内容过长，已截断）",
                    &content.chars().take(800).collect::<String>()
                )
            } else {
                content.clone()
            };
            format!("### {}. {}\n\n{}", i + 1, title, preview)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Build the empty-day diary markdown.
fn empty_day_diary(date: &str) -> String {
    format!("# {date}\n\n今天还没有足够的记录生成日记。\n")
}

fn has_direct_diary_sources(
    turns: &[super::types::ConversationTurn],
    notes: &[(String, String)],
) -> bool {
    turns.iter().any(|turn| !turn.summary.trim().is_empty())
        || notes.iter().any(|(_, content)| !content.trim().is_empty())
}

/// Create or update a diary note in the "diary" category.
fn save_diary_note(
    store: &NoteStore,
    date: &str,
    content: &str,
    existing_note_id: Option<&str>,
    is_regenerate: bool,
) -> Result<DiaryEntry, AppError> {
    // Ensure "diary" category exists (ignore if already exists)
    let _ = store.create_category("diary");

    if let Some(note_id) = existing_note_id {
        // Update existing note
        let updated = store.update_note(
            note_id,
            SaveNoteRequest {
                title: date.to_string(),
                content: content.to_string(),
                category: "diary".to_string(),
            },
        )?;
        Ok(DiaryEntry {
            date: date.to_string(),
            note_id: updated.id,
            title: updated.title,
            content: updated.content,
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    } else {
        // Create new note
        let created = store.create_note(SaveNoteRequest {
            title: date.to_string(),
            content: content.to_string(),
            category: "diary".to_string(),
        })?;
        let _ = is_regenerate; // used for logging clarity
        Ok(DiaryEntry {
            date: date.to_string(),
            note_id: created.id,
            title: created.title,
            content: created.content,
            created_at: created.created_at.to_rfc3339(),
            updated_at: created.updated_at.to_rfc3339(),
        })
    }
}

// ─── Public API ────────────────────────────────────────────────────────────

/// Generate a diary for the given date (or today if date is None).
/// If a diary already exists for the date, regenerate and replace it so later
/// conversations and notes are reflected without manual deletion.
pub async fn generate_diary(
    base_dir: &Path,
    db: &super::database::DbState,
    client: &reqwest::Client,
    user_id: &str,
    date: Option<String>,
) -> Result<DiaryGenerateResult, AppError> {
    let date = date.unwrap_or_else(today_local);
    validate_date(&date)?;

    let store = default_store()?;
    let config = load_ai_config(base_dir)?;

    let existing_id = find_existing_diary(&store, &date)?.map(|e| e.note_id);

    generate_diary_inner(
        base_dir,
        db,
        client,
        user_id,
        &date,
        &config,
        existing_id.as_deref(),
    )
    .await
}

/// Regenerate a diary for the given date, replacing any existing one.
pub async fn regenerate_diary(
    base_dir: &Path,
    db: &super::database::DbState,
    client: &reqwest::Client,
    user_id: &str,
    date: String,
) -> Result<DiaryGenerateResult, AppError> {
    validate_date(&date)?;

    let store = default_store()?;
    let config = load_ai_config(base_dir)?;

    // Find existing diary note id for update
    let existing_id = find_existing_diary(&store, &date)?.map(|e| e.note_id);

    generate_diary_inner(
        base_dir,
        db,
        client,
        user_id,
        &date,
        &config,
        existing_id.as_deref(),
    )
    .await
}

/// Get a specific diary entry by date.
///
/// Note: `user_id` is accepted for future multi-user scoping but not yet
/// enforced — the current NoteStore backend does not partition diary notes
/// per user. When multi-user support is added, this function should filter
/// by `user_id`.
pub fn get_diary(_user_id: &str, date: String) -> Result<Option<DiaryEntry>, AppError> {
    validate_date(&date)?;

    let store = default_store()?;
    find_existing_diary(&store, &date)
}

/// List diary entries for the user.
///
/// Note: `user_id` is accepted for future multi-user scoping but not yet
/// enforced — the current NoteStore backend does not partition diary notes
/// per user. When multi-user support is added, this function should filter
/// by `user_id`.
pub fn get_diary_list(_user_id: &str, limit: Option<usize>) -> Result<Vec<DiaryEntry>, AppError> {
    let store = default_store()?;
    list_diary_entries(&store, limit.unwrap_or(30))
}

// ─── Core Generation Logic ─────────────────────────────────────────────────

async fn generate_diary_inner(
    base_dir: &Path,
    db: &super::database::DbState,
    client: &reqwest::Client,
    user_id: &str,
    date: &str,
    config: &AiConfig,
    existing_note_id: Option<&str>,
) -> Result<DiaryGenerateResult, AppError> {
    let is_regenerate = existing_note_id.is_some();

    // Gather source data — convert local date to UTC range for correct DB comparison
    let (utc_start, utc_end) = local_date_to_utc_range(date);
    let events = db.query_events_by_date(user_id, &utc_start, &utc_end, MAX_SOURCE_EVENTS)?;
    let turns =
        db.query_conversation_turns_by_date(user_id, &utc_start, &utc_end, MAX_SOURCE_TURNS)?;
    let core_memory = load_core_memory(base_dir, user_id, config)?;
    let store_for_notes = default_store()?;
    let user_notes = query_notes_by_date(&store_for_notes, date)?;

    let source_event_count = events.len();
    let source_turn_count = turns.len();
    let source_note_count = user_notes.len();
    let has_direct_sources = has_direct_diary_sources(&turns, &user_notes);

    // Empty day: derived events and long-term memory are not enough to write a diary.
    if !has_direct_sources {
        let content = empty_day_diary(date);
        let store = default_store()?;
        let entry = save_diary_note(&store, date, &content, existing_note_id, is_regenerate)?;
        return Ok(DiaryGenerateResult {
            date: date.to_string(),
            note_id: entry.note_id,
            title: entry.title,
            content: entry.content,
            source_event_count,
            source_turn_count,
            source_note_count,
            regenerated: is_regenerate,
        });
    }

    // Retrieve related past memories for diary context
    let related_memories = super::diary_memory::retrieve_related_diary_memories(
        db,
        &super::diary_memory::DiaryMemoryQuery {
            user_id,
            diary_date: date,
            day_events: &events,
            day_turns: &turns,
            day_notes: &user_notes,
            max_results: 5,
        },
    )?;

    let related_memories_text =
        super::diary_memory::format_related_memories_for_prompt(&related_memories);

    // Build LLM prompt
    let events_text = format_events_for_prompt(&events);
    let turns_text = format_turns_for_prompt(&turns);
    let notes_text = format_notes_for_prompt(&user_notes);
    let memory_text = super::memory::format_memory_for_prompt(&core_memory);

    let prompt = build_diary_prompt(
        date,
        &events_text,
        &turns_text,
        &notes_text,
        &memory_text,
        true,
        related_memories_text.as_deref(),
    );

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: DIARY_ANTI_HALLUCINATION_RULES.to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: prompt,
        },
    ];

    // Call LLM (use main model for user-facing diary artifact)
    let llm_output = call_llm(client, config, &messages, None, DIARY_TEMPERATURE, 2000).await?;

    // Clean up the output: ensure it starts with the heading
    let content = if llm_output.trim().starts_with(&format!("# {date}")) {
        llm_output.trim().to_string()
    } else {
        // Prepend heading if LLM didn't include it
        format!("# {date}\n\n{}", llm_output.trim())
    };

    // Save diary note
    let store = default_store()?;
    let entry = save_diary_note(&store, date, &content, existing_note_id, is_regenerate)?;

    // Record recall for related event memories that were surfaced to the prompt.
    // Only record after successful diary generation and save.
    for mem in &related_memories {
        if let Some(ref event_id) = mem.event_id {
            // Use a small recall boost consistent with diary generation.
            let _ = db.record_recall(user_id, event_id, 0.2);
        }
    }

    Ok(DiaryGenerateResult {
        date: date.to_string(),
        note_id: entry.note_id,
        title: entry.title,
        content: entry.content,
        source_event_count,
        source_turn_count,
        source_note_count,
        regenerated: is_regenerate,
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_date_accepts_valid() {
        assert!(validate_date("2026-05-30").is_ok());
        assert!(validate_date("2026-01-01").is_ok());
        assert!(validate_date("2026-12-31").is_ok());
        assert!(validate_date("2024-02-29").is_ok()); // leap year
    }

    #[test]
    fn test_validate_date_rejects_invalid() {
        assert!(validate_date("").is_err());
        assert!(validate_date("not-a-date").is_err());
        assert!(validate_date("2026-13-01").is_err()); // bad month
        assert!(validate_date("2026-01-32").is_err()); // bad day
        assert!(validate_date("2026-02-30").is_err()); // Feb 30
        assert!(validate_date("2025-02-29").is_err()); // non-leap Feb 29
        assert!(validate_date("2019-01-01").is_err()); // year out of range
        assert!(validate_date("2100-01-01").is_err()); // year out of range
        assert!(validate_date("2026/05/30").is_err()); // wrong separator
        assert!(validate_date("2026-4-5").is_err()); // not zero-padded
    }

    #[test]
    fn test_empty_day_diary_format() {
        let content = empty_day_diary("2026-05-30");
        assert!(content.starts_with("# 2026-05-30"));
        assert!(content.contains("没有足够的记录"));
    }

    #[test]
    fn test_direct_diary_sources_require_nonempty_turn_summary_or_note() {
        use super::super::types::ConversationTurn;

        let no_turns: Vec<ConversationTurn> = vec![];
        let no_notes: Vec<(String, String)> = vec![];
        assert!(!has_direct_diary_sources(&no_turns, &no_notes));

        let empty_turns = vec![ConversationTurn {
            id: 1,
            summary: "  ".to_string(),
            emotions: vec![],
            created_at: "2026-06-04T10:00:00Z".to_string(),
        }];
        assert!(!has_direct_diary_sources(&empty_turns, &no_notes));

        let useful_turns = vec![ConversationTurn {
            id: 2,
            summary: "user said something concrete".to_string(),
            emotions: vec![],
            created_at: "2026-06-04T10:00:00Z".to_string(),
        }];
        assert!(has_direct_diary_sources(&useful_turns, &no_notes));

        let blank_notes = vec![("note".to_string(), "   ".to_string())];
        assert!(!has_direct_diary_sources(&no_turns, &blank_notes));

        let useful_notes = vec![("note".to_string(), "one concrete note".to_string())];
        assert!(has_direct_diary_sources(&no_turns, &useful_notes));
    }

    #[test]
    fn test_format_events_for_prompt() {
        use super::super::types::Event;

        let events = vec![Event {
            id: "e1".to_string(),
            content: "用户今天面试了".to_string(),
            emotions: vec!["焦虑".to_string(), "期待".to_string()],
            importance: 0.8,
            event_type: Some("milestone".to_string()),
            strength: 1.0,
            stability: 30.0,
            recall_count: 0,
            last_recalled_at: None,
            created_at: "2026-05-30T10:00:00Z".to_string(),
            updated_at: "2026-05-30T10:00:00Z".to_string(),
        }];

        let formatted = format_events_for_prompt(&events);
        assert!(formatted.contains("面试"));
        assert!(formatted.contains("焦虑"));
        assert!(formatted.contains("期待"));
        assert!(formatted.contains("0.80"));
    }

    #[test]
    fn test_format_turns_for_prompt() {
        use super::super::types::ConversationTurn;

        let turns = vec![ConversationTurn {
            id: 1,
            summary: "用户讨论了职业规划".to_string(),
            emotions: vec!["困惑".to_string()],
            created_at: "2026-05-30T10:00:00Z".to_string(),
        }];

        let formatted = format_turns_for_prompt(&turns);
        assert!(formatted.contains("职业规划"));
    }

    #[test]
    fn test_format_turns_empty_summary() {
        use super::super::types::ConversationTurn;

        let turns = vec![ConversationTurn {
            id: 1,
            summary: String::new(),
            emotions: vec![],
            created_at: "2026-05-30T10:00:00Z".to_string(),
        }];

        let formatted = format_turns_for_prompt(&turns);
        assert!(formatted.contains("无摘要"));
    }

    #[test]
    fn test_build_diary_prompt_no_data() {
        let prompt = build_diary_prompt("2026-05-30", "", "", "", "", false, None);
        assert!(prompt.is_empty());
    }

    #[test]
    fn test_build_diary_prompt_with_data() {
        let prompt = build_diary_prompt(
            "2026-05-30",
            "1. 面试事件",
            "1. 职业讨论",
            "### 1. 想法\n\n今天的一些思考",
            "用户画像: 程序员",
            true,
            None,
        );
        assert!(prompt.contains("# 2026-05-30"));
        assert!(prompt.contains("面试事件"));
        assert!(prompt.contains("职业讨论"));
        assert!(prompt.contains("今天的一些思考"));
        assert!(prompt.contains("用户画像"));
    }

    #[test]
    fn test_build_diary_prompt_requires_grounded_nonfiction_style() {
        let prompt = build_diary_prompt(
            "2026-05-30",
            "1. 面试事件",
            "1. 用户提到今天有点累",
            "### 1. 随手记\n\n只写了几个关键词",
            "",
            true,
            None,
        );

        assert!(prompt.contains("今天的对话摘要"));
        assert!(prompt.contains("今天的笔记"));
        assert!(prompt.contains("不得编造"));
        assert!(prompt.contains("不要写成小说"));
        assert!(prompt.contains("不要添加戏剧化场景"));
        assert!(prompt.contains("优先准确"));
    }

    #[test]
    fn test_build_diary_prompt_requires_natural_diary_voice() {
        let prompt = build_diary_prompt(
            "2026-06-03",
            "1. 用户补考通过；2. 用户和女朋友吵架",
            "1. 用户犹豫要不要主动去哄女朋友",
            "### 1. 随手记\n\n补考成功过了，哈哈哈\n\n今天和女朋友吵架了",
            "",
            true,
            Some("--- 可能相关的过往记忆 ---\n- [2026-05-31] 用户曾因补考焦虑\n---"),
        );

        assert!(prompt.contains("自然像本人写下来的日记"));
        assert!(prompt.contains("不要写成资料汇总、复盘报告或事实核查"));
        assert!(prompt.contains("不要在正文里写“笔记里”"));
        assert!(prompt.contains("不要在正文里写“重要事件里”"));
        assert!(prompt.contains("不要在正文里写“记忆里”"));
        assert!(prompt.contains("不要把矛盾分析过程写出来"));
        assert!(prompt.contains("静默采用今天的笔记和对话"));
    }

    #[test]
    fn test_build_diary_prompt_contains_anti_hallucination_rules() {
        let prompt = build_diary_prompt(
            "2026-06-04",
            "1. derived event",
            "1. user said hello",
            "",
            "",
            true,
            Some("--- related memory ---\nold memory\n---"),
        );

        assert!(prompt.contains("ANTI-HALLUCINATION RULES"));
        assert!(prompt.contains("same-day chat summaries or same-day user notes"));
        assert!(prompt.contains("must not create new facts for today"));
        assert!(prompt.contains("Do not infer a concrete event"));
        assert!(prompt.contains("Do not invent causes"));
        assert!(prompt.contains("write a sparse diary"));
    }

    #[test]
    fn test_diary_generation_uses_low_temperature() {
        assert!(DIARY_TEMPERATURE <= 0.2);
    }

    #[test]
    fn test_today_local_format() {
        let today = today_local();
        assert_eq!(today.len(), 10);
        assert!(today.chars().nth(4) == Some('-'));
        assert!(today.chars().nth(7) == Some('-'));
    }

    #[test]
    fn test_local_date_to_utc_range_format() {
        let (start, end) = local_date_to_utc_range("2026-05-30");
        // Both should end with Z (UTC marker)
        assert!(start.ends_with('Z'), "start should end with Z: {start}");
        assert!(end.ends_with('Z'), "end should end with Z: {end}");
        // Start should be before end in string comparison
        assert!(
            start < end,
            "utc_start should be before utc_end: {start} >= {end}"
        );
        // Both should have the expected ISO 8601 format
        assert_eq!(
            start.len(),
            20,
            "expected 20 chars, got {}: '{start}'",
            start.len()
        );
        assert_eq!(
            end.len(),
            20,
            "expected 20 chars, got {}: '{end}'",
            end.len()
        );
    }

    #[test]
    fn test_local_date_to_utc_range_covers_full_day() {
        // For any local date, the UTC range should span ~24 hours
        let (start, end) = local_date_to_utc_range("2026-06-15");
        // Parse the timestamps to compute duration
        let start_dt =
            chrono::NaiveDateTime::parse_from_str(start.trim_end_matches('Z'), "%Y-%m-%dT%H:%M:%S")
                .unwrap();
        let end_dt =
            chrono::NaiveDateTime::parse_from_str(end.trim_end_matches('Z'), "%Y-%m-%dT%H:%M:%S")
                .unwrap();
        let duration = end_dt - start_dt;
        // Should be 23:59:59 (86399 seconds) — effectively 24h
        assert_eq!(
            duration.num_seconds(),
            86399,
            "UTC range should span 23h59m59s, got {}s",
            duration.num_seconds()
        );
    }

    #[test]
    fn test_diary_idempotency_find_existing() {
        // Create a temp NoteStore and save a diary, then verify find_existing_diary returns it
        let temp_dir = std::env::temp_dir().join("gc-test-diary-idempotency");
        let _ = std::fs::create_dir_all(&temp_dir);

        let store = super::super::notes::NoteStore::new(temp_dir.clone());
        let _ = store.create_category("diary");

        let date = "2026-05-30";
        let content = "# 2026-05-30\n\nTest diary content.";

        // First save — creates a new note
        let entry1 = save_diary_note(&store, date, content, None, false).unwrap();
        assert!(!entry1.note_id.is_empty());
        assert_eq!(entry1.date, date);
        assert_eq!(entry1.content, content);

        // find_existing_diary should now return this entry
        let found = find_existing_diary(&store, date).unwrap();
        assert!(
            found.is_some(),
            "find_existing_diary should find the saved diary"
        );
        let found = found.unwrap();
        assert_eq!(
            found.note_id, entry1.note_id,
            "should return the same note_id"
        );
        assert_eq!(found.date, date);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_diary_regenerate_replaces_content() {
        let temp_dir = std::env::temp_dir().join("gc-test-diary-regenerate");
        let _ = std::fs::create_dir_all(&temp_dir);

        let store = super::super::notes::NoteStore::new(temp_dir.clone());
        let _ = store.create_category("diary");

        let date = "2026-05-30";
        let original_content = "# 2026-05-30\n\nOriginal diary.";
        let new_content = "# 2026-05-30\n\nRegenerated diary with new insights.";

        // Create the original diary
        let entry1 = save_diary_note(&store, date, original_content, None, false).unwrap();
        let note_id = entry1.note_id.clone();

        // Simulate regeneration: pass existing_note_id to update in place
        let entry2 = save_diary_note(&store, date, new_content, Some(&note_id), true).unwrap();

        // The note_id should stay the same (update, not create)
        assert_eq!(
            entry2.note_id, note_id,
            "regenerate should update existing note, not create a new one"
        );
        assert_eq!(
            entry2.content, new_content,
            "content should be replaced with new content"
        );

        // Reading the note directly should return the new content
        let read_back = store.read_note(&note_id).unwrap();
        assert_eq!(read_back.content, new_content);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_build_diary_prompt_formats_date_in_final_instruction() {
        let prompt = build_diary_prompt(
            "2026-05-30",
            "1. 面试事件",
            "1. 职业讨论",
            "",
            "",
            true,
            None,
        );
        // The final instruction should contain the actual date, not the literal "{date}"
        assert!(
            !prompt.contains("{date}"),
            "prompt should not contain literal {{date}}: {prompt}"
        );
        let last_line = prompt.lines().last().unwrap();
        assert!(
            last_line.contains("2026-05-30"),
            "final instruction should contain formatted date, got: {last_line}"
        );
    }

    #[test]
    fn test_format_notes_for_prompt() {
        let notes = vec![
            ("想法".to_string(), "今天的一些思考".to_string()),
            ("计划".to_string(), "明天要做的事情".to_string()),
        ];
        let formatted = format_notes_for_prompt(&notes);
        assert!(formatted.contains("想法"));
        assert!(formatted.contains("今天的一些思考"));
        assert!(formatted.contains("计划"));
        assert!(formatted.contains("明天要做的事情"));
    }

    #[test]
    fn test_format_notes_empty() {
        let notes: Vec<(String, String)> = vec![];
        let formatted = format_notes_for_prompt(&notes);
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_format_notes_truncates_long_content() {
        let long_content = "a".repeat(2000);
        let notes = vec![("长文".to_string(), long_content)];
        let formatted = format_notes_for_prompt(&notes);
        // Should contain truncated marker
        assert!(formatted.contains("已截断"));
        // Should not include the full 2000 chars
        assert!(formatted.len() < 2000);
    }

    #[test]
    fn test_build_diary_prompt_with_related_memories() {
        let related = "--- 可能相关的过往记忆 ---\n这些记忆可能与今天有关。\n- [2026-05-20] 用户参加了面试 (topic: career)\n---\n";
        let prompt =
            build_diary_prompt("2026-05-30", "1. 面试事件", "", "", "", true, Some(related));
        assert!(
            prompt.contains("可能相关的过往记忆"),
            "should include related memories section"
        );
        assert!(
            prompt.contains("[2026-05-20]"),
            "should include related memory date"
        );
    }

    #[test]
    fn test_build_diary_prompt_without_related_memories() {
        let prompt = build_diary_prompt("2026-05-30", "1. 面试事件", "", "", "", true, None);
        assert!(
            !prompt.contains("可能相关的过往记忆"),
            "should NOT include related memories section"
        );
    }
}
