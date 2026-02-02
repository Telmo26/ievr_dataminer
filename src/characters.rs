use std::{collections::HashMap, path::{Path, PathBuf}};

use crossbeam::channel::Sender;
use ievr_cfg_bin_editor_core::{Row, Table, Value};

mod character;

pub use character::Character;
use rusqlite::{Connection, Result, params};

pub use crate::{characters::character::{Element, Position, Stats, Style}, common::parse_gamefile};

pub const CHARA_ROOT_PATH: &str = "data/common/gamedata/character/";

pub const CHARA_REQUIRED_FILES: [&str; 3] = [
    "^chara_base_\\d+\\.\\d+\\.\\d+\\.\\d+\\.cfg\\.bin$",
    "^chara_param_\\d+\\.\\d+\\.\\d+\\.\\d+\\.cfg\\.bin$",
    "growth_table_config_\\d+\\.\\d+\\.\\d+\\.\\d+\\.cfg\\.bin$"
];

pub fn populate_character_data(extraction_path: &Path, character_database: Connection, requested_files: Vec<String>, char_name_req_tx: Sender<i32>) {
    initialize_database(&character_database).unwrap();

    let root_path= extraction_path.to_path_buf().join(CHARA_ROOT_PATH);

    let chara_base = parse_gamefile(&PathBuf::from(root_path.join(&requested_files[0]))).unwrap();
    let chara_base_info = chara_base.table("CHARA_BASE_INFO").unwrap();

    let chara_param = parse_gamefile(&PathBuf::from(root_path.join(&requested_files[1]))).unwrap();
    let chara_param_info = chara_param.table("CHARA_PARAM_INFO").unwrap();

    let growth_table = parse_gamefile(&PathBuf::from(root_path.join(&requested_files[2]))).unwrap();
    let growth_table_main = growth_table.table("m_growthTableMainList").unwrap();

    let growth_hash_table = parse_growth_table(growth_table_main);    

    let chara_base_info = get_characters(&chara_base_info);

    for row in chara_base_info {
        let index = parse_int_value(&row.values[21][0]);

        let name_id = parse_int_value(&row.values[3][0]);

        char_name_req_tx.send(name_id).unwrap();

        let series_id = parse_int_value(&row.values[15][0]);

        let chara_base_id = parse_int_value(&row.values[0][0]);

        for row in chara_param_info.rows() {
            if parse_int_value(&row.values[1][0]) == chara_base_id {
                let skill_slice: Vec<i32> = row.values[23..=28].iter()// We filter by making sure the character has a second technique path
                    .flatten()
                    .map(parse_int_value)
                    .collect();

                if skill_slice.iter().any(|v| *v == 0) { continue }

                let element = Element::from(parse_int_value(&row.values[2][0]));
                let main_position = Position::from(parse_int_value(&row.values[3][0]));
                let alt_position = Position::from(parse_int_value(&row.values[4][0]));
                let style = Style::from(parse_int_value(&row.values[5][0]));

                let growth_pattern = parse_int_value(&row.values[7][0]) as u8;

                let chara_rank = parse_int_value(&row.values[9][0]) as u8;

                let (lvl50_stats, lvl99_stats) = if main_position != Position::UNKNOWN {
                    match growth_hash_table.get(&(main_position as u8, growth_pattern, chara_rank)) {
                        Some(v) => *v,
                        None => panic!("No value found for {:?}", (main_position as u8, growth_pattern, chara_rank))
                    }
                } else {
                    (Stats::default(), Stats::default())
                };

                let rarity = parse_int_value(&row.values[41][0]);

                let character = Character {
                    index,
                    name_id,
                    element,
                    main_position,
                    alt_position,
                    style,
                    lvl50_stats,
                    lvl99_stats,
                    series_id,
                };

                let result = match rarity {
                    0 => insert_character(&character_database, &character, "characters"),
                    5..8 => insert_character(&character_database, &character, "heroes"),
                    8 => insert_character(&character_database, &character, "basaras"),
                    _ => unreachable!()
                };

                if result.is_err() {
                    eprintln!("Duplicate for player ID {}", index)
                }
            }
        }
    };
}

fn get_characters(chara_base_info: &Table) -> Vec<&Row> {
    let mut filtered_table: Vec<&Row> = chara_base_info.rows().iter()
        .filter(|row| match row.values[21][0] {
            Value::Int(v) => v > 0, // We only want the characters who have a valid index
            _ => unreachable!()
        })
        .collect();

    filtered_table.sort_by_key(|row| {
        match row.values[21][0] {
            Value::Int(v) => v, // We also sort by index while we're at it
            _ => unreachable!()
        }
    });

    filtered_table
}

fn parse_int_value(value: &Value) -> i32 {
    match value {
        Value::Int(v) => *v,
        _ => unreachable!()
    }
}

fn parse_byte_value(value: &Value) -> u8 {
    match value {
        Value::Byte(v) => *v,
        _ => unreachable!()
    }
}

fn parse_growth_table(growth_table_main: &Table) -> HashMap<(u8, u8, u8), (Stats, Stats)> {
    let mut growth_hash_table = HashMap::new();

    for row in growth_table_main.rows() {
        let main_position = parse_byte_value(&row.values[0][0]);
        let growth_pattern = parse_byte_value(&row.values[1][0]);
        let chara_rank = parse_byte_value(&row.values[2][0]);

        growth_hash_table.insert((main_position, growth_pattern, chara_rank), 
            (Stats {
                kick:           parse_int_value(&row.values[3][0]) as u16,
                control:        parse_int_value(&row.values[4][0]) as u16,
                technique:      parse_int_value(&row.values[5][0]) as u16,
                pressure:       parse_int_value(&row.values[6][0]) as u16,
                physical:       parse_int_value(&row.values[7][0]) as u16,
                agility:        parse_int_value(&row.values[8][0]) as u16,
                intelligence:   parse_int_value(&row.values[9][0]) as u16,
            }, 
            Stats {
                kick:           parse_int_value(&row.values[10][0]) as u16,
                control:        parse_int_value(&row.values[11][0]) as u16,
                technique:      parse_int_value(&row.values[12][0]) as u16,
                pressure:       parse_int_value(&row.values[13][0]) as u16,
                physical:       parse_int_value(&row.values[14][0]) as u16,
                agility:        parse_int_value(&row.values[15][0]) as u16,
                intelligence:   parse_int_value(&row.values[16][0]) as u16,
            })
        );
    };

    growth_hash_table
}

fn initialize_database(database: &Connection) -> Result<()> {
    database.execute(
        "CREATE TABLE IF NOT EXISTS characters (
            index_id        INTEGER PRIMARY KEY,
            name_id         INTEGER NOT NULL,
            element         INTEGER NOT NULL,
            main_position   INTEGER NOT NULL,
            alt_position    INTEGER NOT NULL,
            style           INTEGER NOT NULL,
            series_id       INTEGER NOT NULL,

            lvl50_kick          INTEGER,
            lvl50_control       INTEGER,
            lvl50_technique     INTEGER,
            lvl50_pressure      INTEGER,
            lvl50_physical      INTEGER,
            lvl50_agility       INTEGER,
            lvl50_intelligence  INTEGER,

            lvl99_kick          INTEGER,
            lvl99_control       INTEGER,
            lvl99_technique     INTEGER,
            lvl99_pressure      INTEGER,
            lvl99_physical      INTEGER,
            lvl99_agility       INTEGER,
            lvl99_intelligence  INTEGER
        );", 
    ()
    )?;

    database.execute(
        "CREATE TABLE IF NOT EXISTS heroes (
            index_id        INTEGER NOT NULL,
            name_id         INTEGER NOT NULL,
            element         INTEGER NOT NULL,
            main_position   INTEGER NOT NULL,
            alt_position    INTEGER NOT NULL,
            style           INTEGER NOT NULL,
            series_id       INTEGER NOT NULL,

            lvl50_kick          INTEGER,
            lvl50_control       INTEGER,
            lvl50_technique     INTEGER,
            lvl50_pressure      INTEGER,
            lvl50_physical      INTEGER,
            lvl50_agility       INTEGER,
            lvl50_intelligence  INTEGER,

            lvl99_kick          INTEGER,
            lvl99_control       INTEGER,
            lvl99_technique     INTEGER,
            lvl99_pressure      INTEGER,
            lvl99_physical      INTEGER,
            lvl99_agility       INTEGER,
            lvl99_intelligence  INTEGER
        );", 
    ()
    )?;

    database.execute(
        "CREATE TABLE IF NOT EXISTS basaras (
            index_id        INTEGER NOT NULL,
            name_id         INTEGER NOT NULL,
            element         INTEGER NOT NULL,
            main_position   INTEGER NOT NULL,
            alt_position    INTEGER NOT NULL,
            style           INTEGER NOT NULL,
            series_id       INTEGER NOT NULL,

            lvl50_kick          INTEGER,
            lvl50_control       INTEGER,
            lvl50_technique     INTEGER,
            lvl50_pressure      INTEGER,
            lvl50_physical      INTEGER,
            lvl50_agility       INTEGER,
            lvl50_intelligence  INTEGER,

            lvl99_kick          INTEGER,
            lvl99_control       INTEGER,
            lvl99_technique     INTEGER,
            lvl99_pressure      INTEGER,
            lvl99_physical      INTEGER,
            lvl99_agility       INTEGER,
            lvl99_intelligence  INTEGER
        );", 
    ()
    )?;

    Ok(())
}

fn insert_character(conn: &Connection, c: &Character, database_name: &str) -> rusqlite::Result<()> {
    let sql = format!("
        INSERT INTO {database_name} (
            index_id, name_id, element, main_position, alt_position, style, series_id,
            lvl50_kick, lvl50_control, lvl50_technique, lvl50_pressure,
            lvl50_physical, lvl50_agility, lvl50_intelligence,
            lvl99_kick, lvl99_control, lvl99_technique, lvl99_pressure,
            lvl99_physical, lvl99_agility, lvl99_intelligence
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,
                    ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                    ?15, ?16, ?17, ?18, ?19, ?20, ?21)"
    );

    conn.execute(
        &sql,
        params![
            c.index,
            c.name_id,
            c.element as i32,
            c.main_position as i32,
            c.alt_position as i32,
            c.style as i32,
            c.series_id,

            c.lvl50_stats.kick,
            c.lvl50_stats.control,
            c.lvl50_stats.technique,
            c.lvl50_stats.pressure,
            c.lvl50_stats.physical,
            c.lvl50_stats.agility,
            c.lvl50_stats.intelligence,

            c.lvl99_stats.kick,
            c.lvl99_stats.control,
            c.lvl99_stats.technique,
            c.lvl99_stats.pressure,
            c.lvl99_stats.physical,
            c.lvl99_stats.agility,
            c.lvl99_stats.intelligence,
        ],
    )?;
    Ok(())
}
