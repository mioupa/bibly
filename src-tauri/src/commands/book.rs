use crate::db::DbConnection;
use crate::models::{Book, NewBook, UpdateBook};
use tauri::State;

#[tauri::command]
pub fn get_all_books(db: State<DbConnection>) -> Result<Vec<Book>, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, isbn, title, author, publisher, price, c_code, is_read, genre_id FROM books")
        .map_err(|e| e.to_string())?;
    let iter = stmt.query_map([], row_to_book).map_err(|e| e.to_string())?;
    collect_books(iter)
}

#[tauri::command]
pub fn get_books_by_genre(genre_id: i64, db: State<DbConnection>) -> Result<Vec<Book>, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, isbn, title, author, publisher, price, c_code, is_read, genre_id FROM books WHERE genre_id = ?1")
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map([genre_id], row_to_book)
        .map_err(|e| e.to_string())?;
    collect_books(iter)
}

#[tauri::command]
pub fn add_book(new_book: NewBook, db: State<DbConnection>) -> Result<Book, String> {
    if new_book.title.trim().is_empty() {
        return Err("タイトル必須".into());
    }
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
    {
        let mut stmt = conn.prepare(
            "SELECT id, isbn, title, author, publisher, price, c_code, is_read, genre_id FROM books WHERE id = ?1"
        ).map_err(|e| e.to_string())?;
        let book = stmt
            .query_row([id], row_to_book)
            .map_err(|e| e.to_string())?;
        Ok(book)
    }
}

#[tauri::command]
pub fn update_book(book: UpdateBook, db: State<DbConnection>) -> Result<Book, String> {
    if book.title.trim().is_empty() {
        return Err("タイトル必須".into());
    }
    let conn = db.0.lock().unwrap();
    conn.execute(
        "UPDATE books SET
            isbn = ?1,
            title = ?2,
            author = ?3,
            publisher = ?4,
            price = ?5,
            c_code = ?6,
            is_read = ?7,
            genre_id = ?8
         WHERE id = ?9",
        rusqlite::params![
            book.isbn,
            book.title,
            book.author,
            book.publisher,
            book.price,
            book.c_code,
            book.is_read,
            book.genre_id,
            book.id
        ],
    )
    .map_err(|e| e.to_string())?;
    {
        let mut stmt = conn.prepare(
            "SELECT id, isbn, title, author, publisher, price, c_code, is_read, genre_id FROM books WHERE id = ?1"
        ).map_err(|e| e.to_string())?;
        let updated = stmt
            .query_row([book.id], row_to_book)
            .map_err(|e| e.to_string())?;
        Ok(updated)
    }
}

#[tauri::command]
pub fn delete_book(id: i64, db: State<DbConnection>) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    match crate::db::delete_book(&conn, id) {
        Ok(affected) if affected > 0 => Ok(()),
        Ok(_) => Err(format!("Book with id {} not found", id)),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn get_book_count_by_genre(genre_id: i64, db: State<DbConnection>) -> Result<i64, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM books WHERE genre_id = ?1")
        .map_err(|e| e.to_string())?;
    let count: i64 = stmt
        .query_row([genre_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count)
}

fn row_to_book(row: &rusqlite::Row) -> rusqlite::Result<Book> {
    Ok(Book {
        id: row.get(0)?,
        isbn: row.get(1)?,
        title: row.get(2)?,
        author: row.get(3)?,
        publisher: row.get(4)?,
        price: row.get(5)?,
        c_code: row.get(6)?,
        is_read: row.get(7)?,
        genre_id: row.get(8)?,
    })
}

fn collect_books<I>(iter: I) -> Result<Vec<Book>, String>
where
    I: Iterator<Item = Result<Book, rusqlite::Error>>,
{
    let mut v = Vec::new();
    for b in iter {
        v.push(b.map_err(|e| e.to_string())?);
    }
    Ok(v)
}
