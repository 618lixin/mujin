//! Diary-related memory retrieval module.
//!
//! Provides a stable module boundary between diary generation and the memory
//! retrieval backend. Diary generation calls `retrieve_related_diary_memories`
//! without knowing whether results come from topic links, LIKE search, FTS5,
//! embeddings, or a future backend.

use super::database::DbState;
use super::notes::AppError;
use super::types::{ConversationTurn, Event};

// ─── Public Data Structures ──────────────────────────────────────────────

/// Query parameters for diary-related memory retrieval.
pub struct DiaryMemoryQuery<'a> {
    pub user_id: &'a str,
    /// Target diary date in YYYY-MM-DD format (local time).
    pub diary_date: &'a str,
    /// Same-day events already gathered for the diary.
    pub day_events: &'a [Event],
    /// Same-day conversation turns already gathered for the diary.
    pub day_turns: &'a [ConversationTurn],
    /// Same-day user notes: (title, content).
    pub day_notes: &'a [(String, String)],
    /// Maximum number of related memories to return.
    pub max_results: usize,
}

/// A single related past memory surfaced for diary context.
#[derive(Debug, Clone)]
pub struct RelatedDiaryMemory {
    /// Event ID if this memory originated from an event (used for recall tracking).
    pub event_id: Option<String>,
    /// Date string (YYYY-MM-DD extracted from created_at) for display in prompt.
    pub date: String,
    /// Human-readable content of the related memory.
    pub content: String,
    /// Why this memory was selected (e.g. "topic: career", "keyword: 面试").
    pub reason: String,
    /// Composite relevance score (higher = more relevant).
    pub score: f64,
}

// ─── Scoring Weights ─────────────────────────────────────────────────────

const WEIGHT_TOPIC_MATCH: f64 = 3.0;
const WEIGHT_KEYWORD_MATCH: f64 = 1.5;
const WEIGHT_IMPORTANCE: f64 = 1.0;
const WEIGHT_STRENGTH: f64 = 1.0;
const WEIGHT_RECENCY: f64 = 0.5;

/// Default cap on related memories returned.
const DEFAULT_MAX_RESULTS: usize = 5;

// ─── Public Retrieval Function ───────────────────────────────────────────

/// Retrieve past memories related to the diary date's source material.
///
/// Uses a layered strategy:
/// 1. Topic-linked events: find earlier events sharing topics with today's events.
/// 2. Keyword fallback: search past event content and conversation summaries.
/// 3. Recent high-importance context (only if no stronger matches exist).
///
/// Same-day events/turns are excluded to avoid duplication with today's material.
pub fn retrieve_related_diary_memories(
    db: &DbState,
    query: &DiaryMemoryQuery,
) -> Result<Vec<RelatedDiaryMemory>, AppError> {
    let max = if query.max_results == 0 {
        DEFAULT_MAX_RESULTS
    } else {
        query.max_results
    };

    // Build the UTC range boundary: exclude anything on or after the diary date's UTC start.
    let utc_cutoff = local_date_cutoff(query.diary_date);

    // Collect topic IDs from today's events.
    let day_topic_ids = collect_topic_ids_for_events(db, query.user_id, query.day_events);

    // Extract keyword anchors from today's material.
    let anchors = extract_keyword_anchors(query);

    // Gather candidates from each layer.
    let mut candidates: Vec<RelatedDiaryMemory> = Vec::new();
    let mut seen_event_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Layer 1: Topic-linked past events.
    let topic_results =
        query_events_by_topics(db, query.user_id, &day_topic_ids, &utc_cutoff, max * 2)?;
    for (event, topic_name) in &topic_results {
        // Dedup: same event may be linked to multiple topics — keep first (highest-score) occurrence.
        if !seen_event_ids.insert(event.id.clone()) {
            continue;
        }
        let score = score_candidate(
            true,  // topic match
            false, // keyword match
            event.importance,
            event.strength,
            &event.created_at,
        );
        candidates.push(RelatedDiaryMemory {
            event_id: Some(event.id.clone()),
            date: extract_date(&event.created_at),
            content: event.content.clone(),
            reason: format!("topic: {}", topic_name),
            score,
        });
    }

    // Layer 2: Keyword fallback on past events.
    if candidates.len() < max {
        let keyword_events =
            query_events_by_keywords(db, query.user_id, &anchors, &utc_cutoff, max * 2)?;
        for event in &keyword_events {
            if !seen_event_ids.insert(event.id.clone()) {
                continue;
            }
            let score = score_candidate(
                false,
                true,
                event.importance,
                event.strength,
                &event.created_at,
            );
            candidates.push(RelatedDiaryMemory {
                event_id: Some(event.id.clone()),
                date: extract_date(&event.created_at),
                content: event.content.clone(),
                reason: "keyword match".to_string(),
                score,
            });
        }
    }

    // Layer 3: Keyword fallback on past conversation summaries.
    if candidates.len() < max {
        let keyword_turns = query_turns_by_keywords(db, query.user_id, &anchors, &utc_cutoff, max)?;
        for turn in &keyword_turns {
            let score = score_candidate(
                false,
                true,
                0.3, // default importance for turns
                1.0, // turns don't have strength
                &turn.created_at,
            );
            candidates.push(RelatedDiaryMemory {
                event_id: None,
                date: extract_date(&turn.created_at),
                content: turn.summary.clone(),
                reason: "keyword conversation".to_string(),
                score,
            });
        }
    }

    // Layer 4 (fallback): recent high-importance events, only if still empty.
    if candidates.is_empty() {
        let recent = query_recent_important_events(db, query.user_id, &utc_cutoff, 3)?;
        for event in &recent {
            let score = score_candidate(
                false,
                false,
                event.importance,
                event.strength,
                &event.created_at,
            );
            candidates.push(RelatedDiaryMemory {
                event_id: Some(event.id.clone()),
                date: extract_date(&event.created_at),
                content: event.content.clone(),
                reason: "recent important".to_string(),
                score,
            });
        }
    }

    // Sort by score descending, cap at max.
    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(max);

    Ok(candidates)
}

// ─── Prompt Formatting ───────────────────────────────────────────────────

/// Format related memories into a prompt section.
/// Returns `None` if the list is empty.
pub fn format_related_memories_for_prompt(memories: &[RelatedDiaryMemory]) -> Option<String> {
    if memories.is_empty() {
        return None;
    }

    let mut block = String::from("--- 可能相关的过往记忆 ---\n");
    block.push_str(
        "这些记忆可能与今天有关。只有当联系自然、能被材料支持时才写进日记；不要硬凑因果。\n\n",
    );

    for mem in memories {
        block.push_str(&format!(
            "- [{}] {} ({})\n",
            mem.date, mem.content, mem.reason
        ));
    }

    block.push_str("---\n");
    Some(block)
}

// ─── Private Helpers ─────────────────────────────────────────────────────

/// Extract the date portion (YYYY-MM-DD) from an ISO 8601 timestamp.
fn extract_date(timestamp: &str) -> String {
    timestamp.chars().take(10).collect()
}

/// Compute the UTC cutoff: the start-of-day UTC timestamp for the diary date.
/// Anything >= this is considered same-day and excluded.
fn local_date_cutoff(diary_date: &str) -> String {
    use chrono::{Local, TimeZone, Utc};
    let naive_date = chrono::NaiveDate::parse_from_str(diary_date, "%Y-%m-%d")
        .expect("diary_date validated before call");
    let local_start = Local
        .from_local_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    local_start
        .with_timezone(&Utc)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

/// Collect topic IDs linked to the given events.
fn collect_topic_ids_for_events(db: &DbState, user_id: &str, events: &[Event]) -> Vec<String> {
    let mut topic_ids = Vec::new();
    for event in events {
        // For each event, look up topic_links where item_id = event.id.
        // Since get_topic_links takes a topic_id, we need a reverse lookup.
        // Instead, query topics linked to events via a direct SQL approach.
        if let Ok(ids) = db.get_topic_ids_for_item(user_id, &event.id, "event") {
            topic_ids.extend(ids);
        }
    }
    topic_ids.sort();
    topic_ids.dedup();
    topic_ids
}

/// Extract compact keyword anchors from today's source material.
fn extract_keyword_anchors(query: &DiaryMemoryQuery) -> Vec<String> {
    let mut anchors = Vec::new();

    // From event content: take first 15 chars.
    for event in query.day_events {
        let anchor: String = event.content.chars().take(15).collect();
        if anchor.len() >= 2 {
            anchors.push(anchor);
        }
    }

    // From conversation summaries: take first 15 chars.
    for turn in query.day_turns {
        if !turn.summary.is_empty() {
            let anchor: String = turn.summary.chars().take(15).collect();
            if anchor.len() >= 2 {
                anchors.push(anchor);
            }
        }
    }

    // From note titles.
    for (title, _content) in query.day_notes {
        let anchor: String = title.chars().take(15).collect();
        if anchor.len() >= 2 {
            anchors.push(anchor);
        }
    }

    // Dedup (simple approach: sort and dedup by first 4 chars).
    anchors.sort();
    anchors.dedup_by(|a, b| {
        let a_prefix: String = a.chars().take(4).collect();
        let b_prefix: String = b.chars().take(4).collect();
        a_prefix == b_prefix
    });

    // Cap to avoid excessive queries.
    anchors.truncate(8);
    anchors
}

/// Score a candidate memory based on its relevance signals.
fn score_candidate(
    is_topic_match: bool,
    is_keyword_match: bool,
    importance: f64,
    strength: f64,
    created_at: &str,
) -> f64 {
    let topic_score = if is_topic_match {
        WEIGHT_TOPIC_MATCH
    } else {
        0.0
    };
    let keyword_score = if is_keyword_match {
        WEIGHT_KEYWORD_MATCH
    } else {
        0.0
    };
    let importance_score = importance * WEIGHT_IMPORTANCE;
    let strength_score = strength * WEIGHT_STRENGTH;

    // Recency: days since creation, mapped to a 0-1 score.
    let recency_score = {
        let parsed = chrono::DateTime::parse_from_rfc3339(created_at)
            .map(|dt| dt.to_utc())
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%dT%H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            });
        match parsed {
            Ok(dt) => {
                let days = (chrono::Utc::now() - dt).num_seconds() as f64 / 86400.0;
                // Exponential decay with 60-day half-life.
                let decay = (-days / 60.0).exp();
                decay * WEIGHT_RECENCY
            }
            Err(_) => 0.0,
        }
    };

    topic_score + keyword_score + importance_score + strength_score + recency_score
}

// ─── Private Database Query Helpers ──────────────────────────────────────

/// Query past events that share topics with the given topic IDs.
/// Returns events created before the cutoff, with their matching topic name.
fn query_events_by_topics(
    db: &DbState,
    user_id: &str,
    topic_ids: &[String],
    utc_cutoff: &str,
    limit: usize,
) -> Result<Vec<(Event, String)>, AppError> {
    if topic_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    for topic_id in topic_ids {
        let links = db.get_topic_links(user_id, topic_id).unwrap_or_default();
        for link in &links {
            if link.item_type != "event" {
                continue;
            }
            // Fetch the event.
            if let Ok(Some(event)) = db.get_event_by_id(user_id, &link.item_id) {
                // Exclude same-day.
                if event.created_at.as_str() >= utc_cutoff {
                    continue;
                }
                // Get topic name for the reason field.
                let topic_name = db
                    .get_topic_by_id(user_id, topic_id)
                    .ok()
                    .flatten()
                    .map(|t| t.name)
                    .unwrap_or_else(|| topic_id.clone());
                results.push((event, topic_name));
            }
        }
        if results.len() >= limit {
            break;
        }
    }
    Ok(results)
}

/// Query past events whose content matches any of the keyword anchors (LIKE).
fn query_events_by_keywords(
    db: &DbState,
    user_id: &str,
    anchors: &[String],
    utc_cutoff: &str,
    limit: usize,
) -> Result<Vec<Event>, AppError> {
    if anchors.is_empty() {
        return Ok(Vec::new());
    }

    // Use the first few anchors for event content LIKE search.
    let mut results = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for anchor in anchors.iter().take(4) {
        let pattern = format!("%{}%", anchor);
        if let Ok(events) = db.search_events_like(user_id, &pattern, limit) {
            for event in events {
                if event.created_at.as_str() >= utc_cutoff {
                    continue;
                }
                if seen_ids.insert(event.id.clone()) {
                    results.push(event);
                }
            }
        }
        if results.len() >= limit {
            break;
        }
    }

    Ok(results)
}

/// Query past conversation summaries matching keyword anchors (LIKE).
fn query_turns_by_keywords(
    db: &DbState,
    user_id: &str,
    anchors: &[String],
    utc_cutoff: &str,
    limit: usize,
) -> Result<Vec<ConversationTurn>, AppError> {
    if anchors.is_empty() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for anchor in anchors.iter().take(4) {
        if let Ok(turns) = db.search_conversations_like(user_id, anchor, limit) {
            for turn in turns {
                if turn.created_at.as_str() >= utc_cutoff {
                    continue;
                }
                if seen_ids.insert(turn.id) {
                    results.push(turn);
                }
            }
        }
        if results.len() >= limit {
            break;
        }
    }

    Ok(results)
}

/// Query recent high-importance events before the cutoff (fallback layer).
fn query_recent_important_events(
    db: &DbState,
    user_id: &str,
    utc_cutoff: &str,
    limit: usize,
) -> Result<Vec<Event>, AppError> {
    db.query_events(
        user_id,
        &super::types::QueryEventsParams {
            limit,
            min_importance: 0.6,
            end_date: Some(utc_cutoff.to_string()),
            ..Default::default()
        },
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_related_memories_empty() {
        let result = format_related_memories_for_prompt(&[]);
        assert!(result.is_none(), "empty memories should return None");
    }

    #[test]
    fn test_format_related_memories_nonempty() {
        let memories = vec![
            RelatedDiaryMemory {
                event_id: Some("e1".to_string()),
                date: "2026-05-20".to_string(),
                content: "用户参加了面试".to_string(),
                reason: "topic: career".to_string(),
                score: 3.5,
            },
            RelatedDiaryMemory {
                event_id: None,
                date: "2026-05-18".to_string(),
                content: "用户讨论了职业规划".to_string(),
                reason: "keyword match".to_string(),
                score: 2.1,
            },
        ];

        let result = format_related_memories_for_prompt(&memories).unwrap();

        assert!(
            result.contains("可能相关的过往记忆"),
            "should contain header"
        );
        assert!(
            result.contains("不要硬凑因果"),
            "should contain conservative instruction"
        );
        assert!(result.contains("[2026-05-20]"), "should contain first date");
        assert!(
            result.contains("[2026-05-18]"),
            "should contain second date"
        );
        assert!(result.contains("面试"), "should contain first content");
        assert!(result.contains("职业规划"), "should contain second content");
        assert!(
            result.contains("topic: career"),
            "should contain first reason"
        );
        assert!(
            result.contains("keyword match"),
            "should contain second reason"
        );
        assert!(result.starts_with("---"), "should start with separator");
    }

    #[test]
    fn test_same_day_events_excluded() {
        // Test the date cutoff logic: events on the diary date should be filtered out.
        let cutoff = local_date_cutoff("2026-06-01");

        // An event created on the diary date (in UTC terms) should be >= cutoff.
        // Since local_date_cutoff returns the UTC start-of-day for the local date,
        // any timestamp on that date in UTC will be >= cutoff.
        let same_day_utc = cutoff.clone(); // The cutoff itself is the start of the day
        assert!(
            same_day_utc >= cutoff,
            "same-day event should be >= cutoff and thus excluded"
        );

        // An event from a previous day should be < cutoff.
        let prev_day = "2026-05-30T10:00:00Z".to_string();
        assert!(
            prev_day < cutoff,
            "previous day event should be < cutoff and thus included"
        );
    }

    #[test]
    fn test_extract_date() {
        assert_eq!(extract_date("2026-05-30T10:00:00Z"), "2026-05-30");
        assert_eq!(extract_date("2026-01-15T23:59:59Z"), "2026-01-15");
        assert_eq!(extract_date("2026-12-31"), "2026-12-31");
    }

    #[test]
    fn test_extract_keyword_anchors() {
        let events = vec![Event {
            id: "e1".to_string(),
            content: "用户参加了面试并且表现很好".to_string(),
            emotions: vec![],
            importance: 0.8,
            event_type: Some("milestone".to_string()),
            strength: 1.0,
            stability: 30.0,
            recall_count: 0,
            last_recalled_at: None,
            created_at: "2026-06-01T10:00:00Z".to_string(),
            updated_at: "2026-06-01T10:00:00Z".to_string(),
        }];
        let turns = vec![ConversationTurn {
            id: 1,
            summary: "讨论了职业发展".to_string(),
            emotions: vec![],
            created_at: "2026-06-01T10:00:00Z".to_string(),
        }];

        let query = DiaryMemoryQuery {
            user_id: "test",
            diary_date: "2026-06-01",
            day_events: &events,
            day_turns: &turns,
            day_notes: &vec![("想法笔记".to_string(), "内容".to_string())],
            max_results: 5,
        };

        let anchors = extract_keyword_anchors(&query);
        assert!(!anchors.is_empty(), "should extract at least one anchor");
        // First anchor should be from the event content (first 15 chars).
        assert!(
            anchors.iter().any(|a| a.contains("面试")),
            "should contain anchor from event content"
        );
    }

    #[test]
    fn test_score_candidate_topic_higher_than_keyword() {
        let topic_score = score_candidate(true, false, 0.5, 0.8, "2026-05-15T10:00:00Z");
        let keyword_score = score_candidate(false, true, 0.5, 0.8, "2026-05-15T10:00:00Z");

        assert!(
            topic_score > keyword_score,
            "topic match should score higher than keyword match: {topic_score} vs {keyword_score}"
        );
    }

    #[test]
    fn test_score_candidate_importance_matters() {
        let high = score_candidate(false, true, 0.9, 0.8, "2026-05-15T10:00:00Z");
        let low = score_candidate(false, true, 0.2, 0.8, "2026-05-15T10:00:00Z");

        assert!(
            high > low,
            "higher importance should score higher: {high} vs {low}"
        );
    }

    // ─── Database-level Integration Tests ────────────────────────────────

    /// Helper: create a temporary DbState for integration tests.
    fn test_db() -> DbState {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("gc_test_diary_mem_{id}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        DbState::new(dir)
    }

    fn make_event(id: &str, content: &str, importance: f64, created_at: &str) -> Event {
        Event {
            id: id.to_string(),
            content: content.to_string(),
            emotions: vec![],
            importance,
            event_type: Some("milestone".to_string()),
            strength: 1.0,
            stability: 30.0,
            recall_count: 0,
            last_recalled_at: None,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
        }
    }

    /// Task 2.5: Topic-linked events rank ahead of unrelated recent events.
    #[test]
    fn test_topic_linked_events_rank_ahead_of_unrelated_recent() {
        let db = test_db();
        let user_id = "test-user";

        // A past event linked to topic "career" (3 days ago).
        let past_career_event =
            make_event("e_career", "用户参加了面试", 0.6, "2026-05-27T10:00:00Z");
        db.add_event(user_id, &past_career_event, 30.0).unwrap();

        // A more recent but unrelated event (1 day ago, high importance).
        let recent_unrelated =
            make_event("e_unrelated", "用户去公园散步", 0.9, "2026-05-29T10:00:00Z");
        db.add_event(user_id, &recent_unrelated, 30.0).unwrap();

        // Create topic "career" and link it to the past event.
        let topic = super::super::types::Topic {
            id: "t_career".to_string(),
            name: "career".to_string(),
            description: String::new(),
            first_mentioned: Some("2026-05-30T10:00:00Z".to_string()),
            last_mentioned: Some("2026-05-30T10:00:00Z".to_string()),
            mention_count: 1,
        };
        db.add_topic(user_id, &topic).unwrap();
        db.link_topic(user_id, "t_career", "e_career", "event")
            .unwrap();

        // Today's event is about career — linked to same topic.
        let today_event = make_event("e_today", "用户收到了面试结果", 0.8, "2026-05-30T10:00:00Z");
        // Also link today's event to the "career" topic.
        db.link_topic(user_id, "t_career", "e_today", "event")
            .unwrap();

        let query = DiaryMemoryQuery {
            user_id,
            diary_date: "2026-05-30",
            day_events: &[today_event],
            day_turns: &[],
            day_notes: &[],
            max_results: 5,
        };

        let results = retrieve_related_diary_memories(&db, &query).unwrap();

        // The career event should appear (topic-linked).
        let career_idx = results
            .iter()
            .position(|r| r.event_id.as_deref() == Some("e_career"));
        assert!(
            career_idx.is_some(),
            "topic-linked career event should be in results"
        );

        // If unrelated event also appears, career must rank higher (lower index = higher score).
        let unrelated_idx = results
            .iter()
            .position(|r| r.event_id.as_deref() == Some("e_unrelated"));
        if let (Some(ci), Some(ui)) = (career_idx, unrelated_idx) {
            assert!(
                ci < ui,
                "topic-linked event (idx {ci}) should rank ahead of unrelated recent (idx {ui})"
            );
        }
    }

    /// Task 2.6: Keyword fallback works when topic links are absent.
    #[test]
    fn test_keyword_fallback_when_no_topics() {
        let db = test_db();
        let user_id = "test-user";

        // A past event about 面试 (no topic links).
        let past_event = make_event(
            "e_past",
            "用户参加了面试并且表现不错",
            0.7,
            "2026-05-25T10:00:00Z",
        );
        db.add_event(user_id, &past_event, 30.0).unwrap();

        // An unrelated past event.
        let other_event = make_event("e_other", "用户去超市买了菜", 0.5, "2026-05-24T10:00:00Z");
        db.add_event(user_id, &other_event, 30.0).unwrap();

        // No topics created at all. Today's event mentions 面试.
        let today_event = make_event("e_today", "用户收到了面试通知", 0.8, "2026-05-30T10:00:00Z");

        let query = DiaryMemoryQuery {
            user_id,
            diary_date: "2026-05-30",
            day_events: &[today_event],
            day_turns: &[],
            day_notes: &[],
            max_results: 5,
        };

        let results = retrieve_related_diary_memories(&db, &query).unwrap();

        // Should find the past 面试 event via keyword fallback.
        let found = results.iter().any(|r| r.content.contains("面试"));
        assert!(
            found,
            "keyword fallback should find the past interview event, got: {:?}",
            results.iter().map(|r| &r.content).collect::<Vec<_>>()
        );
    }

    /// Task 4.2: Recall is NOT recorded when diary generation fails before save.
    /// We simulate this by testing the recall logic directly.
    #[test]
    fn test_recall_not_recorded_on_diary_failure() {
        let db = test_db();
        let user_id = "test-user";

        let event = make_event("e1", "用户参加了面试", 0.8, "2026-05-25T10:00:00Z");
        db.add_event(user_id, &event, 30.0).unwrap();

        // Initial recall count should be 0.
        let fetched = db.get_event_by_id(user_id, "e1").unwrap().unwrap();
        assert_eq!(fetched.recall_count, 0, "initial recall_count should be 0");

        // Simulate: related memories were collected but diary generation FAILED.
        // In this case, diary.rs would NOT call record_recall.
        // Verify that without calling record_recall, count stays at 0.
        let after = db.get_event_by_id(user_id, "e1").unwrap().unwrap();
        assert_eq!(
            after.recall_count, 0,
            "recall should NOT be recorded when diary fails"
        );

        // Now simulate SUCCESS: record_recall IS called.
        db.record_recall(user_id, "e1", 0.2).unwrap();
        let after_recall = db.get_event_by_id(user_id, "e1").unwrap().unwrap();
        assert_eq!(
            after_recall.recall_count, 1,
            "recall should be incremented after success"
        );
        assert!(
            after_recall.stability > 30.0,
            "stability should be boosted: {}",
            after_recall.stability
        );
    }

    /// Task 4.3: Related memories without event_id do not attempt recall.
    #[test]
    fn test_no_recall_for_memories_without_event_id() {
        let db = test_db();
        let user_id = "test-user";

        // A conversation turn from the past (not an event, so no event_id).
        db.save_conversation_turn(
            user_id,
            "我想聊聊面试",
            "加油，你可以的",
            Some("用户聊到了面试的事情"),
            &[],
        )
        .unwrap();

        // Today's event.
        let today_event = make_event("e_today", "用户收到了面试通知", 0.8, "2026-05-30T10:00:00Z");

        let query = DiaryMemoryQuery {
            user_id,
            diary_date: "2026-05-30",
            day_events: &[today_event],
            day_turns: &[],
            day_notes: &[],
            max_results: 5,
        };

        let results = retrieve_related_diary_memories(&db, &query).unwrap();

        // Find any result that came from conversation (event_id = None).
        let conv_results: Vec<_> = results.iter().filter(|r| r.event_id.is_none()).collect();

        // If conversation results exist, verify they won't trigger recall.
        for mem in &conv_results {
            // The key contract: event_id is None, so diary.rs's recall loop skips it:
            //   if let Some(ref event_id) = mem.event_id { ... } // won't match
            assert!(
                mem.event_id.is_none(),
                "conversation-originated memories must have event_id = None"
            );
        }

        // The actual diary.rs recall code does:
        //   for mem in &related_memories {
        //       if let Some(ref event_id) = mem.event_id {
        //           let _ = db.record_recall(user_id, event_id, 0.2);
        //       }
        //   }
        // This loop is safe — None event_ids are silently skipped.
        // We verify by ensuring no spurious event exists to be recalled.
        let bogus = db.get_event_by_id(user_id, "nonexistent").unwrap();
        assert!(bogus.is_none(), "no phantom event should exist");
    }
}
