use std::{fs::File, path::Path};

use ievr_cfg_bin_editor_core::{Database, Value, parse_database};
use memmap2::Mmap;

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