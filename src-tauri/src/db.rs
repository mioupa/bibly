use rusqlite::Connection;
use std::{fs, sync::Mutex};

pub struct DbConnection(pub Mutex<Connection>);

pub fn setup_database(_app: &tauri::AppHandle) -> Result<Connection, rusqlite::Error> {
    let exe_path = std::env::current_exe().expect("Failed to get current exe path");
    let app_dir = exe_path
        .parent()
        .expect("Failed to get parent dir")
        .to_path_buf();
    let data_dir = app_dir.join("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).expect("Failed to create data dir");
    }
    let db_path = data_dir.join("bibly.sqlite");
    println!("Database (Portable) will be created at: {:?}", db_path);
    let conn = Connection::open(&db_path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS genres (
            id      INTEGER PRIMARY KEY,
            name    TEXT NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS books (
            id          INTEGER PRIMARY KEY,
            isbn        TEXT,
            title       TEXT NOT NULL,
            author      TEXT,
            publisher   TEXT,
            price       INTEGER,
            c_code      TEXT,
            is_read     INTEGER NOT NULL DEFAULT 0,
            genre_id    INTEGER,
            FOREIGN KEY (genre_id) REFERENCES genres (id)
        );
        ",
    )?;
    Ok(conn)
}
