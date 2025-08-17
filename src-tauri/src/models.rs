use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Genre {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
    pub id: i64,
    pub isbn: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub price: Option<i64>,
    pub c_code: Option<String>,
    pub is_read: i64,
    pub genre_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewBook {
    pub title: String,
    pub genre_id: Option<i64>,
    pub isbn: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub price: Option<i64>,
    pub c_code: Option<String>,
    pub is_read: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBook {
    pub id: i64,
    pub isbn: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub price: Option<i64>,
    pub c_code: Option<String>,
    pub is_read: i64,
    pub genre_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BookInfoFromApi {
    pub title: String,
    pub author: String,
    pub publisher: String,
}
