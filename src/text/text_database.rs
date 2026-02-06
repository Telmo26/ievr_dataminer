use std::{collections::HashMap};

use ievr_cfg_bin_editor_core::Database;
use rusqlite::{Connection, params};

use crate::common::{parse_int_value, parse_string_value};

pub struct TextDatabase {
    conn: Connection,
    chara_names: HashMap<i32, String>,
    chara_roma_names: HashMap<i32, String>,
    chara_descriptions: HashMap<i32, String>,
    // skill_text: HashMap<i32, (String, String)>,

    missing_character_names: u32,
}

impl TextDatabase {
    pub fn init(mut conn: Connection, 
        chara_text: Database, 
        chara_text_roma: Database, 
        chara_description: Database, 
        chara_add_info: Database,
        _skill_text: Database
    ) -> TextDatabase {
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "synchronous", "NORMAL").unwrap();

        Self::initialize_database(&conn);

        // Computing the character hash table
        let chara_table = chara_text.table("NOUN_INFO").unwrap();
        let rows = chara_table.rows();
        
        let mut chara_names = HashMap::with_capacity(rows.len());
        for row in rows {
            let index = parse_int_value(&row.values[0][0]);
            let string = parse_string_value(&row.values[5][0]);

            if parse_int_value(&row.values[1][0]) == 0 { // This value is different from 0 when texts are alternatives of the main one
                if chara_names.insert(index, string).is_some() {
                    println!("Text index {index} in double");
                }
            }
        };

        // Computing the character roma hash table
        let chara_roma_table = chara_text_roma.table("NOUN_INFO").unwrap();
        let rows = chara_roma_table.rows();

        let mut chara_roma_names = HashMap::with_capacity(rows.len());
        for row in rows {
            let index = parse_int_value(&row.values[0][0]);
            let string = parse_string_value(&row.values[5][0]);

            if parse_int_value(&row.values[1][0]) == 0 { // This value is different from 0 when texts are alternatives of the main one
                if chara_roma_names.insert(index, string).is_some() {
                    println!("Text index {index} in double");
                }
            }
        }

        // Computing the character description table
        let chara_desc_table = chara_description.table("TEXT_INFO").unwrap();
        let rows = chara_desc_table.rows();

        let mut chara_descriptions = HashMap::with_capacity(rows.len());
        for row in rows {
            let index = parse_int_value(&row.values[0][0]);
            let string = parse_string_value(&row.values[2][0]);

            if chara_descriptions.insert(index, string).is_some() {
                println!("Character description {index} in double");
            }
        }

        // Inserting the series' names into the database
        let series_table = chara_add_info.table("NOUN_INFO").unwrap();
        Self::insert_series(&mut conn, series_table);
        
        TextDatabase { conn, chara_names, chara_roma_names, chara_descriptions, missing_character_names: 0 }
    }

    pub fn write_character(&mut self, index_batch: &Vec<(i32, i32)>) {
        let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive).unwrap();

        {
            let mut name_stmt = tx.prepare_cached("
                INSERT INTO character_names (id, name) 
                VALUES (?1, ?2) 
                ON CONFLICT(id) DO NOTHING
            ").unwrap();

            let mut desc_stmt = tx.prepare_cached("
                INSERT INTO character_descriptions (id, description) 
                VALUES (?1, ?2) 
                ON CONFLICT(id) DO NOTHING
            ").unwrap();
            
            for (chara_index, chara_desc) in index_batch {
                match self.chara_names.get(chara_index) {
                    Some(name) => { name_stmt.execute(params![chara_index, name]).unwrap(); },
                    None => self.missing_character_names += 1,
                } 

                if let Some(desc) = self.chara_descriptions.get(chara_desc) {
                    desc_stmt.execute(params![chara_desc, desc]).unwrap();
                } 
            }
        }
        
        tx.commit().unwrap();
    }

    pub fn write_character_roma(&mut self, index_batch: &Vec<(i32, i32)>) {
        let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive).unwrap();

        {
            let mut stmt = tx.prepare_cached("
                INSERT INTO character_names_roma (id, name) 
                VALUES (?1, ?2) 
                ON CONFLICT(id) DO NOTHING
            ").unwrap();
            
            for (chara_index, _) in index_batch {
                if let Some(name) = self.chara_roma_names.get(chara_index) {
                    stmt.execute(params![chara_index, name]).unwrap();
                }
            }
        }
        
        tx.commit().unwrap();
    }

    #[allow(dead_code)]
    pub fn write_skill(&mut self, _index_batch: &Vec<i32>) {
        let _sql = "INSERT INTO skill_names (id, name, description) 
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO NOTHING";
    }
    
    pub fn get_missing_names(&self) -> u32 {
        self.missing_character_names
    }

    fn initialize_database(conn: &Connection) {
        conn.execute(
                "CREATE TABLE character_names (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )", 
            ()
        ).unwrap();

        conn.execute(
                "CREATE TABLE character_names_roma (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )", 
            ()
        ).unwrap();

        conn.execute(
                "CREATE TABLE character_descriptions (
                id INTEGER PRIMARY KEY,
                description TEXT NOT NULL
            )", 
            ()
        ).unwrap();

        conn.execute(
                "CREATE TABLE series_names (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )", 
            ()
        ).unwrap();

        conn.execute(
                "CREATE TABLE skill_names (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL
            )", 
            ()
        ).unwrap();
    }

    fn insert_series(conn: &mut Connection, series_table: &ievr_cfg_bin_editor_core::Table) {
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive).unwrap();

        {
            let mut stmt = tx.prepare(
                "INSERT INTO series_names (id, name)
                VALUES (?1, ?2);"
            ).unwrap();

            for row in series_table.rows() {
                let index = parse_int_value(&row.values[0][0]);
                let name = parse_string_value(&row.values[5][0]);

                stmt.execute(params![index, name]).unwrap();
            }
        }

        tx.commit().unwrap();
    }
}

/// SAFETY:
/// This object must not share connections with other object of
/// the same type, otherwise race conditions occurr.
unsafe impl Sync for TextDatabase {}