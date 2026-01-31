use std::{path::{PathBuf}};

use ievr_cfg_bin_editor_core::{Row, Table, Value};

mod character;

use character::Character;

use crate::{characters::character::{Element, Position, Style}, common::parse_gamefile};

const CHARA_BASE_PATH: &str = "chara_base_1.03.98.00.cfg.bin";
const CHARA_PARAMS_PATH: &str = "chara_param_1.03.66.00.cfg.bin";

pub fn populate_character_data() {
    let chara_base = parse_gamefile(&PathBuf::from(CHARA_BASE_PATH)).unwrap();
    let chara_base_info = chara_base.table("CHARA_BASE_INFO").unwrap();

    let chara_param = parse_gamefile(&PathBuf::from(CHARA_PARAMS_PATH)).unwrap();
    let chara_param_info = chara_param.table("CHARA_PARAM_INFO").unwrap();

    let chara_base_info = get_characters(&chara_base_info);

    let mut characters = Vec::with_capacity(chara_base_info.len());

    for row in chara_base_info {
        let mut index = parse_int_value(&row.values[21][0]);
        if index >= 3646 { index += 1 } // See README for explanation

        let name_id = parse_int_value(&row.values[3][0]);
        let series_id = parse_int_value(&row.values[15][0]);

        let chara_base_id = parse_int_value(&row.values[0][0]);

        let mut element = Element::UNKNOWN;
        let mut main_position = Position::UNKNOWN;
        let mut alt_position = Position::UNKNOWN;
        let mut style = Style::UNKNOWN;

        for row in chara_param_info.rows() {
            if parse_int_value(&row.values[1][0]) == chara_base_id {
                element = Element::from(parse_int_value(&row.values[2][0]));
                main_position = Position::from(parse_int_value(&row.values[3][0]));
                alt_position = Position::from(parse_int_value(&row.values[4][0]));
                style = Style::from(parse_int_value(&row.values[5][0]));
            }
        }

        characters.push(Character {
            index,
            name_id,
            element,
            main_position,
            alt_position,
            style,
            series_id,
        });
    }

    characters = characters.into_iter().filter(|character| {
        character.style == Style::TENSION && character.main_position == Position::FW
    }).collect();
    
    for character in characters {
        println!("{:?}", character);
    }
}

fn get_characters(chara_base_info: &Table) -> Vec<&Row> {
    let mut filtered_table: Vec<&Row> = chara_base_info.rows().iter()
        .filter(|row| match row.values[21][0] {
            Value::Int(v) => v > 0 && v <= 5854,
            _ => unreachable!()
        })
        .collect();

    filtered_table.sort_by_key(|row| {
        match row.values[21][0] {
            Value::Int(v) => v,
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