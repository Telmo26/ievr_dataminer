use std::{collections::{HashMap, HashSet}, path::Path, sync::LazyLock};

use crossbeam::channel::Sender;
use rusqlite::{Connection, params};

use crate::{common::parse_gamefile, skills::hissatsu::Category};

mod hissatsu;
mod aura;
use hissatsu::Hissatsu;
use aura::Aura;

pub const SKILL_ROOT_PATH: &str = "data/common/gamedata/skill/";

pub static SKILL_REQUIRED_FILES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("skill_config", "^skill_config_\\d+\\.\\d+\\.\\d+\\.\\d+\\.cfg\\.bin$");
    map.insert("aura_skill_config", "^aura_skill_config_\\d+\\.\\d+\\.\\d+\\.\\d+\\.cfg\\.bin$");
    map
});

pub fn populate_skill_data(extraction_path: &Path, mut skill_database_connection: Connection, requested_files: HashMap<&'static str, String>, skill_name_req_tx: Sender<i32>) {
    initialize_database(&mut skill_database_connection).unwrap();

    let root_path = extraction_path.to_path_buf().join(SKILL_ROOT_PATH);

    let skill_config = parse_gamefile(&root_path.join(&requested_files["skill_config"])).unwrap();
    let skill_info_list = skill_config.table("m_skillInfoList").unwrap();

    let mut hissatsus = Vec::new();
    let mut aura_hissatsus = Vec::new();
    let mut passives = Vec::new();

    let mut skill_names = HashSet::new();
    for row in skill_info_list.rows() {
        let skill = Hissatsu::from(row);

        if skill_names.insert(skill.name_id) {
            skill_name_req_tx.send(skill.name_id).unwrap()
        }

        match skill.category {
            Category::Passive | Category::StatsIncrease | Category::Unknown => passives.push(skill),
            Category::Shoot | Category::Dribble | Category::Defense | Category::Goalkeeper if skill.growth_rate == 0 => aura_hissatsus.push(skill),
            _ => hissatsus.push(skill),
        }
    }
    // We empty the buffer
    insert_hissatsu(&mut skill_database_connection, &hissatsus).unwrap();
    insert_aura_hissatsu(&mut skill_database_connection, &aura_hissatsus).unwrap();
    insert_passives(&mut skill_database_connection, &passives).unwrap();

    let aura_skill_config = parse_gamefile(&root_path.join(&requested_files["aura_skill_config"])).unwrap();
    let aura_cmd_info_list = aura_skill_config.table("AURA_CMD_INFO_LIST").unwrap();

    let mut aura_buffer = Vec::with_capacity(aura_cmd_info_list.rows().len() / 4);
    for row in aura_cmd_info_list.rows() {
        if row.values.len() == 19 {
            let aura = Aura::from(row);

            if skill_names.insert(aura.name_id) {
                skill_name_req_tx.send(aura.name_id).unwrap()
            }

            aura_buffer.push(aura);
        }
    }

    insert_aura(&mut skill_database_connection, &aura_buffer).unwrap();
}

fn initialize_database(database: &mut Connection) -> rusqlite::Result<()> {
    database.pragma_update(None, "journal_mode", "WAL").unwrap();
    database.pragma_update(None, "synchronous", "NORMAL").unwrap();

    database.execute(
        "CREATE TABLE IF NOT EXISTS hissatsu (
            skill_id        INTEGER PRIMARY KEY,
            name_id         INTEGER NOT NULL,
            power           INTEGER NOT NULL,
            element         INTEGER NOT NULL,
            category        INTEGER NOT NULL,
            growth_rate     INTEGER NOT NULL,
            is_block        BOOLEAN NOT NULL,
            is_longshot     BOOLEAN NOT NULL,
            tp_consumption  INTEGER NOT NULL,
            cooldown        INTEGER NOT NULL
        );", 
    ()
    )?;

    database.execute(
        "CREATE TABLE IF NOT EXISTS aura_hissatsu (
            skill_id        INTEGER PRIMARY KEY,
            name_id         INTEGER NOT NULL,
            power           INTEGER NOT NULL,
            element         INTEGER NOT NULL,
            category        INTEGER NOT NULL,
            growth_rate     INTEGER NOT NULL,
            is_block        BOOLEAN NOT NULL,
            is_longshot     BOOLEAN NOT NULL,
            tp_consumption  INTEGER NOT NULL,
            cooldown        INTEGER NOT NULL
        );", 
    ()
    )?;

    database.execute(
        "CREATE TABLE IF NOT EXISTS passives (
            skill_id        INTEGER PRIMARY KEY,
            name_id         INTEGER NOT NULL,
            element         INTEGER NOT NULL,
            category        INTEGER NOT NULL,
            cooldown        INTEGER NOT NULL
        );", 
    ()
    )?;

    database.execute(
        "CREATE TABLE IF NOT EXISTS aura (
            aura_id         INTEGER PRIMARY KEY,
            name_id         INTEGER NOT NULL,
            skill_id        INTEGER,
            element         INTEGER NOT NULL
        );", 
    ()
    )?;

    Ok(())
}

fn insert_hissatsu(conn: &mut Connection, hissatsus: &Vec<Hissatsu>) -> rusqlite::Result<()> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive)?;

    {
        let mut stmt = tx.prepare_cached(
        "INSERT INTO hissatsu (
        skill_id,
        name_id,
        power,
        element,
        category,
        growth_rate,
        is_block,
        is_longshot,
        tp_consumption,
        cooldown
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(skill_id) DO NOTHING"
        )?;

        for h in hissatsus {
            stmt.execute(
                params![
                    h.skill_id,
                    h.name_id,
                    h.power,
                    h.element as i32,
                    h.category as u8,
                    h.growth_rate,
                    h.is_block,
                    h.is_longshot,
                    h.tp_consumption,
                    h.cooldown
                ],
            )?;
        }
    }
    
    tx.commit()
}

fn insert_aura_hissatsu(conn: &mut Connection, aura_hissatsus: &Vec<Hissatsu>) -> rusqlite::Result<()> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive)?;

    {
        let mut stmt = tx.prepare_cached(
        "INSERT INTO aura_hissatsu (
        skill_id,
        name_id,
        power,
        element,
        category,
        growth_rate,
        is_block,
        is_longshot,
        tp_consumption,
        cooldown
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(skill_id) DO NOTHING"
        )?;

        for h in aura_hissatsus {
            stmt.execute(
                params![
                    h.skill_id,
                    h.name_id,
                    h.power,
                    h.element as i32,
                    h.category as u8,
                    h.growth_rate,
                    h.is_block,
                    h.is_longshot,
                    h.tp_consumption,
                    h.cooldown
                ],
            )?;
        }
    }
    
    tx.commit()
}

fn insert_passives(conn: &mut Connection, passives: &Vec<Hissatsu>) -> rusqlite::Result<()> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive)?;

    {
        let mut stmt = tx.prepare_cached(
        "INSERT INTO passives (
        skill_id,
        name_id,
        element,
        category,
        cooldown
        ) VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(skill_id) DO NOTHING"
        )?;

        for p in passives {
            stmt.execute(
                params![
                    p.skill_id,
                    p.name_id,
                    p.element as i32,
                    p.category as u8,
                    p.cooldown
                ],
            )?;
        }
    }
    
    tx.commit()
}

fn insert_aura(conn: &mut Connection, auras: &Vec<Aura>) -> rusqlite::Result<()> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive)?;

    {
        let mut stmt = tx.prepare_cached(
        "INSERT INTO aura (
        aura_id,
        name_id,
        skill_id,
        element
        ) VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(aura_id) DO NOTHING"
        )?;

        for a in auras {
            stmt.execute(
                params![
                    a.aura_id,
                    a.name_id,
                    a.skill_id,
                    a.element as u8
                ],
            )?;
        }
    }
    
    tx.commit()
}