pub mod desktop;
pub mod locales;
pub mod services;

use locales::Locale;
use services::notes::{
    default_base_dir, default_store, AppConfig, AppError, Note, NoteMetadata, SaveNoteRequest,
};
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager};

// ─── Existing Note Commands ───────────────────────────────────────────────

#[tauri::command]
fn app_name() -> Result<String, AppError> {
    let locale = Locale::from_tag(&default_store()?.load_config()?.locale);
    Ok(locales::app_name(locale).to_string())
}

#[tauri::command]
fn notes_list() -> Result<Vec<NoteMetadata>, AppError> {
    default_store()?.list_notes()
}

#[tauri::command]
fn notes_get(id: String) -> Result<Note, AppError> {
    default_store()?.read_note(&id)
}

#[tauri::command]
fn notes_create(app: AppHandle, request: SaveNoteRequest) -> Result<Note, AppError> {
    let note = default_store()?.create_note(request)?;
    let _ = app.emit("notes-changed", ());
    Ok(note)
}

#[tauri::command]
fn notes_update(app: AppHandle, id: String, request: SaveNoteRequest) -> Result<Note, AppError> {
    let note = default_store()?.update_note(&id, request)?;
    let _ = app.emit("notes-changed", ());
    Ok(note)
}

#[tauri::command]
fn notes_delete(app: AppHandle, id: String) -> Result<(), AppError> {
    default_store()?.delete_note(&id)?;
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn notes_import_markdown(
    app: AppHandle,
    path: String,
    category: Option<String>,
) -> Result<Note, AppError> {
    let note = default_store()?
        .import_markdown_file(&PathBuf::from(path), &category.unwrap_or_default())?;
    let _ = app.emit("notes-changed", ());
    Ok(note)
}

#[tauri::command]
fn notes_export_markdown(id: String, path: String) -> Result<(), AppError> {
    default_store()?.export_markdown_file(&id, &PathBuf::from(path))
}

#[tauri::command]
fn read_external_file(path: String) -> Result<String, AppError> {
    std::fs::read_to_string(&path).map_err(|e| AppError {
        code: "io".into(),
        message: e.to_string(),
        details: Default::default(),
    })
}

#[tauri::command]
fn get_file_modified_time(path: String) -> Result<f64, AppError> {
    let metadata = std::fs::metadata(&path).map_err(|e| AppError {
        code: "io".into(),
        message: e.to_string(),
        details: Default::default(),
    })?;
    let modified = metadata.modified().map_err(|e| AppError {
        code: "io".into(),
        message: e.to_string(),
        details: Default::default(),
    })?;
    let duration = modified
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    Ok(duration.as_secs_f64() * 1000.0)
}

#[tauri::command]
fn save_external_file(path: String, content: String) -> Result<(), AppError> {
    if let Some(parent) = PathBuf::from(&path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError {
            code: "io".into(),
            message: e.to_string(),
            details: Default::default(),
        })?;
    }
    std::fs::write(&path, content).map_err(|e| AppError {
        code: "io".into(),
        message: e.to_string(),
        details: Default::default(),
    })
}

#[tauri::command]
fn categories_list() -> Result<Vec<String>, AppError> {
    default_store()?.list_categories()
}

#[tauri::command]
fn categories_create(app: AppHandle, name: String) -> Result<(), AppError> {
    default_store()?.create_category(&name)?;
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn categories_rename(app: AppHandle, old_name: String, new_name: String) -> Result<(), AppError> {
    default_store()?.rename_category(&old_name, &new_name)?;
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn categories_delete(app: AppHandle, name: String) -> Result<(), AppError> {
    default_store()?.delete_category(&name)?;
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn notes_move_category(
    app: AppHandle,
    id: String,
    category: String,
) -> Result<NoteMetadata, AppError> {
    let result = default_store()?.move_note_to_category(&id, &category)?;
    let _ = app.emit("notes-changed", ());
    Ok(result)
}

#[tauri::command]
fn config_get() -> Result<AppConfig, AppError> {
    default_store()?.load_config()
}

#[tauri::command]
fn copy_background_image(_app: AppHandle, source_path: String) -> Result<String, AppError> {
    let source = PathBuf::from(source_path.trim());
    if !source.is_file() {
        return Err(AppError {
            code: "invalidSource".into(),
            message: "background image source not found".into(),
            details: Default::default(),
        });
    }

    let store = default_store()?;
    let dir = store.base_dir().join("backgrounds");
    fs::create_dir_all(&dir)?;

    let old_config = store.load_config()?;
    if !old_config.background_image_path.is_empty() {
        let old_path = PathBuf::from(&old_config.background_image_path);
        if old_path.starts_with(&dir) && old_path.is_file() {
            let _ = fs::remove_file(&old_path);
        }
    }

    let ext = source
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("png");
    let dest = dir.join(format!("bg-{}.{}", uuid::Uuid::new_v4(), ext));
    fs::copy(&source, &dest)?;

    dest.to_str().map(str::to_string).ok_or_else(|| AppError {
        code: "path".into(),
        message: "invalid destination path".into(),
        details: Default::default(),
    })
}

#[tauri::command]
fn config_save(app: AppHandle, config: AppConfig) -> Result<AppConfig, AppError> {
    let store = default_store()?;
    let previous = store.load_config()?;
    desktop::apply_runtime_config(&app, &previous, &config).map_err(|error| {
        match error.downcast::<AppError>() {
            Ok(app_error) => *app_error,
            Err(error) => AppError {
                code: "desktopConfig".into(),
                message: error.to_string(),
                details: Default::default(),
            },
        }
    })?;
    let saved = store.save_config(config)?;
    if let Err(error) = desktop::refresh_shell_state(&app, &saved) {
        eprintln!("failed to refresh desktop shell state: {error}");
    }
    let _ = app.emit("config-changed", &saved);
    Ok(saved)
}

#[tauri::command]
fn global_shortcut_check(
    app: AppHandle,
    shortcut: String,
) -> Result<desktop::ShortcutCheckResult, AppError> {
    desktop::check_global_shortcut(&app, &shortcut)
}

#[tauri::command]
async fn open_notepad_window(
    app: AppHandle,
    note_id: Option<String>,
    bounds: Option<desktop::WindowBounds>,
) -> Result<String, AppError> {
    desktop::open_notepad_window(app, note_id, bounds).await
}

#[tauri::command]
async fn recycle_notepad_window(app: AppHandle, label: String) -> Result<(), AppError> {
    desktop::recycle_notepad_window(&app, &label)
}

#[tauri::command]
async fn open_tile_window(
    app: AppHandle,
    note_id: String,
    bounds: Option<desktop::WindowBounds>,
) -> Result<String, AppError> {
    desktop::open_tile_window(app, note_id, bounds).await
}

#[tauri::command]
async fn toggle_tile_window(
    app: AppHandle,
    note_id: String,
    bounds: Option<desktop::WindowBounds>,
) -> Result<bool, AppError> {
    desktop::toggle_tile_window(app, note_id, bounds).await
}

#[tauri::command]
async fn open_note_in_editor(app: AppHandle, note_id: String) -> Result<(), AppError> {
    desktop::show_main_window(&app)?;
    let _ = app.emit("open-note", &note_id);
    Ok(())
}

#[tauri::command]
fn take_startup_file() -> Option<String> {
    desktop::take_startup_file()
}

// ─── AI Commands ──────────────────────────────────────────────────────────

#[tauri::command]
async fn ai_init_user(app: AppHandle, user_id: String) -> Result<(), AppError> {
    let base_dir = default_base_dir()?;
    let db = app.state::<services::database::DbState>();

    // Create user directory
    let user_dir = base_dir.join(&user_id);
    std::fs::create_dir_all(&user_dir)
        .map_err(|e| AppError::new("io", format!("Failed to create user dir: {e}")))?;

    // Initialize database
    db.init_db(&user_id)?;

    Ok(())
}

#[tauri::command]
fn ai_config_get() -> Result<services::config::AiConfig, AppError> {
    let base_dir = default_base_dir()?;
    services::config::load_ai_config(&base_dir)
}

#[tauri::command]
fn ai_config_save(app: AppHandle, config: services::config::AiConfig) -> Result<(), AppError> {
    let base_dir = default_base_dir()?;
    services::config::save_ai_config(&base_dir, &config)?;
    let _ = app.emit("ai-config-changed", &config);
    Ok(())
}

#[tauri::command]
fn ai_get_core_memory(user_id: String) -> Result<services::types::CoreMemoryResponse, AppError> {
    let base_dir = default_base_dir()?;
    let config = services::config::load_ai_config(&base_dir)?;
    let memory = services::memory::load_core_memory(&base_dir, &user_id, &config)?;
    Ok(services::memory::build_core_memory_response(&memory))
}

#[tauri::command]
fn ai_patch_core_memory(
    user_id: String,
    patch: services::types::MemoryPatch,
) -> Result<services::types::CoreMemoryResponse, AppError> {
    let base_dir = default_base_dir()?;
    let config = services::config::load_ai_config(&base_dir)?;
    let memory = services::memory::patch_core_memory(&base_dir, &user_id, &config, &patch)?;
    Ok(services::memory::build_core_memory_response(&memory))
}

#[tauri::command]
fn ai_get_events(
    app: AppHandle,
    user_id: String,
    params: services::types::QueryEventsParams,
) -> Result<Vec<services::types::Event>, AppError> {
    let db = app.state::<services::database::DbState>();
    db.query_events(&user_id, &params)
}

#[tauri::command]
fn ai_delete_event(app: AppHandle, user_id: String, event_id: String) -> Result<bool, AppError> {
    let db = app.state::<services::database::DbState>();
    db.delete_event(&user_id, &event_id)
}

#[tauri::command]
fn ai_get_chat_days(user_id: String) -> Result<Vec<services::types::ChatDaySummary>, AppError> {
    let base_dir = default_base_dir()?;
    services::memory::list_history_days(&base_dir, &user_id)
}

#[tauri::command]
fn ai_get_history(
    user_id: String,
    date: Option<String>,
) -> Result<Vec<services::types::ChatMessage>, AppError> {
    let base_dir = default_base_dir()?;
    let config = services::config::load_ai_config(&base_dir)?;
    match date {
        Some(value) => services::memory::load_history_for_date(
            &base_dir,
            &user_id,
            &value,
            config.max_history_turns,
        ),
        None => services::memory::load_history(&base_dir, &user_id, config.max_history_turns),
    }
}

#[tauri::command]
fn ai_clear_history(user_id: String, date: Option<String>) -> Result<(), AppError> {
    let base_dir = default_base_dir()?;
    let target_date = date.unwrap_or_else(services::memory::current_history_date);
    services::memory::clear_history_for_date(&base_dir, &user_id, &target_date)
}

#[tauri::command]
fn ai_get_observations(
    app: AppHandle,
    user_id: String,
    category: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<services::types::Observation>, AppError> {
    let db = app.state::<services::database::DbState>();
    db.query_observations(&user_id, category.as_deref(), limit.unwrap_or(20))
}

#[tauri::command]
fn ai_get_topics(
    app: AppHandle,
    user_id: String,
    limit: Option<usize>,
) -> Result<Vec<services::types::Topic>, AppError> {
    let db = app.state::<services::database::DbState>();
    db.query_topics(&user_id, limit.unwrap_or(50))
}

#[tauri::command]
fn ai_get_topic_detail(
    app: AppHandle,
    user_id: String,
    topic_id: String,
) -> Result<serde_json::Value, AppError> {
    let db = app.state::<services::database::DbState>();
    let links = db.get_topic_links(&user_id, &topic_id)?;
    Ok(serde_json::json!({ "topicId": topic_id, "links": links }))
}

#[tauri::command]
fn ai_get_projects(
    app: AppHandle,
    user_id: String,
    status: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<services::types::Project>, AppError> {
    let db = app.state::<services::database::DbState>();
    db.query_projects(&user_id, status.as_deref(), limit.unwrap_or(50))
}

#[tauri::command]
fn ai_get_growth_lines(
    app: AppHandle,
    user_id: String,
    limit: Option<usize>,
) -> Result<Vec<services::types::GrowthLine>, AppError> {
    let db = app.state::<services::database::DbState>();
    db.query_growth_lines(&user_id, limit.unwrap_or(50))
}

#[tauri::command]
fn ai_maintain_memory(app: AppHandle, user_id: String) -> Result<serde_json::Value, AppError> {
    let base_dir = default_base_dir()?;
    let config = services::config::load_ai_config(&base_dir)?;
    let db = app.state::<services::database::DbState>();

    let decayed = db.decay_all_events(&user_id, config.forget_min_strength)?;
    let cleaned = db.cleanup_forgotten_events(&user_id, config.forget_min_strength)?;

    Ok(serde_json::json!({
        "decayedCount": decayed,
        "cleanedCount": cleaned,
    }))
}

#[tauri::command]
fn ai_search_conversations(
    app: AppHandle,
    user_id: String,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<services::types::ConversationTurn>, AppError> {
    let db = app.state::<services::database::DbState>();
    db.search_conversations(&user_id, &query, limit.unwrap_or(5))
}

// ─── Diary Commands ────────────────────────────────────────────────────────

#[tauri::command]
async fn ai_generate_diary(
    app: AppHandle,
    user_id: String,
    date: Option<String>,
) -> Result<services::types::DiaryGenerateResult, AppError> {
    let base_dir = default_base_dir()?;
    let db = app.state::<services::database::DbState>();
    let client = {
        let llm_state = app.state::<Mutex<reqwest::Client>>();
        let guard = llm_state
            .lock()
            .map_err(|e| AppError::new("state", format!("Failed to lock LLM client: {e}")))?;
        guard.clone()
    };

    services::diary::generate_diary(&base_dir, &db, &client, &user_id, date).await
}

#[tauri::command]
fn ai_get_diary_list(
    user_id: String,
    limit: Option<usize>,
) -> Result<Vec<services::types::DiaryEntry>, AppError> {
    services::diary::get_diary_list(&user_id, limit)
}

#[tauri::command]
fn ai_get_diary(
    user_id: String,
    date: String,
) -> Result<Option<services::types::DiaryEntry>, AppError> {
    services::diary::get_diary(&user_id, date)
}

#[tauri::command]
async fn ai_regenerate_diary(
    app: AppHandle,
    user_id: String,
    date: String,
) -> Result<services::types::DiaryGenerateResult, AppError> {
    let base_dir = default_base_dir()?;
    let db = app.state::<services::database::DbState>();
    let client = {
        let llm_state = app.state::<Mutex<reqwest::Client>>();
        let guard = llm_state
            .lock()
            .map_err(|e| AppError::new("state", format!("Failed to lock LLM client: {e}")))?;
        guard.clone()
    };

    services::diary::regenerate_diary(&base_dir, &db, &client, &user_id, date).await
}

// ─── App Entry Point ──────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(file_path) = desktop::extract_file_arg(&args) {
                let _ = app.emit("open-external-file", file_path);
            }
            let _ = desktop::show_main_window(app);
        }))
        .setup(|app| {
            desktop::setup_desktop(app)?;

            // Initialize AI state
            let base_dir = default_base_dir()?;
            app.manage(services::database::DbState::new(base_dir.clone()));
            app.manage(Mutex::new(reqwest::Client::new()));
            app.manage(services::scheduler::PendingMessages::default());

            // Start heartbeat if AI is configured
            if let Ok(ai_config) = services::config::load_ai_config(&base_dir) {
                if !ai_config.llm_api_key.is_empty() {
                    services::scheduler::start_heartbeat(
                        app.handle().clone(),
                        ai_config.heartbeat_interval_minutes,
                    );
                }
            }

            Ok(())
        })
        .on_window_event(desktop::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            // Existing note commands
            app_name,
            notes_list,
            notes_get,
            notes_create,
            notes_update,
            notes_delete,
            notes_import_markdown,
            notes_export_markdown,
            notes_move_category,
            read_external_file,
            save_external_file,
            get_file_modified_time,
            categories_list,
            categories_create,
            categories_rename,
            categories_delete,
            config_get,
            copy_background_image,
            config_save,
            global_shortcut_check,
            open_notepad_window,
            recycle_notepad_window,
            open_tile_window,
            toggle_tile_window,
            open_note_in_editor,
            take_startup_file,
            // AI commands
            ai_init_user,
            ai_config_get,
            ai_config_save,
            ai_get_core_memory,
            ai_patch_core_memory,
            ai_get_events,
            ai_delete_event,
            ai_get_chat_days,
            ai_get_history,
            ai_clear_history,
            ai_get_observations,
            ai_get_topics,
            ai_get_topic_detail,
            ai_get_projects,
            ai_get_growth_lines,
            ai_maintain_memory,
            ai_search_conversations,
            // Chat + scheduler commands (from services)
            services::chat::chat_send,
            services::chat::chat_stream_start,
            services::extractor::quick_extract,
            services::scheduler::ai_get_pending_message,
            // Diary commands
            ai_generate_diary,
            ai_get_diary_list,
            ai_get_diary,
            ai_regenerate_diary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
