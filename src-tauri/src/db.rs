use crate::app::FileType;
use crate::models::{EmbeddingKind, ModelManifest};
use anyhow::{Result, anyhow};
use include_dir::{Dir, include_dir};
use indoc::{formatdoc, indoc};
use rusqlite::{Connection, OptionalExtension, ffi::sqlite3_auto_extension, params};
use rusqlite_migration::Migrations;
use sqlite_vec::sqlite3_vec_init;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf, absolute},
    sync::Mutex,
};
use zerocopy::IntoBytes;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

pub struct FileEmbResult {
    pub file_id: i64,
    pub embeddings: Vec<EmbMetadata>,
}

pub struct EmbMetadata {
    pub id: i64,
    pub file_id: i64,
    pub emb_type_id: u32,
    pub last_file_mtime: f64,
    pub last_file_size: i64,
}

pub struct Database {
    conn: Mutex<Option<Connection>>,
}

impl Database {
    pub fn new(full_path: &str) -> Result<Self> {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }

        let mut conn = Connection::open(full_path)?;

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        Migrations::from_directory(&MIGRATIONS_DIR)?.to_latest(&mut conn)?;

        Ok(Self {
            conn: Mutex::new(Some(conn)),
        })
    }

    pub fn close(&self) {
        if let Ok(mut guard) = self.conn.lock() {
            if let Some(conn) = guard.take() {
                let _ = conn.execute_batch("PRAGMA optimize;");
                let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
                let _ = conn.close();
            }
        }
    }

    fn with_conn<F: FnOnce(&mut Connection) -> Result<R>, R>(&self, f: F) -> Result<R> {
        let mut guard = self.conn.lock().unwrap();
        let conn = guard.as_mut().unwrap();
        f(conn)
    }

    pub fn add_model_to_db(&self, model_manifest: &ModelManifest) -> Result<Vec<u32>> {
        let name = model_manifest.name;
        let mut ids = Vec::new();

        for kind in model_manifest.capabilities {
            // Check if embedding type already exists
            let existing = self.with_conn(|conn| {
                let mut stmt =
                    conn.prepare("SELECT id FROM emb_type WHERE kind = ?1 AND model_name = ?2")?;
                let id = stmt
                    .query_row([kind.as_str(), name], |row| row.get::<_, u32>(0))
                    .optional()?;
                Ok(id)
            })?;

            if let Some(id) = existing {
                ids.push(id);
                continue;
            }

            // Create virtual table for this model
            self.init_embedding_virtual_table(name, model_manifest.dim)?;

            // Insert new embedding type and return its id
            let id = self.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO emb_type (kind, model_name) VALUES (?1, ?2)",
                    [kind.as_str(), name],
                )?;
                Ok(conn.last_insert_rowid() as u32)
            })?;
            ids.push(id);
        }

        Ok(ids)
    }

    fn init_embedding_virtual_table(&self, model_name: &str, dim: u32) -> Result<()> {
        self.with_conn(|conn| {
            let sql = format!(
                "CREATE VIRTUAL TABLE IF NOT EXISTS vec_{} USING vec0(emb_id INTEGER PRIMARY KEY, vec float[{}])", model_name, dim
            );
            let _ = conn.execute(&sql, [])?;
            Ok(())
        })
    }

    pub fn get_emb_type_id(
        &self,
        model_manifest: &ModelManifest,
        kind: &EmbeddingKind,
    ) -> Result<u32> {
        self.with_conn(|conn| {
            let mut stmt =
                conn.prepare("SELECT id FROM emb_type WHERE kind = ?1 AND model_name = ?2")?;
            let id = stmt
                .query_row([kind.as_str(), model_manifest.name], |row| {
                    row.get::<_, u32>(0)
                })
                .map_err(|e| anyhow!(e));
            id
        })
    }

    pub fn get_emb_by_file_path(
        &self,
        path: &Path,
        emb_type_id: Option<u32>,
    ) -> Result<Option<FileEmbResult>> {
        self.with_conn(|conn| {
            let path_string = path.to_string_lossy();

            let sql = if emb_type_id.is_none() {
                indoc! {"
                    SELECT f.id, em.id, em.emb_type_id, em.last_file_mtime, em.last_file_size
                    FROM file f
                    LEFT JOIN emb_metadata em
                        ON em.file_id = f.id
                    WHERE f.path = ?1
                "}
            } else {
                indoc! {"
                    SELECT f.id, em.id, em.emb_type_id, em.last_file_mtime, em.last_file_size
                    FROM file f
                    LEFT JOIN emb_metadata em
                        ON em.file_id = f.id
                        AND em.emb_type_id = ?2
                    WHERE f.path = ?1
                "}
            };

            let mut stmt = conn.prepare(sql)?;
            let mut rows = if emb_type_id.is_none() {
                stmt.query(params![&path_string])?
            } else {
                stmt.query(params![&path_string, emb_type_id.unwrap()])?
            };

            let mut file_id: Option<i64> = None;
            let mut embeddings: Vec<EmbMetadata> = Vec::new();

            while let Some(row) = rows.next()? {
                file_id = row.get(0).ok();

                if let Some(emb_id) = row.get::<_, Option<i64>>(1)? {
                    embeddings.push(EmbMetadata {
                        id: emb_id,
                        file_id: file_id.unwrap(),
                        emb_type_id: row.get(2)?,
                        last_file_mtime: row.get(3)?,
                        last_file_size: row.get(4)?,
                    });
                }
            }

            Ok(file_id.map(|file_id| FileEmbResult {
                file_id,
                embeddings,
            }))
        })
    }

    pub fn insert_metadata(
        &self,
        paths_and_metadatas: Vec<(PathBuf, &FileType, HashMap<String, String>)>,
    ) -> Result<()> {
        self.with_conn(|conn: &mut Connection| {
            let tx = conn.transaction()?;

            for (path, file_type, key_value) in paths_and_metadatas.iter() {
                let path_str = absolute(path)?.display().to_string();
                tx.execute(indoc! {"
                    INSERT INTO file (path, type) VALUES (?1, ?2)
                    ON CONFLICT(path) DO UPDATE SET type = excluded.type
                "}, (&path_str, file_type.as_str()))?;
                let file_id: i64 = tx.query_row(
                    "SELECT id FROM file WHERE path = ?1",
                    params![&path_str],
                    |row| row.get(0),
                )?;

                for (key, value) in key_value {
                    tx.execute(indoc! {"
                        INSERT INTO file_metadata (file_id, meta_key, meta_value) VALUES (?1, ?2, ?3)
                        ON CONFLICT(file_id, meta_key) DO UPDATE SET meta_value = excluded.meta_value
                    "}, params![file_id, key, value])?;
                };
            };

            tx.commit()?;
            Ok(())
        })
    }

    pub fn insert_files_and_embeddings(
        &self,
        paths_and_embeddings: Vec<(PathBuf, Vec<f32>)>,
        files_type: &FileType,
        emb_model: &str,
        emb_type_id: u32,
    ) -> Result<()> {
        self.with_conn(|conn: &mut Connection| {
            let tx = conn.transaction()?;

            for (path, embedding) in paths_and_embeddings {
                let path_str = absolute(&path)?.display().to_string();
                let metadata = fs::metadata(path)?;
                let modified_at = metadata
                    .modified()?
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs_f64();
                let size = metadata.len() as i64;

                tx.execute(
                    indoc! {"
                        INSERT INTO file (path, type) VALUES (?1, ?2)
                        ON CONFLICT(path) DO UPDATE SET type = excluded.type
                    "},
                    (&path_str, files_type.as_str()),
                )?;
                let file_id: i64 = tx.query_row(
                    "SELECT id FROM file WHERE path = ?1",
                    params![&path_str],
                    |row| row.get(0),
                )?;

                let emb_id: i64 = tx.query_row(
                    indoc! {"
                        INSERT INTO emb_metadata (file_id, emb_type_id, last_file_mtime, last_file_size)
                        VALUES (?1, ?2, ?3, ?4)
                        ON CONFLICT(file_id, emb_type_id) DO UPDATE SET
                            last_file_mtime = excluded.last_file_mtime,
                            last_file_size = excluded.last_file_size
                        RETURNING id
                    "},
                    params![file_id, emb_type_id, modified_at, size],
                    |row| row.get(0),
                )?;

                tx.execute(
                    &format!(
                        "INSERT OR REPLACE INTO vec_{} (emb_id, vec) VALUES (?1, ?2)",
                        emb_model
                    ),
                    params![emb_id, embedding.as_bytes()],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
    }

    pub fn count_indexed_files(&self) -> Result<i64> {
        self.with_conn(|conn| {
            conn.query_row("SELECT COUNT(id) FROM file", [], |row| row.get(0))
                .map_err(Into::into)
        })
    }

    pub fn clear_orphan_vecs(&self) -> Result<usize> {
        let mut total: usize = 0;

        self.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT DISTINCT model_name FROM emb_type")?;
            let model_names: Vec<String> = {
                stmt.query_map([], |row| row.get::<_, String>(0))?
                    .filter_map(|r| r.ok())
                    .collect()
            };
            for model_name in &model_names {
                let sql = formatdoc!(
                    "
                    DELETE FROM vec_{0} WHERE NOT EXISTS (
                        SELECT 1 FROM emb_metadata
                        WHERE emb_metadata.id = vec_{0}.emb_id
                    )
                    ",
                    model_name
                );
                total += conn.execute(&sql, [])?;
            }

            Ok(total)
        })
    }
}
