use rusqlite::{Connection, OptionalExtension};
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

pub fn delete_book(conn: &Connection, id: i64) -> Result<usize, rusqlite::Error> {
    let affected_rows = conn.execute("DELETE FROM books WHERE id = ?", [id])?;
    Ok(affected_rows)
}

pub fn delete_genre_and_unassign_books(conn: &mut Connection, genre_id: i64) -> Result<(), rusqlite::Error> {
    let tx = conn.transaction()?;

    // 1. Ensure "未分類" genre exists and resolve its id.
    let mut unclassified_id = tx
        .query_row("SELECT id FROM genres WHERE name = '未分類'", [], |row| row.get::<_, i64>(0))
        .optional()?;

    if unclassified_id.is_none() {
        tx.execute("INSERT INTO genres (name) VALUES (?1)", ["未分類"])?;
        unclassified_id = Some(tx.last_insert_rowid());
    }

    // 2. Move books to "未分類" before deleting the genre.
    tx.execute(
        "UPDATE books SET genre_id = ?1 WHERE genre_id = ?2",
        [unclassified_id.unwrap(), genre_id],
    )?;

    // 3. Delete the genre itself.
    tx.execute("DELETE FROM genres WHERE id = ?", [genre_id])?;

    tx.commit()
}
