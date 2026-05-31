use serde::{Deserialize, Serialize};
use std::path::Path;

use super::notes::{write_json_atomic, AppError};

// ─── AI Configuration ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    #[serde(default)]
    pub llm_api_key: String,
    #[serde(default = "default_llm_base_url")]
    pub llm_base_url: String,
    #[serde(default = "default_llm_model")]
    pub llm_model: String,
    #[serde(default = "default_llm_cheap_model")]
    pub llm_cheap_model: String,

    // Core memory capacity
    #[serde(default = "default_profile_max_chars")]
    pub profile_max_chars: u32,
    #[serde(default = "default_notes_max_chars")]
    pub notes_max_chars: u32,
    #[serde(default = "default_capacity_warning_pct")]
    pub capacity_warning_pct: f64,

    // Chat history
    #[serde(default = "default_max_history_turns")]
    pub max_history_turns: u32,

    // Forgetting curve
    #[serde(default = "default_forget_min_strength")]
    pub forget_min_strength: f64,
    #[serde(default = "default_forget_base_stability")]
    pub forget_base_stability: f64,
    #[serde(default = "default_forget_recall_boost")]
    pub forget_recall_boost: f64,

    // Heartbeat
    #[serde(default = "default_heartbeat_interval_minutes")]
    pub heartbeat_interval_minutes: u64,
    #[serde(default = "default_heartbeat_min_idle_minutes")]
    pub heartbeat_min_idle_minutes: u64,
    #[serde(default = "default_heartbeat_max_idle_minutes")]
    pub heartbeat_max_idle_minutes: u64,
    #[serde(default = "default_heartbeat_proactive_enabled")]
    pub heartbeat_proactive_enabled: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            llm_api_key: String::new(),
            llm_base_url: default_llm_base_url(),
            llm_model: default_llm_model(),
            llm_cheap_model: default_llm_cheap_model(),
            profile_max_chars: default_profile_max_chars(),
            notes_max_chars: default_notes_max_chars(),
            capacity_warning_pct: default_capacity_warning_pct(),
            max_history_turns: default_max_history_turns(),
            forget_min_strength: default_forget_min_strength(),
            forget_base_stability: default_forget_base_stability(),
            forget_recall_boost: default_forget_recall_boost(),
            heartbeat_interval_minutes: default_heartbeat_interval_minutes(),
            heartbeat_min_idle_minutes: default_heartbeat_min_idle_minutes(),
            heartbeat_max_idle_minutes: default_heartbeat_max_idle_minutes(),
            heartbeat_proactive_enabled: default_heartbeat_proactive_enabled(),
        }
    }
}

// ─── Default Functions ────────────────────────────────────────────────────

fn default_llm_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}
fn default_llm_model() -> String {
    "gpt-4o-mini".to_string()
}
fn default_llm_cheap_model() -> String {
    "gpt-4o-mini".to_string()
}
fn default_profile_max_chars() -> u32 {
    1200
}
fn default_notes_max_chars() -> u32 {
    800
}
fn default_capacity_warning_pct() -> f64 {
    0.8
}
fn default_max_history_turns() -> u32 {
    20
}
fn default_forget_min_strength() -> f64 {
    0.05
}
fn default_forget_base_stability() -> f64 {
    30.0
}
fn default_forget_recall_boost() -> f64 {
    0.5
}
fn default_heartbeat_interval_minutes() -> u64 {
    30
}
fn default_heartbeat_min_idle_minutes() -> u64 {
    120
}
fn default_heartbeat_max_idle_minutes() -> u64 {
    1440
}
fn default_heartbeat_proactive_enabled() -> bool {
    true
}

// ─── Load / Save ──────────────────────────────────────────────────────────

pub fn ai_config_path(base_dir: &Path) -> std::path::PathBuf {
    base_dir.join("ai_config.json")
}

pub fn load_ai_config(base_dir: &Path) -> Result<AiConfig, AppError> {
    let path = ai_config_path(base_dir);
    if !path.exists() {
        return Ok(AiConfig::default());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| AppError::new("io", format!("Failed to read ai_config.json: {e}")))?;
    let config: AiConfig = serde_json::from_str(&data)
        .map_err(|e| AppError::new("parse", format!("Failed to parse ai_config.json: {e}")))?;
    Ok(config)
}

pub fn save_ai_config(base_dir: &Path, config: &AiConfig) -> Result<(), AppError> {
    let path = ai_config_path(base_dir);
    write_json_atomic(&path, config)
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AiConfig::default();
        assert_eq!(config.llm_api_key, "");
        assert_eq!(config.llm_base_url, "https://api.openai.com/v1");
        assert_eq!(config.llm_model, "gpt-4o-mini");
        assert_eq!(config.profile_max_chars, 1200);
        assert_eq!(config.notes_max_chars, 800);
        assert_eq!(config.max_history_turns, 20);
        assert!((config.forget_min_strength - 0.05).abs() < f64::EPSILON);
        assert!((config.forget_base_stability - 30.0).abs() < f64::EPSILON);
        assert!(config.heartbeat_proactive_enabled);
    }

    #[test]
    fn test_load_save_roundtrip() {
        let dir = std::env::temp_dir().join("gc_test_config_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let config = AiConfig {
            llm_api_key: "test-key".to_string(),
            llm_base_url: "https://api.custom.com/v1".to_string(),
            ..AiConfig::default()
        };

        save_ai_config(&dir, &config).unwrap();
        let loaded = load_ai_config(&dir).unwrap();
        assert_eq!(config, loaded);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let dir = std::env::temp_dir().join("gc_test_config_missing");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let loaded = load_ai_config(&dir).unwrap();
        assert_eq!(loaded, AiConfig::default());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_serde_camel_case() {
        let config = AiConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"llmApiKey\":"));
        assert!(json.contains("\"llmBaseUrl\":"));
        assert!(json.contains("\"maxHistoryTurns\":"));
    }
}
