use std::path::Path;

use chrono::{Datelike, Local, NaiveDate, TimeZone};

use super::config::{load_ai_config, AiConfig};
use super::llm::call_llm;
use super::notes::{default_store, AppError, NoteStore, SaveNoteRequest};
use super::types::{
    ChatMessage, ConversationTurn, Event, GrowthLine, LifeChapterEntry, LifeChapterGenerateResult,
    LifeChapterSourceCounts, LifeChapterUpdateResult, Observation, Project, Topic,
};

const LIFE_CHAPTER_CATEGORY: &str = "life-chapter";
const LIFE_CHAPTER_TEMPERATURE: f64 = 0.25;
const MAX_CHAPTER_EVENTS: usize = 160;
const MAX_CHAPTER_TURNS: usize = 260;
const DEFAULT_CHAPTER_SOURCE_LIMIT: usize = 60;

const LIFE_CHAPTER_GROUNDING_RULES: &str = r#"扎根规则：
- 只使用所选日期范围内提供的记录。
- 全文必须使用中文输出。
- 不要编造缺失的原因、对话、决定、结果、时间线、地点或关系状态。
- 如果来源材料稀疏，就写成简短章节，并明确说明记录有限。
- 优先写有日期范围支撑的变化模式，不要制造缺乏依据的故事弧线。
- 不要出现数字人格权重或 Ti/Te/Fi/Fe/Si/Se/Ni/Ne 时间线。"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifeChapterRange {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl LifeChapterRange {
    pub fn new(start_date: &str, end_date: &str) -> Result<Self, AppError> {
        let start_date = parse_date(start_date)?;
        let end_date = parse_date(end_date)?;
        if end_date < start_date {
            return Err(AppError::new(
                "invalidDateRange",
                "end date must be on or after start date",
            ));
        }
        Ok(Self {
            start_date,
            end_date,
        })
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

    fn label(&self) -> String {
        format!("{} ~ {}", self.start_date, self.end_date)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifeChapterNoteSource {
    pub title: String,
    pub content: String,
    pub category: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LifeChapterSources {
    pub events: Vec<Event>,
    pub turns: Vec<ConversationTurn>,
    pub observations: Vec<Observation>,
    pub topics: Vec<Topic>,
    pub projects: Vec<Project>,
    pub growth_lines: Vec<GrowthLine>,
    pub notes: Vec<LifeChapterNoteSource>,
    pub counts: LifeChapterSourceCounts,
}

impl LifeChapterSources {
    pub fn filter_to_range(&mut self, range: &LifeChapterRange) {
        self.topics
            .retain(|topic| topic_overlaps_range(topic, range));
        self.projects
            .retain(|project| project_overlaps_range(project, range));
        self.growth_lines = self
            .growth_lines
            .drain(..)
            .filter_map(|mut line| {
                line.records
                    .retain(|record| record_date_in_range(record, range));
                if line.records.is_empty() {
                    None
                } else {
                    Some(line)
                }
            })
            .collect();
        self.refresh_counts();
    }

    pub fn refresh_counts(&mut self) {
        let diary_count = self
            .notes
            .iter()
            .filter(|note| note.category == "diary")
            .count();
        let weekly_summary_count = self
            .notes
            .iter()
            .filter(|note| note.category == "weekly-growth-summary")
            .count();
        let note_count = self
            .notes
            .len()
            .saturating_sub(diary_count + weekly_summary_count);
        self.counts = LifeChapterSourceCounts {
            diary_count,
            weekly_summary_count,
            event_count: self.events.len(),
            turn_count: self.turns.len(),
            note_count,
            topic_count: self.topics.len(),
            project_count: self.projects.len(),
            growth_line_count: self.growth_lines.len(),
            observation_count: self.observations.len(),
        };
    }

    pub fn has_material(&self) -> bool {
        self.counts.total() > 0
    }
}

pub fn build_sparse_life_chapter(
    range: &LifeChapterRange,
    sources: &LifeChapterSources,
) -> LifeChapterGenerateResult {
    let title = generate_chapter_title(range, sources);
    LifeChapterGenerateResult {
        note_id: String::new(),
        title: title.clone(),
        start_date: range.start_date.to_string(),
        end_date: range.end_date.to_string(),
        content: format!(
            "# {title}\n\n{} 这段时间的可用记录较少。为了避免补写没有依据的经历，这一章会保持简短，只保留已有记录能够支撑的内容。",
            range.label()
        ),
        source_counts: sources.counts.clone(),
    }
}

pub fn limit_chapter_sources(
    mut sources: LifeChapterSources,
    max_each: usize,
) -> LifeChapterSources {
    sources.events.truncate(max_each);
    sources.turns.truncate(max_each);
    sources.observations.truncate(max_each);
    sources.topics.truncate(max_each);
    sources.projects.truncate(max_each);
    sources.growth_lines.truncate(max_each);
    sources.notes.truncate(max_each);
    sources.refresh_counts();
    sources
}

pub fn generate_chapter_title(range: &LifeChapterRange, sources: &LifeChapterSources) -> String {
    let time_label = if range.start_date.year() == range.end_date.year() {
        format!(
            "{} {}-{}",
            range.start_date.year(),
            range.start_date.month(),
            range.end_date.month()
        )
    } else {
        format!("{} to {}", range.start_date, range.end_date)
    };
    let theme = sources
        .observations
        .first()
        .map(|obs| obs.content.as_str())
        .or_else(|| sources.events.first().map(|event| event.content.as_str()))
        .or_else(|| sources.notes.first().map(|note| note.title.as_str()))
        .unwrap_or("记录有限");
    format!("{time_label}: {}", title_fragment(theme))
}

pub fn build_life_chapter_prompt(range: &LifeChapterRange, sources: &LifeChapterSources) -> String {
    if !sources.has_material() {
        return String::new();
    }

    let mut sections = vec![
        format!(
            "你是一个克制、准确、自然的成长日记整理助手。请根据以下记录，为 {} 生成一章中文人生章节，并生成一个有反思感的中文标题。",
            range.label()
        ),
        "写作风格要求（优先级很高）：
- 使用 Markdown，标题格式参考《时间标签：一句描述变化的话》。
- 语气要像用户在回看自己经历过的一段时期，不要像 AI 资料汇总。
- 不要写成研究报告、项目复盘、事实核查或来源审计。
- 正文里不要反复出现“根据记录”“材料显示”“来源显示”“对话里”“笔记里”“事件记忆里”。
- 可以串联长期模式、情绪变化、主题反复出现的地方，但不要把没有依据的联系写成确定事实。
- 如果材料很少，宁可写短一点；不要为了形成完整人生故事而补充不存在的细节。
- 只输出章节成品，不输出分析过程、证据判断或写作说明。"
            .to_string(),
        LIFE_CHAPTER_GROUNDING_RULES.to_string(),
        format!(
            "来源数量：日记={}，周总结={}，事件记忆={}，对话摘要={}，笔记={}，主题={}，项目={}，成长线={}，定性观察={}",
            sources.counts.diary_count,
            sources.counts.weekly_summary_count,
            sources.counts.event_count,
            sources.counts.turn_count,
            sources.counts.note_count,
            sources.counts.topic_count,
            sources.counts.project_count,
            sources.counts.growth_line_count,
            sources.counts.observation_count
        ),
    ];

    if !sources.notes.is_empty() {
        sections.push(format!(
            "笔记、日记与周总结：\n{}",
            sources
                .notes
                .iter()
                .enumerate()
                .map(|(index, note)| format!(
                    "{}. [{}] {}\n{}",
                    index + 1,
                    note.category,
                    note.title,
                    truncate(&note.content, 1000)
                ))
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
    }
    if !sources.events.is_empty() {
        sections.push(format_items(
            "事件记忆",
            sources
                .events
                .iter()
                .map(|event| format!("{} ({})", event.content, event.created_at)),
        ));
    }
    if !sources.turns.is_empty() {
        sections.push(format_items(
            "对话摘要",
            sources
                .turns
                .iter()
                .map(|turn| format!("{} ({})", turn.summary, turn.created_at)),
        ));
    }
    if !sources.observations.is_empty() {
        sections.push(format_items(
            "定性观察",
            sources
                .observations
                .iter()
                .map(|obs| format!("[{}] {}", obs.date, obs.content)),
        ));
    }
    if !sources.topics.is_empty() {
        sections.push(format_items(
            "主题",
            sources.topics.iter().map(|topic| {
                format!(
                    "{}：{}；首次={}；最近={}；次数={}",
                    topic.name,
                    truncate(&topic.description, 240),
                    topic.first_mentioned.as_deref().unwrap_or("未知"),
                    topic.last_mentioned.as_deref().unwrap_or("未知"),
                    topic.mention_count
                )
            }),
        ));
    }
    if !sources.projects.is_empty() {
        sections.push(format_items(
            "项目",
            sources.projects.iter().map(|project| {
                format!(
                    "{} [{}]：{}；范围={} ~ {}",
                    project.title,
                    project.status,
                    truncate(&project.summary, 240),
                    project.start_date.as_deref().unwrap_or("未知"),
                    project.end_date.as_deref().unwrap_or("未知")
                )
            }),
        ));
    }
    if !sources.growth_lines.is_empty() {
        sections.push(format_items(
            "成长线",
            sources.growth_lines.iter().map(|line| {
                format!(
                    "{}：{}",
                    line.dimension,
                    truncate(
                        &serde_json::Value::Array(line.records.clone()).to_string(),
                        480
                    )
                )
            }),
        ));
    }

    sections.push(
        "请直接输出中文 Markdown 人生章节正文。宁可短而真实，也不要写得完整但缺乏依据。"
            .to_string(),
    );
    sections.join("\n\n")
}

pub fn save_life_chapter_note(
    store: &NoteStore,
    range: &LifeChapterRange,
    title: &str,
    content: &str,
    source_counts: LifeChapterSourceCounts,
) -> Result<LifeChapterGenerateResult, AppError> {
    let _ = store.create_category(LIFE_CHAPTER_CATEGORY);
    let note = store.create_note(SaveNoteRequest {
        title: title.to_string(),
        content: content.to_string(),
        category: LIFE_CHAPTER_CATEGORY.to_string(),
    })?;
    Ok(LifeChapterGenerateResult {
        note_id: note.id,
        title: note.title,
        start_date: range.start_date.to_string(),
        end_date: range.end_date.to_string(),
        content: strip_range_metadata(&note.content).to_string(),
        source_counts,
    })
}

pub fn list_life_chapters(store: &NoteStore) -> Result<Vec<LifeChapterEntry>, AppError> {
    let mut entries = Vec::new();
    for meta in store.list_notes()? {
        if meta.category != LIFE_CHAPTER_CATEGORY {
            continue;
        }
        let note = store.read_note(&meta.id)?;
        let (start_date, end_date) = parse_chapter_range_from_content(&note.content)
            .unwrap_or_else(|| (String::new(), String::new()));
        entries.push(LifeChapterEntry {
            note_id: note.id,
            title: note.title,
            start_date,
            end_date,
            content: strip_range_metadata(&note.content).to_string(),
            source_counts: LifeChapterSourceCounts::default(),
            created_at: meta.created_at.to_rfc3339(),
            updated_at: meta.updated_at.to_rfc3339(),
        });
    }
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.updated_at.clone()));
    Ok(entries)
}

pub fn update_life_chapter(
    store: &NoteStore,
    note_id: &str,
    title: String,
    content: String,
) -> Result<LifeChapterUpdateResult, AppError> {
    let existing = store.read_note(note_id)?;
    if existing.category != LIFE_CHAPTER_CATEGORY {
        return Err(AppError::new(
            "lifeChapterNotFound",
            format!("Life chapter {note_id} was not found"),
        ));
    }
    let (start_date, end_date) =
        parse_chapter_range_from_content(&existing.content).ok_or_else(|| {
            AppError::new(
                "lifeChapterMetadataMissing",
                format!("Life chapter {note_id} is missing date-range metadata"),
            )
        })?;
    let range = LifeChapterRange::new(&start_date, &end_date)?;
    let title = title.trim().to_string();
    let title = if title.is_empty() {
        existing.title
    } else {
        title
    };
    let normalized = normalize_chapter_heading(&title, &content);
    let stored_content = with_range_metadata(&range, &normalized);
    let note = store.update_note(
        note_id,
        SaveNoteRequest {
            title: title.clone(),
            content: stored_content,
            category: LIFE_CHAPTER_CATEGORY.to_string(),
        },
    )?;

    Ok(LifeChapterUpdateResult {
        note_id: note.id,
        title: note.title,
        start_date,
        end_date,
        content: strip_range_metadata(&note.content).to_string(),
        updated_at: note.updated_at.to_rfc3339(),
    })
}

pub async fn generate_life_chapter(
    base_dir: &Path,
    db: &super::database::DbState,
    client: &reqwest::Client,
    user_id: &str,
    start_date: String,
    end_date: String,
    title: Option<String>,
) -> Result<LifeChapterGenerateResult, AppError> {
    let range = LifeChapterRange::new(&start_date, &end_date)?;
    let config = load_ai_config(base_dir)?;
    let store = default_store()?;
    let sources = limit_chapter_sources(
        gather_life_chapter_sources(db, &store, user_id, &range)?,
        DEFAULT_CHAPTER_SOURCE_LIMIT,
    );
    let title = title
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| generate_chapter_title(&range, &sources));
    let content = if sources.has_material() {
        generate_life_chapter_markdown(client, &config, &range, &sources, &title).await?
    } else {
        build_sparse_life_chapter_with_title(&range, &sources, &title).content
    };
    let content = with_range_metadata(&range, &content);
    save_life_chapter_note(&store, &range, &title, &content, sources.counts)
}

pub fn gather_life_chapter_sources(
    db: &super::database::DbState,
    store: &NoteStore,
    user_id: &str,
    range: &LifeChapterRange,
) -> Result<LifeChapterSources, AppError> {
    let (utc_start, utc_end) = range.utc_range();
    let mut sources = LifeChapterSources {
        events: db.query_events_by_date(user_id, &utc_start, &utc_end, MAX_CHAPTER_EVENTS)?,
        turns: db.query_conversation_turns_by_date(
            user_id,
            &utc_start,
            &utc_end,
            MAX_CHAPTER_TURNS,
        )?,
        observations: db
            .query_observations(user_id, None, 1000)?
            .into_iter()
            .filter(|obs| {
                NaiveDate::parse_from_str(&obs.date, "%Y-%m-%d")
                    .map(|date| range.contains_date(date))
                    .unwrap_or(false)
            })
            .collect(),
        topics: db.query_topics(user_id, 200)?,
        projects: db.query_projects(user_id, None, 200)?,
        growth_lines: db.query_growth_lines(user_id, 200)?,
        notes: collect_notes_for_range(store, range)?,
        counts: LifeChapterSourceCounts::default(),
    };
    sources.filter_to_range(range);
    sources.refresh_counts();
    Ok(sources)
}

async fn generate_life_chapter_markdown(
    client: &reqwest::Client,
    config: &AiConfig,
    range: &LifeChapterRange,
    sources: &LifeChapterSources,
    title: &str,
) -> Result<String, AppError> {
    let prompt = build_life_chapter_prompt(range, sources);
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: LIFE_CHAPTER_GROUNDING_RULES.to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: prompt,
        },
    ];
    let output = call_llm(
        client,
        config,
        &messages,
        None,
        LIFE_CHAPTER_TEMPERATURE,
        3000,
    )
    .await?;
    let trimmed = output.trim();
    Ok(normalize_chapter_heading(title, trimmed))
}

fn collect_notes_for_range(
    store: &NoteStore,
    range: &LifeChapterRange,
) -> Result<Vec<LifeChapterNoteSource>, AppError> {
    let mut sources = Vec::new();
    for meta in store.list_notes()? {
        if meta.category == LIFE_CHAPTER_CATEGORY {
            continue;
        }
        let date = if meta.category == "diary" {
            NaiveDate::parse_from_str(&meta.title, "%Y-%m-%d").ok()
        } else {
            Some(meta.created_at.date_naive())
        };
        if date.is_some_and(|date| range.contains_date(date)) {
            let note = store.read_note(&meta.id)?;
            sources.push(LifeChapterNoteSource {
                title: meta.title,
                content: note.content,
                category: meta.category,
            });
        }
    }
    Ok(sources)
}

fn parse_date(value: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::new("invalidDate", format!("Invalid date: {value}")))
}

fn title_fragment(value: &str) -> String {
    let fragment = value
        .split_whitespace()
        .take(8)
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|ch: char| matches!(ch, '.' | ',' | ';' | '。' | '，' | '；'))
        .to_string();
    if fragment.is_empty() {
        "记录有限".to_string()
    } else {
        truncate(&fragment, 28)
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut result: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        result.push_str("...");
    }
    result
}

fn format_items(label: &str, items: impl Iterator<Item = String>) -> String {
    format!(
        "{label}:\n{}",
        items
            .enumerate()
            .map(|(index, item)| format!("{}. {}", index + 1, item))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn with_range_metadata(range: &LifeChapterRange, content: &str) -> String {
    format!(
        "---\nstart_date: {}\nend_date: {}\n---\n\n{}",
        range.start_date,
        range.end_date,
        content.trim()
    )
}

fn parse_chapter_range_from_content(content: &str) -> Option<(String, String)> {
    let start_date = content
        .lines()
        .find_map(|line| line.strip_prefix("start_date: ").map(str::trim))?;
    let end_date = content
        .lines()
        .find_map(|line| line.strip_prefix("end_date: ").map(str::trim))?;
    Some((start_date.to_string(), end_date.to_string()))
}

fn strip_range_metadata(content: &str) -> &str {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---\n") && !trimmed.starts_with("---\r\n") {
        return content;
    }

    let after_open = trimmed
        .strip_prefix("---\r\n")
        .or_else(|| trimmed.strip_prefix("---\n"))
        .unwrap_or(trimmed);
    if let Some(rest) = after_open
        .strip_prefix("start_date: ")
        .and_then(|_| after_open.split_once("\n---\n").map(|(_, rest)| rest))
        .or_else(|| {
            after_open
                .strip_prefix("start_date: ")
                .and_then(|_| after_open.split_once("\r\n---\r\n").map(|(_, rest)| rest))
        })
    {
        rest.trim_start()
    } else {
        content
    }
}

fn build_sparse_life_chapter_with_title(
    range: &LifeChapterRange,
    sources: &LifeChapterSources,
    title: &str,
) -> LifeChapterGenerateResult {
    LifeChapterGenerateResult {
        note_id: String::new(),
        title: title.to_string(),
        start_date: range.start_date.to_string(),
        end_date: range.end_date.to_string(),
        content: format!(
            "# {title}\n\n{} 这段时间的可用记录较少。为了避免补写没有依据的经历，这一章会保持简短，只保留已有记录能够支撑的内容。",
            range.label()
        ),
        source_counts: sources.counts.clone(),
    }
}

fn normalize_chapter_heading(title: &str, content: &str) -> String {
    let trimmed = content.trim();
    if let Some(rest) = trimmed.strip_prefix("# ") {
        if let Some((_, body)) = rest.split_once('\n') {
            format!("# {title}\n\n{}", body.trim_start())
        } else {
            format!("# {title}")
        }
    } else {
        format!("# {title}\n\n{trimmed}")
    }
}

fn topic_overlaps_range(topic: &Topic, range: &LifeChapterRange) -> bool {
    dated_interval_overlaps(
        topic.first_mentioned.as_deref(),
        topic.last_mentioned.as_deref(),
        range,
    )
}

fn project_overlaps_range(project: &Project, range: &LifeChapterRange) -> bool {
    dated_interval_overlaps(
        project.start_date.as_deref(),
        project.end_date.as_deref(),
        range,
    )
}

fn dated_interval_overlaps(
    start_value: Option<&str>,
    end_value: Option<&str>,
    range: &LifeChapterRange,
) -> bool {
    let start = start_value.and_then(parse_date_prefix);
    let end = end_value.and_then(parse_date_prefix);
    match (start, end) {
        (Some(start), Some(end)) => start <= range.end_date && end >= range.start_date,
        (Some(date), None) | (None, Some(date)) => range.contains_date(date),
        (None, None) => false,
    }
}

fn record_date_in_range(record: &serde_json::Value, range: &LifeChapterRange) -> bool {
    record
        .get("date")
        .and_then(|value| value.as_str())
        .and_then(parse_date_prefix)
        .map(|date| range.contains_date(date))
        .unwrap_or(false)
}

fn parse_date_prefix(value: &str) -> Option<NaiveDate> {
    value
        .get(..10)
        .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::notes::{NoteStore, SaveNoteRequest};
    use crate::services::types::{Event, Observation};

    fn temp_store(name: &str) -> NoteStore {
        let root = std::env::temp_dir().join(format!("gc_life_chapter_{name}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp store");
        NoteStore::new(root)
    }

    fn event(id: &str, content: &str, created_at: &str) -> Event {
        Event {
            id: id.to_string(),
            content: content.to_string(),
            emotions: vec![],
            importance: 0.8,
            event_type: Some("milestone".to_string()),
            strength: 1.0,
            stability: 30.0,
            recall_count: 0,
            last_recalled_at: None,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
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
    fn validates_chapter_date_range() {
        let range = LifeChapterRange::new("2026-06-01", "2026-06-30").unwrap();
        assert_eq!(range.start_date.to_string(), "2026-06-01");
        assert_eq!(range.end_date.to_string(), "2026-06-30");

        let err = LifeChapterRange::new("2026-07-01", "2026-06-30").unwrap_err();
        assert_eq!(err.code, "invalidDateRange");
    }

    #[test]
    fn sparse_chapter_does_not_fill_missing_details() {
        let range = LifeChapterRange::new("2026-06-01", "2026-06-30").unwrap();
        let result = build_sparse_life_chapter(&range, &LifeChapterSources::default());

        assert!(result.content.contains("可用记录较少"));
        assert!(!result.content.contains("dialogue"));
        assert!(!result.content.contains("decision"));
        assert!(!result.title.trim().is_empty());
    }

    #[test]
    fn long_range_sources_are_limited_before_prompting() {
        let mut sources = LifeChapterSources::default();
        sources.events = (0..80)
            .map(|index| {
                event(
                    &format!("e{index}"),
                    "important event",
                    "2026-06-01T09:00:00Z",
                )
            })
            .collect();

        let limited = limit_chapter_sources(sources, 20);

        assert_eq!(limited.events.len(), 20);
    }

    #[test]
    fn generated_title_uses_date_range_and_source_theme() {
        let range = LifeChapterRange::new("2026-03-01", "2026-05-31").unwrap();
        let sources = LifeChapterSources {
            events: vec![event(
                "e1",
                "Started allowing imperfect work",
                "2026-04-01T09:00:00Z",
            )],
            observations: vec![observation(
                "o1",
                "2026-04-02",
                "More tolerant of imperfection",
            )],
            ..Default::default()
        };

        let title = generate_chapter_title(&range, &sources);

        assert!(title.contains("2026"));
        assert!(title.contains(":"));
        assert!(!title.contains("Untitled"));
        assert!(!title.contains("limited records"));
    }

    #[test]
    fn stable_chapter_note_identity_survives_title_edit() {
        let store = temp_store("stable-id");
        let range = LifeChapterRange::new("2026-06-01", "2026-06-30").unwrap();

        let saved = save_life_chapter_note(
            &store,
            &range,
            "2026 Jun: A quieter month",
            "# chapter",
            LifeChapterSourceCounts::default(),
        )
        .unwrap();
        let edited = store
            .update_note(
                &saved.note_id,
                SaveNoteRequest {
                    title: "A title edited by user".to_string(),
                    content: "# chapter edited".to_string(),
                    category: "life-chapter".to_string(),
                },
            )
            .unwrap();

        assert_eq!(saved.note_id, edited.id);
        assert_eq!(
            store.read_note(&saved.note_id).unwrap().title,
            "A title edited by user"
        );
    }

    #[test]
    fn chapter_prompt_contains_missing_detail_constraints() {
        let range = LifeChapterRange::new("2026-06-01", "2026-06-30").unwrap();
        let mut sources = LifeChapterSources {
            events: vec![event(
                "e1",
                "Started a calmer routine",
                "2026-06-01T09:00:00Z",
            )],
            ..Default::default()
        };
        sources.refresh_counts();

        let prompt = build_life_chapter_prompt(&range, &sources);

        assert!(prompt.contains("来源数量"));
        assert!(prompt.contains("不要编造缺失的原因"));
        assert!(prompt.contains("数字人格权重"));
        assert!(prompt.contains("全文必须使用中文输出"));
        assert!(prompt.contains("不要像 AI 资料汇总"));
        assert!(prompt.contains("只输出章节成品"));
        assert!(prompt.contains("2026-06-01 ~ 2026-06-30"));
    }

    #[test]
    fn growth_line_records_outside_range_do_not_count_as_material() {
        let range = LifeChapterRange::new("2026-06-01", "2026-06-30").unwrap();
        let mut sources = LifeChapterSources {
            growth_lines: vec![GrowthLine {
                id: "gl1".to_string(),
                dimension: "self_trust".to_string(),
                records: vec![serde_json::json!({"date": "2026-05-20", "note": "older record"})],
            }],
            ..Default::default()
        };
        sources.filter_to_range(&range);
        sources.refresh_counts();

        assert!(!sources.has_material());
        assert_eq!(sources.counts.growth_line_count, 0);
    }

    #[test]
    fn chapter_heading_is_normalized_to_selected_title() {
        let normalized =
            normalize_chapter_heading("2026 6-6: 真实标题", "# 模型自己起的标题\n\n正文内容");

        assert_eq!(normalized, "# 2026 6-6: 真实标题\n\n正文内容");
    }

    #[test]
    fn list_and_generate_results_hide_range_metadata_from_content() {
        let store = temp_store("hide-metadata");
        let range = LifeChapterRange::new("2026-06-01", "2026-06-30").unwrap();
        let saved = save_life_chapter_note(
            &store,
            &range,
            "2026 6-6: 真实标题",
            &with_range_metadata(&range, "# 2026 6-6: 真实标题\n\n正文内容"),
            LifeChapterSourceCounts::default(),
        )
        .unwrap();

        assert!(saved.content.starts_with("# 2026 6-6: 真实标题"));
        assert!(!saved.content.contains("start_date:"));

        let listed = list_life_chapters(&store).unwrap();
        assert_eq!(listed[0].start_date, "2026-06-01");
        assert_eq!(listed[0].end_date, "2026-06-30");
        assert!(!listed[0].content.contains("end_date:"));
    }

    #[test]
    fn update_life_chapter_preserves_note_id_and_date_metadata() {
        let store = temp_store("update");
        let range = LifeChapterRange::new("2026-06-01", "2026-06-30").unwrap();
        let saved = save_life_chapter_note(
            &store,
            &range,
            "Old title",
            &with_range_metadata(&range, "# Old title\n\nold body"),
            LifeChapterSourceCounts::default(),
        )
        .unwrap();

        let updated = update_life_chapter(
            &store,
            &saved.note_id,
            "New title".to_string(),
            "# User heading\n\nnew body".to_string(),
        )
        .unwrap();
        let stored = store.read_note(&saved.note_id).unwrap();

        assert_eq!(updated.note_id, saved.note_id);
        assert_eq!(updated.title, "New title");
        assert_eq!(updated.start_date, "2026-06-01");
        assert_eq!(updated.end_date, "2026-06-30");
        assert_eq!(updated.content, "# New title\n\nnew body");
        assert!(stored.content.contains("start_date: 2026-06-01"));
        assert!(stored.content.contains("end_date: 2026-06-30"));
        assert!(stored.content.contains("# New title"));
    }

    #[test]
    fn update_life_chapter_rejects_non_chapter_note() {
        let store = temp_store("update-non-chapter");
        let note = store
            .create_note(SaveNoteRequest {
                title: "Plain note".to_string(),
                content: "body".to_string(),
                category: "diary".to_string(),
            })
            .unwrap();

        let err = update_life_chapter(
            &store,
            &note.id,
            "New title".to_string(),
            "new body".to_string(),
        )
        .unwrap_err();

        assert_eq!(err.code, "lifeChapterNotFound");
    }
}
