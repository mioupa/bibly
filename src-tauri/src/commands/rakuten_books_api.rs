use crate::models::BookInfoFromApi;
use serde_json::Value;

#[tauri::command]
pub async fn fetch_book_info_from_rakuten(
    isbn: String,
    application_id: String,
) -> Result<BookInfoFromApi, String> {
    let url = format!(
        "https://app.rakuten.co.jp/services/api/BooksBook/Search/20170404?applicationId={}&isbn={}",
        application_id, isbn
    );

    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(items) = json["Items"].as_array() {
        if let Some(item) = items
            .first()
            .and_then(|i| i.get("Item"))
        {
            let title = item
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let author = item
                .get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let publisher = item
                .get("publisherName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if !title.is_empty() && !author.is_empty() && !publisher.is_empty() {
                return Ok(BookInfoFromApi {
                    title,
                    author,
                    publisher,
                });
            } else {
                return Err("書籍情報が不足しています".to_string());
            }
        }
    }

    Err("書籍情報が見つかりませんでした".to_string())
}

