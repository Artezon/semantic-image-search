use std::sync::Mutex;

use anyhow::Result;
use include_dir::{include_dir, Dir};
use rusqlite::{ffi::sqlite3_auto_extension, Connection};
use rusqlite_migration::Migrations;
use sqlite_vec::sqlite3_vec_init;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

pub struct Database {
    conn: Mutex<Connection>,
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

        let _ = Migrations::from_directory(&MIGRATIONS_DIR)?.to_latest(&mut conn);

        // print!(
        //     "{}",
        //     conn.query_row("SELECT vec_version()", [], |row| row.get::<_, String>(0))?
        // );

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}
