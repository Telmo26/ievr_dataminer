use std::path::Path;

use crossbeam::channel::Receiver;
use rusqlite::{Connection, Result};

pub const TEXT_LANGUAGES: [&str; 9] = [
    "de", "en", "es", "fr", "it", "ja", "pt", "zh_hans", "zh_hant"
];

pub const TEXT_ROOT_PATH: &str = "data/common/text";

pub const TEXT_REQUIRED_FILES: [&str; 3] = [
    "^chara_text.cfg.bin$", "^chara_text_roma.cfg.bin$", "^skill_text.cfg.bin$"
];

pub fn populate_text_data(extraction_path: &Path, text_database_connections: Vec<Connection>, requested_files: Vec<String>, char_name_req_rx: Receiver<i32>) {
    initialize_databases(text_database_connections).unwrap();

    while let Ok(_request) = char_name_req_rx.recv() {
        
    }
}

fn initialize_databases(text_databases: Vec<Connection>) -> Result<()> {
    for database in text_databases {
        database.execute(
                "CREATE TABLE character_names (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )", 
            ()
        )?;

        database.execute(
                "CREATE TABLE skill_names (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )", 
            ()
        )?;
    };

    Ok(())
}