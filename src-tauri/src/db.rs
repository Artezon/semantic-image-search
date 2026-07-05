use crate::models::{EmbeddingKind, ModelManifest};
use anyhow::Result;
use include_dir::{Dir, include_dir};
use indoc::{formatdoc, indoc};
use rusqlite::{Connection, OptionalExtension, ffi::sqlite3_auto_extension, params};
use rusqlite_migration::Migrations;
use sqlite_vec::sqlite3_vec_init;
use std::{
    collections::HashSet,
    path::{Path, PathBuf, absolute},
    sync::Mutex,
};
use zerocopy::IntoBytes;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

#[derive(PartialEq)]
pub enum FileType {
    IMG,
    #[cfg(feature = "video")]
    VID,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IMG => "IMG",
            #[cfg(feature = "video")]
            Self::VID => "VID",
        }
    }
}

pub struct FileWithEmbStatus {
    pub id: i64,
    pub path: PathBuf,
    pub file_type: String,
    pub last_file_mtime: Option<i64>,
    pub last_file_size: Option<i64>,
}

// pub struct FileMetadata {
//     file_id: i64,
//     key_value: HashMap<String, String>,
// }

pub struct FileEmbedding {
    pub file_id: i64,
    pub file_mtime: i64,
    pub file_size: i64,
    pub embedding: Vec<f32>,
}

// pub struct FileEmbResult {
//     pub file_id: i64,
//     pub embeddings: Vec<EmbMetadata>,
// }

// pub struct EmbMetadata {
//     pub id: i64,
//     pub file_id: i64,
//     pub emb_type_id: u32,
//     pub last_file_mtime: i64,
//     pub last_file_size: i64,
// }

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub path: String,
    pub filename: String,
    pub file_type: String,
    pub score: f32,
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

    pub fn count_indexed_files(&self) -> Result<i64> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(DISTINCT file_id) FROM emb_metadata",
                [],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
    }

    pub fn get_dirs(&self) -> Result<Vec<String>> {
        self.with_conn(|conn| {
            let mut stmt =
                conn.prepare("SELECT path FROM indexed_directory ORDER BY sort_order ASC")?;
            let paths = stmt
                .query_map([], |row| row.get::<_, String>(0))?
                .filter_map(|r| r.ok())
                .collect();
            Ok(paths)
        })
    }

    pub fn add_directory(&self, dir_path: &str) -> Result<()> {
        self.with_conn(|conn| {
            let max_order: i64 = conn
                .query_row(
                    "SELECT COALESCE(MAX(sort_order), -1) FROM indexed_directory",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(-1);
            conn.execute(
                "INSERT OR IGNORE INTO indexed_directory (path, sort_order) VALUES (?1, ?2)",
                params![dir_path, max_order + 1],
            )?;
            Ok(())
        })
    }

    pub fn update_directory(&self, dir_path: &str, files: Vec<(PathBuf, FileType)>) -> Result<usize> {
        let files: Vec<(String, FileType)> = files
            .into_iter()
            .map(|(p, t)| Ok((absolute(&p)?.display().to_string(), t)))
            .collect::<Result<_>>()?;

        self.with_conn(|conn| {
            let tx = conn.transaction()?;

            let dir_id: i64 = tx.query_row(
                "SELECT id FROM indexed_directory WHERE path = ?1",
                params![dir_path],
                |row| row.get(0),
            )?;

            // Load files existing in database
            let existing: Vec<(i64, String)> = {
                let mut stmt = tx.prepare(indoc! {"
                    SELECT f.id, f.path FROM file f
                    JOIN directory_files df ON df.file_id = f.id
                    WHERE df.directory_id = ?1
                "})?;
                stmt.query_map(params![dir_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<rusqlite::Result<_>>()?
            };

            let new_paths: HashSet<&str> = files.iter().map(|(p, _)| p.as_str()).collect();

            // Remove files no longer on disk from database
            let to_remove: Vec<i64> = existing
                .iter()
                .filter(|(_, p)| !new_paths.contains(p.as_str()))
                .map(|(id, _)| *id)
                .collect();
            let removed_count = to_remove.len();
            for file_id in to_remove {
                tx.execute(
                    "DELETE FROM directory_files WHERE directory_id = ?1 AND file_id = ?2",
                    params![dir_id, file_id],
                )?;
            }

            // Upsert files to database
            for (path_str, file_type) in &files {
                let file_id: i64 = tx.query_row(
                    indoc! {"
                        INSERT INTO file (path, type) VALUES (?1, ?2)
                        ON CONFLICT(path) DO UPDATE SET type = excluded.type
                        RETURNING id
                    "},
                    params![path_str, file_type.as_str()],
                    |row| row.get(0),
                )?;
                tx.execute(
                    "INSERT OR IGNORE INTO directory_files (directory_id, file_id) VALUES (?1, ?2)",
                    params![dir_id, file_id],
                )?;
            }

            tx.commit()?;
            Ok(removed_count)
        })
    }

    pub fn remove_directory(&self, path: &str) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM indexed_directory WHERE path = ?1",
                params![path],
            )?;
            Ok(())
        })
    }

    pub fn reorder_directories(&self, paths: &[String]) -> Result<()> {
        self.with_conn(|conn| {
            let tx = conn.transaction()?;
            for (i, p) in paths.iter().enumerate() {
                tx.execute(
                    "UPDATE indexed_directory SET sort_order = ?1 WHERE path = ?2",
                    params![i as i64, p],
                )?;
            }
            tx.commit()?;
            Ok(())
        })
    }

    pub fn add_model_to_db(&self, model_manifest: &ModelManifest) -> Result<Vec<u32>> {
        let name = model_manifest.name;
        let dim = model_manifest.dim;
        let mut ids = vec![];

        self.with_conn(|conn| {
            let tx = conn.transaction()?;

            for kind in model_manifest.capabilities {
                // Check if embedding type already exists
                let existing: Option<u32> = {
                    let mut stmt = tx.prepare(
                        "SELECT id FROM emb_type WHERE kind = ?1 AND model_name = ?2",
                    )?;
                    stmt.query_row([kind.as_str(), name], |row| row.get::<_, u32>(0))
                        .optional()?
                };

                if let Some(id) = existing {
                    ids.push(id);
                    continue;
                }

                // Insert new embedding type and return its id
                tx.execute(
                    "INSERT INTO emb_type (kind, model_name) VALUES (?1, ?2)",
                    [kind.as_str(), name],
                )?;
                let id = tx.last_insert_rowid() as u32;

                // Create virtual table for this model
                let sql = format!(
                    "CREATE VIRTUAL TABLE IF NOT EXISTS vec_model{id} USING vec0(emb_id INTEGER PRIMARY KEY, vec float[{dim}])"
                );
                tx.execute(&sql, [])?;
                let trigger = formatdoc!("
                    CREATE TRIGGER IF NOT EXISTS trg_emb_metadata_del_vec_model{id}
                    AFTER DELETE ON emb_metadata
                    WHEN OLD.emb_type_id = {id}
                    BEGIN
                        DELETE FROM vec_model{id} WHERE emb_id = OLD.id;
                    END
                ");
                tx.execute(&trigger, [])?;

                ids.push(id);
            }

            tx.commit()?;
            Ok(())
        })?;

        Ok(ids)
    }

    pub fn get_emb_type_id(
        &self,
        model_manifest: &ModelManifest,
        kind: &EmbeddingKind,
    ) -> Result<Option<u32>> {
        self.with_conn(|conn| {
            let mut stmt =
                conn.prepare("SELECT id FROM emb_type WHERE kind = ?1 AND model_name = ?2")?;
            let id = stmt
                .query_row([kind.as_str(), model_manifest.name], |row| {
                    row.get::<_, u32>(0)
                })
                .optional()?;
            Ok(id)
        })
    }

    // pub fn get_emb_by_file_path(
    //     &self,
    //     path: &Path,
    //     emb_type_id: Option<u32>,
    // ) -> Result<Option<FileEmbResult>> {
    //     self.with_conn(|conn| {
    //         let path_string = path.to_string_lossy();

    //         let sql = if emb_type_id.is_none() {
    //             indoc! {"
    //                 SELECT f.id, em.id, em.emb_type_id, em.last_file_mtime, em.last_file_size
    //                 FROM file f
    //                 LEFT JOIN emb_metadata em
    //                     ON em.file_id = f.id
    //                 WHERE f.path = ?1
    //             "}
    //         } else {
    //             indoc! {"
    //                 SELECT f.id, em.id, em.emb_type_id, em.last_file_mtime, em.last_file_size
    //                 FROM file f
    //                 LEFT JOIN emb_metadata em
    //                     ON em.file_id = f.id
    //                     AND em.emb_type_id = ?2
    //                 WHERE f.path = ?1
    //             "}
    //         };

    //         let mut stmt = conn.prepare(sql)?;
    //         let mut rows = if emb_type_id.is_none() {
    //             stmt.query(params![&path_string])?
    //         } else {
    //             stmt.query(params![&path_string, emb_type_id.unwrap()])?
    //         };

    //         let mut file_id: Option<i64> = None;
    //         let mut embeddings: Vec<EmbMetadata> = vec![];

    //         while let Some(row) = rows.next()? {
    //             file_id = row.get(0).ok();

    //             if let Some(emb_id) = row.get::<_, Option<i64>>(1)? {
    //                 embeddings.push(EmbMetadata {
    //                     id: emb_id,
    //                     file_id: file_id.unwrap(),
    //                     emb_type_id: row.get(2)?,
    //                     last_file_mtime: row.get(3)?,
    //                     last_file_size: row.get(4)?,
    //                 });
    //             }
    //         }

    //         Ok(file_id.map(|file_id| FileEmbResult {
    //             file_id,
    //             embeddings,
    //         }))
    //     })
    // }

    // pub fn insert_metadata(&self, files_metadata: Vec<FileMetadata>) -> Result<()> {
    //     self.with_conn(|conn| {
    //         for file_meta in files_metadata {
    //             for (key, value) in file_meta.key_value {
    //                 conn.execute(
    //                     indoc! {"
    //                         INSERT INTO file_metadata (file_id, key, value) VALUES (?1, ?2, ?3)
    //                         ON CONFLICT(file_id, key) DO UPDATE SET value = excluded.value
    //                     "},
    //                     params![file_meta.file_id, key, value],
    //                 )?;
    //             }
    //         }
    //         Ok(())
    //     })
    // }

    pub fn insert_embeddings(
        &self,
        emb_type_id: u32,
        files_embeddings: Vec<FileEmbedding>,
    ) -> Result<()> {
        self.with_conn(|conn| {
            let tx = conn.transaction()?;

            for file_emb in files_embeddings {
                let indexed_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_millis() as i64;

                let emb_id: i64 = tx.query_row(
                    indoc! {"
                        INSERT INTO emb_metadata (file_id, emb_type_id, last_file_mtime, last_file_size, indexed_at)
                        VALUES (?1, ?2, ?3, ?4, ?5)
                        ON CONFLICT(file_id, emb_type_id) DO UPDATE SET
                            last_file_mtime = excluded.last_file_mtime,
                            last_file_size = excluded.last_file_size,
                            indexed_at = excluded.indexed_at
                        RETURNING id
                    "},
                    params![file_emb.file_id, emb_type_id, file_emb.file_mtime, file_emb.file_size, indexed_at],
                    |row| row.get(0),
                )?;

                tx.execute(
                    &format!(
                        "INSERT OR REPLACE INTO vec_model{emb_type_id} (emb_id, vec) VALUES (?1, ?2)"
                    ),
                    params![emb_id, file_emb.embedding.as_bytes()],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
    }

    pub fn get_all_files_with_emb_status(
        &self,
        emb_type_id: u32,
    ) -> Result<Vec<FileWithEmbStatus>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(indoc! {"
                SELECT f.id, f.path, f.type, em.last_file_mtime, em.last_file_size
                FROM file f
                LEFT JOIN emb_metadata em ON em.file_id = f.id AND em.emb_type_id = ?1
            "})?;

            stmt.query_map(params![emb_type_id], |row| {
                Ok(FileWithEmbStatus {
                    id: row.get(0)?,
                    path: PathBuf::from(row.get::<_, String>(1)?),
                    file_type: row.get(2)?,
                    last_file_mtime: row.get(3)?,
                    last_file_size: row.get(4)?,
                })
            })?
            .map(|r| r.map_err(Into::into))
            .collect()
        })
    }

    pub fn search_embeddings(
        &self,
        query_vec: Vec<f32>,
        emb_type_id: u32,
        max_results: u32,
        threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        self.with_conn(|conn| {
            let sql = formatdoc!(
                "
                SELECT
                    file.id AS file_id,
                    file.path AS path,
                    file.type AS type,
                    1 - vec_distance_cosine(vec_table.vec, ?1) AS cosine_similarity
                FROM vec_model{emb_type_id} AS vec_table
                JOIN emb_metadata
                    ON emb_metadata.id = vec_table.emb_id
                JOIN file
                    ON file.id = emb_metadata.file_id
                WHERE
                    emb_metadata.emb_type_id = ?2
                    AND vec_table.vec MATCH ?1
                    AND k = ?3
                ORDER BY cosine_similarity DESC
                "
            );

            let mut stmt = conn.prepare(&sql)?;
            let results = stmt
                .query_map(
                    params![query_vec.as_bytes(), emb_type_id, max_results],
                    |row| {
                        let path = row.get::<_, String>(1)?;
                        let file_type = row.get::<_, String>(2)?;
                        let score = row.get::<_, f32>(3)?;
                        let filename = Path::new(&path)
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("")
                            .to_string();
                        Ok(SearchResult {
                            path,
                            filename,
                            file_type,
                            score,
                        })
                    },
                )?
                .filter_map(|r| r.ok())
                .filter(|r| r.score >= threshold)
                .collect();

            Ok(results)
        })
    }
}
