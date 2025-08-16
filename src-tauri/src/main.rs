// src-tauri/src/main.rs

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;
use tauri::Manager;
use tauri::State;

// データベース接続を保持するための構造体
pub struct DbConnection(pub Mutex<Connection>);

// フロントエンドに渡すGenreのデータ構造
#[derive(Debug, Serialize, Deserialize)]
pub struct Genre {
    id: i64,
    name: String,
}

// フロントエンドに渡すBookのデータ構造 (項目を増やす)
#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
    id: i64,
    isbn: Option<String>,
    title: String,
    author: Option<String>,
    publisher: Option<String>,
    price: Option<i64>,
    c_code: Option<String>, // ← 追加
    is_read: i64,
}

// 新規書籍登録用のデータ構造
#[derive(Debug, Serialize, Deserialize)]
pub struct NewBook {
    title: String,
    genre_id: i64,
    isbn: Option<String>,
    author: Option<String>,
    publisher: Option<String>,
    price: Option<i64>,
    c_code: Option<String>, // ← 追加
    is_read: Option<i64>,
}

// 既存書籍更新用
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBook {
    id: i64,
    isbn: Option<String>,
    title: String,
    author: Option<String>,
    publisher: Option<String>,
    price: Option<i64>,
    c_code: Option<String>,
    is_read: i64,
}

// ジャンル一覧を取得するコマンド
#[tauri::command]
fn get_genres(db: State<DbConnection>) -> Result<Vec<Genre>, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name FROM genres ORDER BY id") // ORDER BYを追加
        .map_err(|e| e.to_string())?;
    let genre_iter = stmt
        .query_map([], |row| {
            Ok(Genre {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut genres = Vec::new();
    for genre in genre_iter {
        genres.push(genre.map_err(|e| e.to_string())?);
    }
    Ok(genres)
}

// ★★★ 新しく追加: 全ての書籍を取得するコマンド ★★★
#[tauri::command]
fn get_all_books(db: State<DbConnection>) -> Result<Vec<Book>, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, isbn, title, author, publisher, price, c_code, is_read FROM books")
        .map_err(|e| e.to_string())?;
    let book_iter = stmt
        .query_map([], |row| {
            Ok(Book {
                id: row.get(0)?,
                isbn: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                publisher: row.get(4)?,
                price: row.get(5)?,
                c_code: row.get(6)?,
                is_read: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut books = Vec::new();
    for book in book_iter {
        books.push(book.map_err(|e| e.to_string())?);
    }
    Ok(books)
}

// 指定されたジャンルの書籍一覧を取得するコマンド (取得項目を増やす)
#[tauri::command]
fn get_books_by_genre(genre_id: i64, db: State<DbConnection>) -> Result<Vec<Book>, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, isbn, title, author, publisher, price, c_code, is_read FROM books WHERE genre_id = ?1")
        .map_err(|e| e.to_string())?;
    let book_iter = stmt
        .query_map([genre_id], |row| {
            Ok(Book {
                id: row.get(0)?,
                isbn: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                publisher: row.get(4)?,
                price: row.get(5)?,
                c_code: row.get(6)?,
                is_read: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut books = Vec::new();
    for book in book_iter {
        books.push(book.map_err(|e| e.to_string())?);
    }
    Ok(books)
}

// 新しい書籍を追加するコマンド
#[tauri::command]
fn add_book(new_book: NewBook, db: State<DbConnection>) -> Result<Book, String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO books (title, genre_id, isbn, author, publisher, price, c_code, is_read)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, COALESCE(?8,0))",
        rusqlite::params![
            new_book.title,
            new_book.genre_id,
            new_book.isbn,
            new_book.author,
            new_book.publisher,
            new_book.price,
            new_book.c_code,
            new_book.is_read
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let mut stmt = conn.prepare(
        "SELECT id, isbn, title, author, publisher, price, c_code, is_read FROM books WHERE id = ?1"
    ).map_err(|e| e.to_string())?;
    let book = stmt
        .query_row([id], |row| {
            Ok(Book {
                id: row.get(0)?,
                isbn: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                publisher: row.get(4)?,
                price: row.get(5)?,
                c_code: row.get(6)?,
                is_read: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(book)
}

// 書籍情報を更新するコマンド
#[tauri::command]
fn update_book(book: UpdateBook, db: State<DbConnection>) -> Result<Book, String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "UPDATE books
         SET isbn = ?1,
             title = ?2,
             author = ?3,
             publisher = ?4,
             price = ?5,
             c_code = ?6,
             is_read = ?7
         WHERE id = ?8",
        rusqlite::params![
            book.isbn,
            book.title,
            book.author,
            book.publisher,
            book.price,
            book.c_code,
            book.is_read,
            book.id
        ],
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, isbn, title, author, publisher, price, c_code, is_read FROM books WHERE id = ?1"
    ).map_err(|e| e.to_string())?;
    let updated = stmt
        .query_row([book.id], |row| {
            Ok(Book {
                id: row.get(0)?,
                isbn: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                publisher: row.get(4)?,
                price: row.get(5)?,
                c_code: row.get(6)?,
                is_read: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(updated)
}

fn setup_database(_app: &tauri::AppHandle) -> Result<Connection, rusqlite::Error> {
    // 実行ファイル(.exe)のフルパスを取得
    let exe_path = std::env::current_exe().expect("Failed to get current exe path");
    // .exeファイルが存在するディレクトリのパスを取得
    let app_dir = exe_path
        .parent()
        .expect("Failed to get parent dir")
        .to_path_buf();

    // データ保存用のサブディレクトリを定義
    let data_dir = app_dir.join("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).expect("Failed to create data dir");
    }

    // データベースファイルのフルパスを定義
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

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // データベース接続をセットアップし、TauriのStateとして管理する
            let conn = setup_database(&app.handle()).expect("Failed to setup database");
            app.manage(DbConnection(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 作成したコマンドをここに登録
            get_genres,
            get_books_by_genre,
            get_all_books,
            add_book,
            update_book,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
