use crossbeam::channel;
use rusqlite::Connection;

mod characters;
mod text;
mod common;

use std::{fs::{self, File}, path::PathBuf, thread};

use characters::populate_character_data;
use text::populate_text_data;

const DATABASE_ROOT: &str = "data";

const DATABASES: [&str; 11] = [
    "characters.sqlite",
    "skills.sqlite",
    "text/de.sqlite",
    "text/en.sqlite",
    "text/es.sqlite",
    "text/fr.sqlite",
    "text/it.sqlite",
    "text/ja.sqlite",
    "text/pt.sqlite",
    "text/zh_hans.sqlite",
    "text/zh_hant.sqlite",
];

fn main() {
    create_required_files();

    let (char_name_req_tx, char_name_req_rx) = channel::unbounded();
    
    let mut path = PathBuf::from(DATABASE_ROOT);
    path.push(DATABASES[0]);

    let char_database = Connection::open(path).unwrap(); 
    let character_thread = thread::spawn(|| {
        populate_character_data(char_database, char_name_req_tx);
        
    });

    let text_databases: Vec<Connection> = DATABASES[2..].iter()
        .map(|p| {
            let mut path = PathBuf::from(DATABASE_ROOT);
            path.push(p); 
            Connection::open(path).unwrap()
        }).collect();

    let text_thread = thread::spawn(|| {
        populate_text_data(text_databases, char_name_req_rx);
    });
        
    let _ = character_thread.join();
}

fn create_required_files() {
    let root = PathBuf::from(DATABASE_ROOT);

    if let Ok(true) = fs::exists(&root) {
        fs::remove_dir_all(&root).unwrap();
    }

    fs::create_dir(&root).unwrap();

    for database_path in DATABASES {
        let mut clone = root.clone();
        clone.push(database_path);

        let parent = clone.parent().unwrap();
        fs::create_dir_all(parent).unwrap();
        
        File::create(&clone).unwrap();
    }
}