use std::path::Path;

use chrono::{Duration, Local, NaiveDate, TimeZone, Weekday};

use super::config::{load_ai_config, AiConfig};
use super::llm::call_llm;
use super::notes::{default_store, AppError, NoteStore, SaveNoteRequest};
use super::types::{
    ChatMessage, ConversationTurn, Event, Observation, WeeklySourceCounts, WeeklySummaryEntry,
    WeeklySummaryGenerateResult,
};

const WEEKLY_SUMMARY_CATEGORY: &str = "weekly-growth-summary";
const MAX_WEEKLY_EVENTS: usize = 100;
const MAX_WEEKLY_TURNS: usize = 200;
const WEEKLY_TEMPERATURE: f64 = 0.2;

const WEEKLY_GROUNDING_RULES: &str = r#"扎根规则：
- 只使用本 ISO 周内提供的记录。
- 全文必须使用中文输出。
- 不要编造事件、原因、对话、决定、结果、时间线、地点或关系状态。
- 如果来源材料稀疏，就写短一些，并明确说明本周记录有限。
- 可以使用定性观察，但不要出现数字人格权重或 Ti/Te/Fi/Fe/Si/Se/Ni/Ne 时间线。
- 不确定的地方要直接说明，不要用看似合理的故事补全空白。"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeeklySummaryIdentity {
    pub year: i32,
    pub week: u32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl WeeklySummaryIdentity {
    pub fn new(year: i32, week: u32) -> Result<Self, AppError> {
        let start_date = NaiveDate::from_isoywd_opt(year, week, Weekday::Mon).ok_or_else(|| {
            AppError::new(
                "invalidIsoWeek",
                format!("Invalid ISO week identity: {year}-W{week:02}"),
            )
        })?;
        let end_date = start_date + Duration::days(6);
        Ok(Self {
            year,
            week,
            start_date,
            end_date,
        })
    }

    pub fn note_title(&self) -> String {
        format!("week-{}-W{:02}", self.year, self.week)
    }

    pub fn week_display_range(&self) -> String {
        format!("{} ~ {}", self.start_date, self.end_date)
    }

    fn contains_date(&self, date: NaiveDate) -> bool {
        date >= self.start_date && date <= self.end_date
    }

    fn utc_range(&self) -> (String, String) {
        let local_start = Local
            .from_local_datetime(&self.start_date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap();
        let local_end = Local
            .from_local_datetime(&self.end_date.and_hms_opt(23, 59, 59).unwrap())
            .unwrap();
        (
            local_start
                .with_timezone(&chrono::Utc)
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string(),
            local_end
                .with_timezone(&chrono::Utc)
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeeklyNoteSource {
    pub title: String,
    pub content: String,
    pub category: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WeeklySummarySources {
    pub events: Vec<Event>,
    pub turns: Vec<ConversationTurn>,
    pub observations: Vec<Observation>,
    pub topics: Vec<String>,
    pub projects: Vec<String>,
    pub growth_lines: Vec<String>,
    pub notes: Vec<WeeklyNoteSource>,
    pub counts: WeeklySourceCounts,
}

impl WeeklySummarySources {
    pub fn from_parts(
        events: Vec<Event>,
        turns: Vec<ConversationTurn>,
        observations: Vec<Observation>,
        topics: Vec<String>,
        projects: Vec<String>,
        growth_lines: Vec<String>,
        notes: Vec<WeeklyNoteSource>,
    ) -> Self {
        let diary_count = notes.iter().filter(|note| note.category == "diary").count();
        let note_count = notes.len().saturating_sub(diary_count);
        let counts = WeeklySourceCounts {
            diary_count,
            event_count: events.len(),
            turn_count: turns.len(),
            note_count,
            observation_count: observations.len(),
        };
        Self {
            events,
            turns,
            observations,
            topics,
            projects,
            growth_lines,
            notes,
            counts,
        }
    }

    pub fn has_material(&self) -> bool {
        self.counts.total() > 0
            || !self.topics.is_empty()
            || !self.projects.is_empty()
            || !self.growth_lines.is_empty()
    }
}

pub fn collect_notes_for_week(
    store: &NoteStore,
    identity: &WeeklySummaryIdentity,
) -> Result<Vec<WeeklyNoteSource>, AppError> {
    let mut sources = Vec::new();
    for meta in store.list_notes()? {
        if meta.category == WEEKLY_SUMMARY_CATEGORY {
            continue;
        }

        let date = if meta.category == "diary" {
            NaiveDate::parse_from_str(&meta.title, "%Y-%m-%d").ok()
        } else {
            Some(meta.created_at.date_naive())
        };

        if date.is_some_and(|date| identity.contains_date(date)) {
            let note = store.read_note(&meta.id)?;
            sources.push(WeeklyNoteSource {
                title: meta.title,
                content: note.content,
                category: meta.category,
            });
        }
    }
    Ok(sources)
}

pub fn build_sparse_weekly_summary(
    identity: &WeeklySummaryIdentity,
    sources: &WeeklySummarySources,
) -> WeeklySummaryGenerateResult {
    WeeklySummaryGenerateResult {
        iso_year: identity.year,
        iso_week: identity.week,
        week_display_range: identity.week_display_range(),
        note_id: String::new(),
        title: identity.note_title(),
        content: format!(
            "# {}\n\n本周可用记录较少，暂时只能生成一份很短的成长总结；为避免编造细节，这里只保留已被记录支持的信息。",
            identity.note_title()
        ),
        source_counts: sources.counts.clone(),
        regenerated: false,
    }
}

pub fn build_weekly_summary_prompt(
    identity: &WeeklySummaryIdentity,
    sources: &WeeklySummarySources,
) -> String {
    if !sources.has_material() {
        return String::new();
    }

    let mut sections = vec![
        format!(
            "你是一个克制、准确、自然的成长日记整理助手。请根据以下记录，为 ISO 周 {}-W{:02}（{}）生成一份中文周成长总结。",
            identity.year,
            identity.week,
            identity.week_display_range()
        ),
        "写作风格要求（优先级很高）：
- 使用 Markdown，标题以本周范围或 week 标识开头。
- 语气要像用户自己在周末回看这一周，不要像 AI 资料汇总。
- 不要写成工作报告、复盘模板、事实核查或来源审计。
- 正文里不要反复出现“根据记录”“材料显示”“来源显示”“对话里”“笔记里”“事件记忆里”。
- 可以做温和整理和谨慎连接，但每个具体事实都必须能被下面的记录支持。
- 如果材料很少，宁可短一点；不要为了显得完整而补充细节。
- 只输出周总结成品，不输出分析过程、证据判断或写作说明。"
            .to_string(),
        WEEKLY_GROUNDING_RULES.to_string(),
        format!(
            "来源数量：日记={}，事件记忆={}，对话摘要={}，笔记={}，定性观察={}",
            sources.counts.diary_count,
            sources.counts.event_count,
            sources.counts.turn_count,
            sources.counts.note_count,
            sources.counts.observation_count
        ),
    ];

    if !sources.notes.is_empty() {
        sections.push(format!(
            "笔记与日记：\n{}",
            sources
                .notes
                .iter()
                .enumerate()
                .map(|(index, note)| format!(
                    "{}. [{}] {}\n{}",
                    index + 1,
                    note.category,
                    note.title,
                    truncate(&note.content, 800)
                ))
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
    }

    if !sources.events.is_empty() {
        sections.push(format!(
            "事件记忆：\n{}",
            sources
                .events
                .iter()
                .enumerate()
                .map(|(index, event)| format!(
                    "{}. {} ({})",
                    index + 1,
                    event.content,
                    event.created_at
                ))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    if !sources.turns.is_empty() {
        sections.push(format!(
            "对话摘要：\n{}",
            sources
                .turns
                .iter()
                .enumerate()
                .map(|(index, turn)| format!(
                    "{}. {} ({})",
                    index + 1,
                    turn.summary,
                    turn.created_at
                ))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    if !sources.observations.is_empty() {
        sections.push(format!(
            "定性观察：\n{}",
            sources
                .observations
                .iter()
                .enumerate()
                .map(|(index, obs)| format!("{}. [{}] {}", index + 1, obs.date, obs.content))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    sections.push(
        "请直接输出中文 Markdown 周总结正文。宁可短而真实，也不要写得丰富但缺乏依据。"
            .to_string(),
    );
    sections.join("\n\n")
}

pub fn save_weekly_summary_note(
    store: &NoteStore,
    identity: &WeeklySummaryIdentity,
    content: &str,
    source_counts: WeeklySourceCounts,
    regenerated: bool,
) -> Result<WeeklySummaryGenerateResult, AppError> {
    let _ = store.create_category(WEEKLY_SUMMARY_CATEGORY);
    let title = identity.note_title();
    let existing_id = store
        .list_notes()?
        .into_iter()
        .find(|note| note.category == WEEKLY_SUMMARY_CATEGORY && note.title == title)
        .map(|note| note.id);

    let request = SaveNoteRequest {
        title: title.clone(),
        content: content.to_string(),
        category: WEEKLY_SUMMARY_CATEGORY.to_string(),
    };
    let note = if let Some(id) = existing_id {
        store.update_note(&id, request)?
    } else {
        store.create_note(request)?
    };

    Ok(WeeklySummaryGenerateResult {
        iso_year: identity.year,
        iso_week: identity.week,
        week_display_range: identity.week_display_range(),
        note_id: note.id,
        title,
        content: note.content,
        source_counts,
        regenerated,
    })
}

pub fn list_weekly_summaries(store: &NoteStore) -> Result<Vec<WeeklySummaryEntry>, AppError> {
    let mut entries = Vec::new();
    for meta in store.list_notes()? {
        if meta.category != WEEKLY_SUMMARY_CATEGORY {
            continue;
        }
        let Some((iso_year, iso_week)) = parse_week_title(&meta.title) else {
            continue;
        };
        let identity = WeeklySummaryIdentity::new(iso_year, iso_week)?;
        let note = store.read_note(&meta.id)?;
        entries.push(WeeklySummaryEntry {
            iso_year,
            iso_week,
            week_display_range: identity.week_display_range(),
            note_id: note.id,
            title: meta.title,
            content: note.content,
            source_counts: WeeklySourceCounts::default(),
            created_at: meta.created_at.to_rfc3339(),
            updated_at: meta.updated_at.to_rfc3339(),
        });
    }
    entries.sort_by_key(|entry| std::cmp::Reverse((entry.iso_year, entry.iso_week)));
    Ok(entries)
}

pub async fn generate_weekly_summary(
    base_dir: &Path,
    db: &super::database::DbState,
    client: &reqwest::Client,
    user_id: &str,
    iso_year: i32,
    iso_week: u32,
) -> Result<WeeklySummaryGenerateResult, AppError> {
    generate_weekly_summary_inner(base_dir, db, client, user_id, iso_year, iso_week, false).await
}

pub async fn regenerate_weekly_summary(
    base_dir: &Path,
    db: &super::database::DbState,
    client: &reqwest::Client,
    user_id: &str,
    iso_year: i32,
    iso_week: u32,
) -> Result<WeeklySummaryGenerateResult, AppError> {
    generate_weekly_summary_inner(base_dir, db, client, user_id, iso_year, iso_week, true).await
}

async fn generate_weekly_summary_inner(
    base_dir: &Path,
    db: &super::database::DbState,
    client: &reqwest::Client,
    user_id: &str,
    iso_year: i32,
    iso_week: u32,
    regenerated: bool,
) -> Result<WeeklySummaryGenerateResult, AppError> {
    let identity = WeeklySummaryIdentity::new(iso_year, iso_week)?;
    let config = load_ai_config(base_dir)?;
    let store = default_store()?;
    let sources = gather_weekly_sources(db, &store, user_id, &identity)?;

    let content = if sources.has_material() {
        generate_weekly_markdown(client, &config, &identity, &sources).await?
    } else {
        build_sparse_weekly_summary(&identity, &sources).content
    };

    save_weekly_summary_note(&store, &identity, &content, sources.counts, regenerated)
}

pub fn gather_weekly_sources(
    db: &super::database::DbState,
    store: &NoteStore,
    user_id: &str,
    identity: &WeeklySummaryIdentity,
) -> Result<WeeklySummarySources, AppError> {
    let (utc_start, utc_end) = identity.utc_range();
    let events = db.query_events_by_date(user_id, &utc_start, &utc_end, MAX_WEEKLY_EVENTS)?;
    let turns =
        db.query_conversation_turns_by_date(user_id, &utc_start, &utc_end, MAX_WEEKLY_TURNS)?;
    let observations = db
        .query_observations(user_id, None, 500)?
        .into_iter()
        .filter(|obs| {
            NaiveDate::parse_from_str(&obs.date, "%Y-%m-%d")
                .map(|date| identity.contains_date(date))
                .unwrap_or(false)
        })
        .collect();
    let notes = collect_notes_for_week(store, identity)?;
    Ok(WeeklySummarySources::from_parts(
        events,
        turns,
        observations,
        vec![],
        vec![],
        vec![],
        notes,
    ))
}

async fn generate_weekly_markdown(
    client: &reqwest::Client,
    config: &AiConfig,
    identity: &WeeklySummaryIdentity,
    sources: &WeeklySummarySources,
) -> Result<String, AppError> {
    let prompt = build_weekly_summary_prompt(identity, sources);
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: WEEKLY_GROUNDING_RULES.to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: prompt,
        },
    ];
    let output = call_llm(client, config, &messages, None, WEEKLY_TEMPERATURE, 2000).await?;
    let trimmed = output.trim();
    if trimmed.starts_with("# ") {
        Ok(trimmed.to_string())
    } else {
        Ok(format!("# {}\n\n{}", identity.note_title(), trimmed))
    }
}

fn parse_week_title(title: &str) -> Option<(i32, u32)> {
    let rest = title.strip_prefix("week-")?;
    let (year, week) = rest.split_once("-W")?;
    Some((year.parse().ok()?, week.parse().ok()?))
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut result: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        result.push_str("...");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::notes::{NoteStore, SaveNoteRequest};
    use crate::services::types::{ConversationTurn, Event, Observation};
    use chrono::Datelike;

    fn temp_store(name: &str) -> NoteStore {
        let root = std::env::temp_dir().join(format!("gc_weekly_summary_{name}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp store");
        NoteStore::new(root)
    }

    fn event(id: &str, content: &str, created_at: &str) -> Event {
        Event {
            id: id.to_string(),
            content: content.to_string(),
            emotions: vec!["calm".to_string()],
            importance: 0.7,
            event_type: Some("milestone".to_string()),
            strength: 1.0,
            stability: 30.0,
            recall_count: 0,
            last_recalled_at: None,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
        }
    }

    fn turn(id: i64, summary: &str, created_at: &str) -> ConversationTurn {
        ConversationTurn {
            id,
            summary: summary.to_string(),
            emotions: vec![],
            created_at: created_at.to_string(),
        }
    }

    fn observation(id: &str, date: &str, content: &str) -> Observation {
        Observation {
            id: id.to_string(),
            date: date.to_string(),
            content: content.to_string(),
            category: Some("growth".to_string()),
            source: Some("reflection".to_string()),
            created_at: format!("{date}T10:00:00Z"),
        }
    }

    #[test]
    fn computes_iso_week_identity_and_display_range() {
        let identity = WeeklySummaryIdentity::new(2026, 23).expect("valid week");

        assert_eq!(identity.year, 2026);
        assert_eq!(identity.week, 23);
        assert_eq!(identity.note_title(), "week-2026-W23");
        assert_eq!(identity.week_display_range(), "2026-06-01 ~ 2026-06-07");
        assert_eq!(identity.start_date.weekday(), chrono::Weekday::Mon);
        assert_eq!(identity.end_date.weekday(), chrono::Weekday::Sun);
    }

    #[test]
    fn aggregates_weekly_sources_and_counts_by_type() {
        let store = temp_store("aggregate");
        store.create_category("diary").unwrap();
        store.create_note(SaveNoteRequest {
            title: "2026-06-02".to_string(),
            content: "# 2026-06-02\n\nDiary body".to_string(),
            category: "diary".to_string(),
        }).unwrap();
        store.create_note(SaveNoteRequest {
            title: "Loose note".to_string(),
            content: "A note from the same week".to_string(),
            category: String::new(),
        }).unwrap();

        let sources = WeeklySummarySources::from_parts(
            vec![event("e1", "Finished a difficult task", "2026-06-03T09:00:00Z")],
            vec![turn(1, "Talked about recovery", "2026-06-04T09:00:00Z")],
            vec![observation("o1", "2026-06-05", "More patient under pressure")],
            vec![],
            vec![],
            vec![],
            collect_notes_for_week(&store, &WeeklySummaryIdentity::new(2026, 23).unwrap()).unwrap(),
        );

        assert_eq!(sources.counts.diary_count, 1);
        assert_eq!(sources.counts.event_count, 1);
        assert_eq!(sources.counts.turn_count, 1);
        assert_eq!(sources.counts.note_count, 1);
        assert_eq!(sources.counts.observation_count, 1);
        assert!(sources.has_material());
    }

    #[test]
    fn sparse_week_uses_empty_state_without_inventing_events() {
        let identity = WeeklySummaryIdentity::new(2026, 23).unwrap();
        let sources = WeeklySummarySources::default();
        let result = build_sparse_weekly_summary(&identity, &sources);

        assert!(result.content.contains("可用记录较少"));
        assert_eq!(result.source_counts.total(), 0);
        assert!(!result.content.contains("dialogue"));
        assert!(!result.content.contains("decision"));
    }

    #[test]
    fn weekly_prompt_includes_counts_and_grounding_rules() {
        let identity = WeeklySummaryIdentity::new(2026, 23).unwrap();
        let sources = WeeklySummarySources::from_parts(
            vec![event("e1", "Finished a difficult task", "2026-06-03T09:00:00Z")],
            vec![],
            vec![observation("o1", "2026-06-05", "More patient under pressure")],
            vec![],
            vec![],
            vec![],
            vec![],
        );

        let prompt = build_weekly_summary_prompt(&identity, &sources);

        assert!(prompt.contains("来源数量"));
        assert!(prompt.contains("事件记忆=1"));
        assert!(prompt.contains("定性观察=1"));
        assert!(prompt.contains("不要编造事件"));
        assert!(prompt.contains("数字人格权重"));
        assert!(prompt.contains("全文必须使用中文输出"));
        assert!(prompt.contains("不要像 AI 资料汇总"));
        assert!(prompt.contains("只输出周总结成品"));
        assert!(prompt.contains("2026-06-01 ~ 2026-06-07"));
    }

    #[test]
    fn same_week_regeneration_replaces_existing_note() {
        let store = temp_store("regenerate");
        let identity = WeeklySummaryIdentity::new(2026, 23).unwrap();

        let first = save_weekly_summary_note(
            &store,
            &identity,
            "first body",
            WeeklySourceCounts::default(),
            false,
        ).unwrap();
        let second = save_weekly_summary_note(
            &store,
            &identity,
            "second body",
            WeeklySourceCounts::default(),
            true,
        ).unwrap();

        assert_eq!(first.note_id, second.note_id);
        assert_eq!(store.list_notes().unwrap().len(), 1);
        assert_eq!(store.read_note(&first.note_id).unwrap().content, "second body");
    }

    #[test]
    fn rejects_invalid_iso_week() {
        assert!(WeeklySummaryIdentity::new(2026, 54).is_err());
    }

    #[test]
    fn list_weekly_summaries_orders_by_week_descending() {
        let store = temp_store("list");
        let first = WeeklySummaryIdentity::new(2026, 22).unwrap();
        let second = WeeklySummaryIdentity::new(2026, 23).unwrap();
        save_weekly_summary_note(&store, &first, "first", WeeklySourceCounts::default(), false)
            .unwrap();
        save_weekly_summary_note(&store, &second, "second", WeeklySourceCounts::default(), false)
            .unwrap();

        let entries = list_weekly_summaries(&store).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].iso_week, 23);
        assert_eq!(entries[1].iso_week, 22);
    }
}
