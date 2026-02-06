use std::{path::Path};

use crossbeam::channel::Receiver;
use rayon::prelude::*;
use rusqlite::Connection;

mod text_database;

use text_database::TextDatabase;

use crate::{
    common::parse_gamefile,
};

pub const TEXT_LANGUAGES: [&str; 9] = [
    "de", "en", "es", "fr", "it", "ja", "pt", "zh_hans", "zh_hant"
];

pub const TEXT_ROOT_PATH: &str = "data/common/text";

pub const TEXT_REQUIRED_FILES: [&str; 5] = [
    "^chara_add_info_text.cfg.bin$",
    "^chara_description_text.cfg.bin$",
    "^chara_text.cfg.bin$", 
    "^chara_text_roma.cfg.bin$", 
    "^skill_text.cfg.bin$"
];

pub fn populate_text_data(extraction_path: &Path, text_database_connections: Vec<Connection>, requested_files: Vec<String>, char_name_req_rx: Receiver<(i32, i32)>) {
    assert!(text_database_connections.len() == TEXT_LANGUAGES.len());

    #[cfg(debug_assertions)]
    println!("{:#?}", requested_files);

    let root_path = extraction_path.join(TEXT_ROOT_PATH);

    let mut databases: Vec<TextDatabase> = text_database_connections.into_par_iter().enumerate().map(|(i, conn)| {
        TextDatabase::init( 
            conn,
            parse_gamefile(&root_path.join(&requested_files[TEXT_REQUIRED_FILES.len() * i + 2])).unwrap(), 
            parse_gamefile(&root_path.join(&requested_files[TEXT_REQUIRED_FILES.len() * i + 3])).unwrap(), 
            parse_gamefile(&root_path.join(&requested_files[TEXT_REQUIRED_FILES.len() * i + 1])).unwrap(),
            parse_gamefile(&root_path.join(&requested_files[TEXT_REQUIRED_FILES.len() * i + 0])).unwrap(),
            parse_gamefile(&root_path.join(&requested_files[TEXT_REQUIRED_FILES.len() * i + 4])).unwrap()
        )
    }).collect();

    let mut char_requests = Vec::with_capacity(1000);

    while let Ok(char_request) = char_name_req_rx.recv() {
        char_requests.push(char_request);

        if char_requests.len() >= 1000 {
            databases.par_iter_mut().for_each(|d| {
                d.write_character(&char_requests);
                d.write_character_roma(&char_requests);
            });

            char_requests.clear();
        }
    }

    // Flush remaining requests
    if !char_requests.is_empty() {
        databases.par_iter_mut().for_each(|d| {
            d.write_character(&char_requests);
            d.write_character_roma(&char_requests);
        });
    }

    println!("[TEXT]: {} requested name(s) not found.", databases[0].get_missing_names());
}