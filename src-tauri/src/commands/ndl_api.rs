use crate::models::BookInfoFromApi;
use quick_xml::events::Event;
use quick_xml::Reader;

#[tauri::command]
pub async fn fetch_book_info_from_ndl(isbn: String) -> Result<BookInfoFromApi, String> {
    let url = format!(
        "https://ndlsearch.ndl.go.jp/api/sru?operation=searchRetrieve&version=1.2&recordSchema=dcndl&query=isbn={}",
        isbn
    );

    let xml = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut record_data_content = None;
    let mut in_record_data = false;

    // 1. <recordData> の中身をテキストとして取得
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"recordData" => {
                in_record_data = true;
            }
            Ok(Event::Text(e)) if in_record_data => {
                record_data_content = Some(e.unescape().unwrap().into_owned());
                // recordDataを見つけたら、外側のループは抜けてOK
                break;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Outer XML parsing error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    if let Some(escaped_xml) = record_data_content {
        // 2. 取得したテキストを再度XMLとしてパース
        let mut inner_reader = Reader::from_str(&escaped_xml);
        inner_reader.trim_text(true);
        let mut inner_buf = Vec::new();
        let mut title = None;
        let mut creator = None;
        let mut publisher = None;

        let mut in_title = false;
        let mut in_foaf_name = false; // foaf:nameタグの中にいるか
        // コンテキストを追跡するフラグ
        let mut in_creator_context = false;
        let mut in_publisher_context = false;

        loop {
            match inner_reader.read_event_into(&mut inner_buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"dcterms:title" => in_title = true,
                    b"dcterms:creator" => in_creator_context = true,
                    b"dcterms:publisher" => in_publisher_context = true,
                    b"foaf:name" => in_foaf_name = true,
                    _ => {}
                },
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"dcterms:creator" => in_creator_context = false,
                    b"dcterms:publisher" => in_publisher_context = false,
                    b"foaf:name" => in_foaf_name = false,
                    _ => {}
                },
                Ok(Event::Text(e)) => {
                    if in_title {
                        if title.is_none() {
                            title = Some(e.unescape().unwrap().into_owned());
                        }
                        in_title = false;
                    } else if in_foaf_name {
                        if in_creator_context {
                            if creator.is_none() {
                                creator = Some(e.unescape().unwrap().into_owned());
                            }
                        } else if in_publisher_context {
                            if publisher.is_none() {
                                publisher = Some(e.unescape().unwrap().into_owned());
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("Inner XML parsing error: {}", e)),
                _ => {}
            }
            inner_buf.clear();
        }

        if title.is_some() && creator.is_some() && publisher.is_some() {
            Ok(BookInfoFromApi {
                title: title.unwrap(),
                author: creator.unwrap(),
                publisher: publisher.unwrap(),
            })
        } else {
            let mut missing_items = Vec::new();
            if title.is_none() {
                missing_items.push("title");
            }
            if creator.is_none() {
                missing_items.push("creator");
            }
            if publisher.is_none() {
                missing_items.push("publisher");
            }
            Err(format!(
                "書籍情報が見つかりませんでした。不足: [{}]",
                missing_items.join(", ")
            ))
        }
    } else {
        Err("recordDataが見つかりませんでした。".to_string())
    }
}