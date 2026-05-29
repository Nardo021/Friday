use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::errors::{AppError, AppResult};

const SCHEMA_VERSION: i32 = 4;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: PathBuf) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path).map_err(|e| AppError::Storage(e.to_string()))?;
        conn.busy_timeout(std::time::Duration::from_millis(5_000))
            .map_err(|e| AppError::Storage(e.to_string()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn migrate(&self) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap_or(0);

        if version < 1 {
            conn.execute_batch(
                "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                project_type TEXT,
                trusted INTEGER NOT NULL DEFAULT 0,
                default_adapter_id TEXT,
                created_at TEXT NOT NULL,
                last_used_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                project_id TEXT,
                adapter_id TEXT NOT NULL,
                title TEXT NOT NULL,
                prompt TEXT,
                status TEXT NOT NULL,
                cwd TEXT,
                pid INTEGER,
                summary TEXT,
                created_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS commands (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                command TEXT NOT NULL,
                cwd TEXT NOT NULL,
                risk TEXT NOT NULL,
                exit_code INTEGER,
                started_at TEXT,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS file_changes (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                path TEXT NOT NULL,
                action TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS approvals (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                title TEXT NOT NULL,
                command TEXT,
                risk TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value_json TEXT NOT NULL
            );
            ",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        }

        if version < 2 {
            migrate_v2(&conn)?;
        }

        if version < 3 {
            migrate_v3(&conn)?;
        }

        if version < 4 {
            migrate_v4(&conn)?;
        }

        conn.pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|e| AppError::Storage(e.to_string()))?;

        conn.execute_batch(
            "
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA cache_size=-64000;
            PRAGMA temp_store=MEMORY;
            CREATE INDEX IF NOT EXISTS idx_session_events_session_created
                ON session_events(session_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_messages_session_created
                ON messages(session_id, created_at);
            ",
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;

        Ok(())
    }

    pub fn with_conn<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>,
    {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        f(&conn)
    }
}

fn migrate_v2(conn: &Connection) -> AppResult<()> {
    let add_column = |sql: &str| {
        if let Err(e) = conn.execute(sql, []) {
            let msg = e.to_string();
            if !msg.contains("duplicate column") {
                return Err(AppError::Storage(msg));
            }
        }
        Ok(())
    };

    add_column("ALTER TABLE sessions ADD COLUMN session_type TEXT")?;
    add_column("ALTER TABLE sessions ADD COLUMN ownership TEXT")?;
    add_column("ALTER TABLE sessions ADD COLUMN control_level TEXT")?;
    add_column("ALTER TABLE sessions ADD COLUMN updated_at TEXT")?;
    add_column("ALTER TABLE sessions ADD COLUMN repo_json TEXT")?;
    add_column("ALTER TABLE sessions ADD COLUMN process_json TEXT")?;
    add_column("ALTER TABLE sessions ADD COLUMN cloud_json TEXT")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS session_events (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            type TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS cloud_agents (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            agent_id TEXT,
            run_id TEXT,
            pr_url TEXT,
            status TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS processes (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            pid INTEGER,
            pty_id TEXT,
            exe_name TEXT,
            cwd TEXT,
            created_at TEXT NOT NULL
        );
        ",
    )
    .map_err(|e| AppError::Storage(e.to_string()))?;

    let events_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='events'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    if events_exists {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO session_events SELECT * FROM events",
            [],
        );
    }

    Ok(())
}

fn migrate_v4(conn: &Connection) -> AppResult<()> {
    if let Err(e) = conn.execute("ALTER TABLE projects ADD COLUMN remote_url TEXT", []) {
        let msg = e.to_string();
        if !msg.contains("duplicate column") {
            return Err(AppError::Storage(msg));
        }
    }
    Ok(())
}

fn migrate_v3(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS ideas (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            project_id TEXT,
            session_id TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS queued_instructions (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            text TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        ",
    )
    .map_err(|e| AppError::Storage(e.to_string()))?;
    Ok(())
}

pub fn app_data_dir() -> AppResult<PathBuf> {
    crate::storage::local_data::app_data_dir()
}

pub fn db_path() -> AppResult<PathBuf> {
    Ok(app_data_dir()?.join("friday.db"))
}

pub fn logs_dir() -> AppResult<PathBuf> {
    Ok(app_data_dir()?.join("logs"))
}
