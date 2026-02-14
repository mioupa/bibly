use crate::models::BookInfoFromApi;
use serde_json::Value;

#[tauri::command]
pub async fn fetch_book_info_from_google_books(
    isbn: String,
    api_key: Option<String>,
) -> Result<BookInfoFromApi, String> {
    let isbn = isbn.trim();
    if isbn.is_empty() {
        return Err("ISBNを入力してください".to_string());
    }

    let mut url = format!("https://www.googleapis.com/books/v1/volumes?q=isbn:{}", isbn);
    if let Some(api_key) = api_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
    {
        url.push_str("&key=");
        url.push_str(api_key);
    }

    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "Google Books APIの呼び出しに失敗しました (HTTP {})",
            resp.status()
        ));
    }
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(items) = json["items"].as_array() {
        for item in items {
            if let Some(book_info) = parse_item(item) {
                return Ok(book_info);
            }
        }
        Err("書籍情報が見つかりませんでした".to_string())
    } else {
        Err("書籍情報が見つかりませんでした".to_string())
    }
}

fn parse_item(item: &Value) -> Option<BookInfoFromApi> {
    let volume_info = &item["volumeInfo"];
    let title = volume_info["title"].as_str().unwrap_or("").trim().to_string();
    if title.is_empty() {
        return None;
    }

    let author = volume_info["authors"]
        .as_array()
        .map(|authors| {
            authors
                .iter()
                .filter_map(|v| v.as_str().map(str::trim))
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    let publisher = volume_info["publisher"]
        .as_str()
        .map(str::trim)
        .unwrap_or("")
        .to_string();

    Some(BookInfoFromApi {
        title,
        author,
        publisher,
    })
}
