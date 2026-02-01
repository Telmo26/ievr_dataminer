use crossbeam::channel::Receiver;
use rusqlite::{Connection, Result};

pub fn populate_text_data(text_databases: Vec<Connection>, char_name_req_rx: Receiver<i32>) {
    initialize_databases(text_databases).unwrap();

    while let Ok(request) = char_name_req_rx.recv() {
        
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