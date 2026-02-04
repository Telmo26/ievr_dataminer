use std::{collections::HashMap};

use ievr_cfg_bin_editor_core::Database;
use rusqlite::{Connection, params};

use crate::common::{parse_int_value, parse_string_value};

pub struct TextDatabase {
    conn: Connection,
    chara: HashMap<i32, String>,
    chara_roma: HashMap<i32, String>,
    // skill_file: HashMap<i32, (String, String)>,
}

impl TextDatabase {
    pub fn init(conn: Connection, chara_file: Database, chara_roma_file: Database, _skill_file: Database) -> TextDatabase {
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "synchronous", "NORMAL").unwrap();

        // Computing the character hash table
        let chara_table = chara_file.table("NOUN_INFO").unwrap();
        
        let mut chara = HashMap::new();
        for row in chara_table.rows() {
            let index = parse_int_value(&row.values[0][0]);
            let string = parse_string_value(&row.values[5][0]);

            if parse_int_value(&row.values[1][0]) == 0 { // This value is different from 0 when texts are alternatives of the main one
                if chara.insert(index, string).is_some() {
                    println!("Text index {index} in double");
                }
            }
        };

        // Computing the character roma hash table
        let chara_roma_table = chara_roma_file.table("NOUN_INFO").unwrap();

        let mut chara_roma = HashMap::new();
        for row in chara_roma_table.rows() {
            let index = parse_int_value(&row.values[0][0]);
            let string = parse_string_value(&row.values[5][0]);

            if parse_int_value(&row.values[1][0]) == 0 { // This value is different from 0 when texts are alternatives of the main one
                if chara_roma.insert(index, string).is_some() {
                    println!("Text index {index} in double");
                }
            }
        }
        
        TextDatabase { conn, chara, chara_roma }
    }

    pub fn write_character(&mut self, index_batch: &Vec<i32>) {
        let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive).unwrap();

        {
            let mut stmt = tx.prepare_cached("
                INSERT INTO character_names (id, name) 
                VALUES (?1, ?2) 
                ON CONFLICT(id) DO NOTHING
            ").unwrap();
            
            for index in index_batch {
                if let Some(name) = self.chara.get(index) {
                    stmt.execute(params![index, name]).unwrap();
                }
            }
        }
        
        tx.commit().unwrap();
    }

    pub fn write_character_roma(&mut self, index_batch: &Vec<i32>) {
        let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive).unwrap();

        {
            let mut stmt = tx.prepare_cached("
                INSERT INTO character_names_roma (id, name) 
                VALUES (?1, ?2) 
                ON CONFLICT(id) DO NOTHING
            ").unwrap();
            
            for index in index_batch {
                if let Some(name) = self.chara_roma.get(index) {
                    stmt.execute(params![index, name]).unwrap();
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
}

/// SAFETY:
/// This object must not share connections with other object of
/// the same type, otherwise race conditions occurr.
unsafe impl Sync for TextDatabase {}