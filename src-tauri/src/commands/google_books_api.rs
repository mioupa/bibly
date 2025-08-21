use crate::models::BookInfoFromApi;
use serde_json::Value;

#[tauri::command]
pub async fn fetch_book_info_from_google_books(
    isbn: String,
    api_key: String,
) -> Result<BookInfoFromApi, String> {
    let url = format!(
        "https://www.googleapis.com/books/v1/volumes?q=isbn:{}&key={}",
        isbn, api_key
    );

    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(item) = json["items"].as_array().and_then(|arr| arr.first()) {
        let volume_info = &item["volumeInfo"];
        let title = volume_info["title"].as_str().unwrap_or("").to_string();
        let author = volume_info["authors"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let publisher = volume_info["publisher"].as_str().unwrap_or("").to_string();

        if title.is_empty() || author.is_empty() || publisher.is_empty() {
            Err("書籍情報が不足しています".to_string())
        } else {
            Ok(BookInfoFromApi {
                title,
                author,
                publisher,
            })
        }
    } else {
        Err("書籍情報が見つかりませんでした".to_string())
    }
}
