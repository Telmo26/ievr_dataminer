use std::{fs::File, path::Path};

use ievr_cfg_bin_editor_core::{Database, Value, parse_database};
use memmap2::Mmap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Element {
    WIND = 1,
    FOREST = 2,
    FIRE = 3,
    MOUNTAIN = 4,
    UNKNOWN = 5,
}

impl From<i32> for Element {
    fn from(value: i32) -> Self {
        match value {
            1 => Element::WIND,
            2 => Element::FOREST,
            3 => Element::FIRE,
            4 => Element::MOUNTAIN,
            _ => Element::UNKNOWN,
        }
    }
}

pub fn parse_gamefile(file_path: &Path) -> Option<Database> {
    let file = File::open(file_path).unwrap();

    let mmap = unsafe { Mmap::map(&file).unwrap() };

    parse_database(&mmap).ok()
}

pub fn parse_int_value(value: &Value) -> i32 {
    match value {
        Value::Int(v) => *v,
        _ => unreachable!()
    }
}

pub fn parse_uint_value(value: &Value) -> u32 {
    match value {
        Value::UInt(v) => *v,
        _ => unreachable!()
    }
}

pub fn parse_f32_value(value: &Value) -> f32 {
    match value {
        Value::Float(v) => *v,
        _ => unreachable!()
    }
}

pub fn parse_short_value(value: &Value) -> i16 {
    match value {
        Value::Short(v) => *v,
        _ => unreachable!()
    }
}

pub fn parse_byte_value(value: &Value) -> u8 {
    match value {
        Value::Byte(v) => *v,
        _ => unreachable!()
    }
}

pub fn parse_string_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => panic!("Encountered {:?}", value)
    }
}