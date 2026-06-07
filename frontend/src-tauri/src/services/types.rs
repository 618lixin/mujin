use serde::{Deserialize, Serialize};

// ─── Event ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub content: String,
    #[serde(default)]
    pub emotions: Vec<String>,
    pub importance: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(default = "default_strength")]
    pub strength: f64,
    #[serde(default = "default_stability")]
    pub stability: f64,
    #[serde(default)]
    pub recall_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_recalled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Emotion Result ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EmotionResult {
    #[serde(default)]
    pub emotions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(default)]
    pub importance: f64,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub topics: Vec<String>,
}

impl EmotionResult {
    pub fn empty() -> Self {
        Self {
            emotions: vec![],
            event_type: None,
            importance: 0.0,
            summary: String::new(),
            topics: vec![],
        }
    }
}

// ─── Core Memory ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct CoreMemory {
    #[serde(default)]
    pub profile_content: String,
    #[serde(default)]
    pub notes_content: String,
    #[serde(default = "default_profile_max_chars")]
    pub profile_max_chars: u32,
    #[serde(default = "default_notes_max_chars")]
    pub notes_max_chars: u32,
}

impl CoreMemory {
    pub fn profile_usage(&self) -> usize {
        self.profile_content.len()
    }
    pub fn notes_usage(&self) -> usize {
        self.notes_content.len()
    }
    pub fn profile_pct(&self) -> f64 {
        if self.profile_max_chars == 0 {
            return 0.0;
        }
        self.profile_usage() as f64 / self.profile_max_chars as f64
    }
    pub fn notes_pct(&self) -> f64 {
        if self.notes_max_chars == 0 {
            return 0.0;
        }
        self.notes_usage() as f64 / self.notes_max_chars as f64
    }
    pub fn profile_near_limit(&self) -> bool {
        self.profile_pct() >= 0.8
    }
    pub fn notes_near_limit(&self) -> bool {
        self.notes_pct() >= 0.8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MemoryPatch {
    pub action: String, // "add" | "replace" | "remove"
    pub target: String, // "profile" | "notes"
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_text: Option<String>,
}

// ─── Chat Message ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatDaySummary {
    pub date: String,
    pub message_count: usize,
}

// ─── Conversation Turn ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationTurn {
    pub id: i64,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub emotions: Vec<String>,
    pub created_at: String,
}

// ─── Insight ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Insight {
    pub id: i64,
    pub category: String,
    pub content: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub source: String,
    pub created_at: String,
}

// ─── Observation ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub id: String,
    pub date: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub created_at: String,
}

// ─── Topic ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_mentioned: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_mentioned: Option<String>,
    #[serde(default)]
    pub mention_count: u32,
}

// ─── Topic Link ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TopicLink {
    pub topic_id: String,
    pub item_id: String,
    pub item_type: String,
}

// ─── Project ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_project_status")]
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(default)]
    pub event_ids: Vec<String>,
    #[serde(default)]
    pub summary: String,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Growth Line ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GrowthLine {
    pub id: String,
    pub dimension: String,
    #[serde(default)]
    pub records: Vec<serde_json::Value>,
}

// ─── Query Params ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueryEventsParams {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub min_importance: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(default)]
    pub min_strength: f64,
}

impl Default for QueryEventsParams {
    fn default() -> Self {
        Self {
            limit: 20,
            min_importance: 0.0,
            event_type: None,
            start_date: None,
            end_date: None,
            min_strength: 0.0,
        }
    }
}

// ─── Post Chat Result ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PostChatResult {
    pub emotion: EmotionResult,
    pub turn_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reflection: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes_update: Option<serde_json::Value>,
}

// ─── Pending Message ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PendingMessage {
    pub message: String,
    pub reason: String,
    pub created_at: String,
}

// ─── Stream Events ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamTokenEvent {
    pub stream_id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamDoneEvent {
    pub stream_id: String,
    pub meta: PostChatResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamErrorEvent {
    pub stream_id: String,
    pub error: String,
}

// ─── Core Memory Response ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoreMemoryResponse {
    pub profile: CoreMemoryStats,
    pub notes: CoreMemoryStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoreMemoryStats {
    pub content: String,
    pub chars: usize,
    pub max_chars: u32,
    pub pct: f64,
    pub near_limit: bool,
}

// ─── Diary ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiaryEntry {
    pub date: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiaryGenerateResult {
    pub date: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub source_event_count: usize,
    pub source_turn_count: usize,
    pub source_note_count: usize,
    pub regenerated: bool,
}

// ─── Weekly Growth Summary ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WeeklySummaryRequest {
    pub user_id: String,
    pub iso_year: i32,
    pub iso_week: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WeeklySourceCounts {
    pub diary_count: usize,
    pub event_count: usize,
    pub turn_count: usize,
    pub note_count: usize,
    pub observation_count: usize,
}

impl WeeklySourceCounts {
    pub fn total(&self) -> usize {
        self.diary_count
            + self.event_count
            + self.turn_count
            + self.note_count
            + self.observation_count
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WeeklySummaryEntry {
    pub iso_year: i32,
    pub iso_week: u32,
    pub week_display_range: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub source_counts: WeeklySourceCounts,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WeeklySummaryGenerateResult {
    pub iso_year: i32,
    pub iso_week: u32,
    pub week_display_range: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub source_counts: WeeklySourceCounts,
    pub regenerated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WeeklySummaryUpdateResult {
    pub iso_year: i32,
    pub iso_week: u32,
    pub week_display_range: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub updated_at: String,
}

// ─── Life Chapter ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LifeChapterRequest {
    pub user_id: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct LifeChapterSourceCounts {
    pub diary_count: usize,
    pub weekly_summary_count: usize,
    pub event_count: usize,
    pub turn_count: usize,
    pub note_count: usize,
    pub topic_count: usize,
    pub project_count: usize,
    pub growth_line_count: usize,
    pub observation_count: usize,
}

impl LifeChapterSourceCounts {
    pub fn total(&self) -> usize {
        self.diary_count
            + self.weekly_summary_count
            + self.event_count
            + self.turn_count
            + self.note_count
            + self.topic_count
            + self.project_count
            + self.growth_line_count
            + self.observation_count
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LifeChapterEntry {
    pub note_id: String,
    pub title: String,
    pub start_date: String,
    pub end_date: String,
    pub content: String,
    pub source_counts: LifeChapterSourceCounts,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LifeChapterGenerateResult {
    pub note_id: String,
    pub title: String,
    pub start_date: String,
    pub end_date: String,
    pub content: String,
    pub source_counts: LifeChapterSourceCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LifeChapterUpdateResult {
    pub note_id: String,
    pub title: String,
    pub start_date: String,
    pub end_date: String,
    pub content: String,
    pub updated_at: String,
}

// ─── Default Functions ────────────────────────────────────────────────────

pub fn default_strength() -> f64 {
    1.0
}
pub fn default_stability() -> f64 {
    30.0
}
pub fn default_profile_max_chars() -> u32 {
    1200
}
pub fn default_notes_max_chars() -> u32 {
    800
}
pub fn default_confidence() -> f64 {
    0.5
}
pub fn default_project_status() -> String {
    "active".to_string()
}
pub fn default_limit() -> usize {
    20
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serde_roundtrip() {
        let event = Event {
            id: "abc123".to_string(),
            content: "Test event".to_string(),
            emotions: vec!["joy".to_string(), "calm".to_string()],
            importance: 0.8,
            event_type: Some("milestone".to_string()),
            strength: 1.0,
            stability: 30.0,
            recall_count: 0,
            last_recalled_at: None,
            created_at: "2026-05-30T12:00:00Z".to_string(),
            updated_at: "2026-05-30T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"eventType\":"));
        let parsed: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_core_memory_usage() {
        let mem = CoreMemory {
            profile_content: "Hello world".to_string(),
            notes_content: "Test notes".to_string(),
            profile_max_chars: 100,
            notes_max_chars: 50,
        };
        assert_eq!(mem.profile_usage(), 11);
        assert!((mem.profile_pct() - 0.11).abs() < 0.001);
        assert!(!mem.profile_near_limit());
    }

    #[test]
    fn test_core_memory_near_limit() {
        let mem = CoreMemory {
            profile_content: "A".repeat(90),
            notes_content: String::new(),
            profile_max_chars: 100,
            notes_max_chars: 100,
        };
        assert!(mem.profile_near_limit());
    }

    #[test]
    fn test_emotion_result_empty() {
        let result = EmotionResult::empty();
        assert!(result.emotions.is_empty());
        assert!(result.event_type.is_none());
        assert_eq!(result.importance, 0.0);
    }

    #[test]
    fn test_query_events_params_default() {
        let params = QueryEventsParams::default();
        assert_eq!(params.limit, 20);
        assert_eq!(params.min_importance, 0.0);
    }

    #[test]
    fn test_stream_events_camel_case() {
        let event = StreamTokenEvent {
            stream_id: "test-123".to_string(),
            token: "hello".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"streamId\":"));
    }
}
