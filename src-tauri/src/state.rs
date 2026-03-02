use crate::db::Database;
use crate::models;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};

const PATH_CONFIG: &'static str = "config.json";
const PATH_DB: &'static str = "index.db";
const PATH_MODELS_DIR: &'static str = "models";

pub struct AppState {
    pub data_path: PathBuf,
    pub config: RwLock<Config>,
    pub db: Database,
    pub model_manager: models::ModelManager,
    pub selected_model: &'static models::ModelManifest,
    pub is_indexing: Mutex<bool>,
    pub indexing_stopped: AtomicBool,
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

        fs::create_dir_all(&data_path).unwrap();

        let db = match Database::new(&data_path.join(PATH_DB).to_string_lossy()) {
            Ok(db) => db,
            Err(e) => {
                panic!("Failed to load database: {}", e);
            }
        };

        db.clear_orphan_vecs().unwrap();

        let model_manager = models::ModelManager::new(data_path.join(PATH_MODELS_DIR));
        let selected_model = &models::metaclip2::MANIFEST;

        Self {
            config: RwLock::new(Config::load(&data_path.join(PATH_CONFIG))),
            model_manager,
            selected_model,
            is_indexing: Mutex::new(false),
            db,
            data_path,
            indexing_stopped: AtomicBool::new(false),
        }
    }

    pub fn save_config(&self) {
        let config = self.config.read().unwrap();
        config.save(&self.data_path.join(PATH_CONFIG));
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

impl Config {
    fn load(config_json_path: &Path) -> Self {
        match std::fs::read_to_string(config_json_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            Some(config) => config,
            None => {
                let default = Config::default();
                default.save(config_json_path);
                default
            }
        }
    }

    fn save(&self, config_json_path: &Path) {
        std::fs::write(
            config_json_path,
            serde_json::to_string_pretty(self).unwrap(),
        )
        .unwrap();
    }
}
