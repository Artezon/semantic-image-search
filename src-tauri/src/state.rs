use crate::db::Database;
use crate::errors::fatal_error;
use crate::models;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU64};

pub const PATH_CONFIG: &'static str = "config.json";
pub const PATH_DB: &'static str = "index.db";
pub const PATH_MODELS_DIR: &'static str = "models";

pub fn resolve_lang(lang: &str) -> String {
    if lang == "system" {
        sys_locale::get_locale()
            .unwrap_or_default()
            .trim()
            .split(['-', '_'])
            .next()
            .unwrap_or("en")
            .to_string()
    } else {
        lang.to_string()
    }
}

pub struct AppState {
    pub data_path: PathBuf,
    pub config: RwLock<Config>,
    pub db: Database,
    pub model_manager: models::ModelManager,
    pub selected_model: &'static models::ModelManifest,
    pub is_indexing: AtomicBool,
    pub indexing_paused: AtomicBool,
    pub indexing_elapsed_secs: AtomicU64,
}

impl AppState {
    pub fn new(handle: &tauri::AppHandle) -> Self {
        #[cfg(feature = "portable")]
        let _ = handle;

        #[cfg(feature = "portable")]
        let data_path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("data");

        #[cfg(not(feature = "portable"))]
        let data_path = handle.path().app_local_data_dir().unwrap();

        fs::create_dir_all(&data_path).unwrap_or_else(|e| {
            fatal_error(
                handle,
                &format!(
                    "Could not create or access data directory at '{}'\n\nError: {}",
                    data_path.display(),
                    e
                ),
                "Startup Error",
            )
        });

        let db = match Database::new(&data_path.join(PATH_DB).to_string_lossy()) {
            Ok(db) => db,
            Err(e) => fatal_error(
                handle,
                &format!("Failed to load database: {}", e),
                "Startup Error",
            ),
        };

        let model_manager = models::ModelManager::new(data_path.join(PATH_MODELS_DIR));
        let selected_model = &models::metaclip2::MANIFEST;

        Self {
            config: RwLock::new(Config::load(&data_path.join(PATH_CONFIG))),
            model_manager,
            selected_model,
            is_indexing: AtomicBool::new(false),
            db,
            data_path,
            indexing_paused: AtomicBool::new(false),
            indexing_elapsed_secs: AtomicU64::new(0),
        }
    }

    pub fn update_config(&self, updates: HashMap<String, Value>) {
        let mut config = self.config.write().unwrap();
        let mut config_value = serde_json::to_value(&*config).unwrap();
        if let Some(obj) = config_value.as_object_mut() {
            for (key, value) in updates {
                obj.insert(key, value);
            }
        }
        *config = serde_json::from_value(config_value).unwrap();
    }

    pub fn save_config(&self) {
        let config = self.config.read().unwrap();
        config.save(&self.data_path.join(PATH_CONFIG));
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    pub lang: String,
    pub batch_size: u32,
    pub video_frames: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lang: "system".into(),
            batch_size: 8,
            video_frames: 5,
        }
    }
}

impl Config {
    fn load(config_json_path: &Path) -> Self {
        let config: Config = std::fs::read_to_string(config_json_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        config.save(config_json_path);
        config
    }

    fn save(&self, config_json_path: &Path) {
        std::fs::write(config_json_path, serde_json::to_string(self).unwrap()).unwrap();
    }
}
