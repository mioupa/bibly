use crate::db::DbConnection;
use crate::models::Genre;
use tauri::State;

#[tauri::command]
pub fn get_genres(db: State<DbConnection>) -> Result<Vec<Genre>, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name FROM genres ORDER BY name")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    let mut genres = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        genres.push(Genre {
            id: row.get(0).map_err(|e| e.to_string())?,
            name: row.get(1).map_err(|e| e.to_string())?,
        });
    }
    Ok(genres)
}

#[tauri::command]
pub fn add_genre(name: String, db: State<DbConnection>) -> Result<Genre, String> {
    if name.trim().is_empty() {
        return Err("ジャンル名が空です".into());
    }
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO genres (name) VALUES (?1)",
        rusqlite::params![name],
    )
    .map_err(|e| e.to_string())?;
    {
        let mut stmt = conn
            .prepare("SELECT id, name FROM genres WHERE name = ?1")
            .map_err(|e| e.to_string())?;
        let genre = stmt
            .query_row([&name], |row| {
                Ok(Genre {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?;
        Ok(genre)
    }
}

#[tauri::command]
pub fn delete_genre(genre_id: i64, db: State<DbConnection>) -> Result<(), String> {
    let mut conn = db.0.lock().unwrap();
    crate::db::delete_genre_and_unassign_books(&mut conn, genre_id)
        .map_err(|e| e.to_string())
}
