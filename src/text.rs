use std::{collections::HashMap, path::Path, sync::LazyLock};

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

pub const TEXT_REQUIRED_FILES: LazyLock<HashMap<&'static str, HashMap<&'static str, &'static str>>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(TEXT_LANGUAGES.len());

    for language in TEXT_LANGUAGES {
        map.insert(language, HashMap::new());
        let language_map = map.get_mut(language).unwrap();

        language_map.insert("chara_add_info",       "^chara_add_info_text.cfg.bin$");
        language_map.insert("chara_description",    "^chara_description_text.cfg.bin$");
        language_map.insert("chara_text",           "^chara_text.cfg.bin$");
        language_map.insert("chara_text_roma",      "^chara_text_roma.cfg.bin$");
        language_map.insert("skill_text",           "^skill_text.cfg.bin$");
    };
    map
});

pub fn populate_text_data(extraction_path: &Path, text_database_connections: HashMap<&'static str, Connection>, requested_files: HashMap<&'static str, HashMap<&'static str, String>>, char_name_req_rx: Receiver<(i32, i32)>) {
    assert!(text_database_connections.len() == TEXT_LANGUAGES.len());

    #[cfg(debug_assertions)]
    println!("{:#?}", requested_files);

    let root_path = extraction_path.join(TEXT_ROOT_PATH);

    let mut databases: Vec<TextDatabase> = text_database_connections.into_par_iter().map(|(language, conn)| {
        TextDatabase::init( 
            conn,
            parse_gamefile(&root_path.join(language).join(&requested_files[language]["chara_text"])).unwrap(), 
            parse_gamefile(&root_path.join(language).join(&requested_files[language]["chara_text_roma"])).unwrap(), 
            parse_gamefile(&root_path.join(language).join(&requested_files[language]["chara_description"])).unwrap(),
            parse_gamefile(&root_path.join(language).join(&requested_files[language]["chara_add_info"])).unwrap(),
            parse_gamefile(&root_path.join(language).join(&requested_files[language]["skill_text"])).unwrap()
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