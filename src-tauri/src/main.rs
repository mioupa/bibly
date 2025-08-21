// Modularized main: database, models, and commands split into separate files.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;

use db::{setup_database, DbConnection};
use std::sync::Mutex;
use tauri::Manager; // for app.manage

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let conn = setup_database(&app.handle()).expect("Failed to setup database");
            app.manage(DbConnection(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_genres,
            commands::get_books_by_genre,
            commands::get_all_books,
            commands::add_book,
            commands::update_book,
            commands::add_genre,
            commands::fetch_book_info_from_ndl,
            commands::fetch_book_info_from_amazon,
            commands::delete_book,
            commands::get_book_count_by_genre,
            commands::delete_genre,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
