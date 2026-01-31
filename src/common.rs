use std::{fs::File, path::Path};

use ievr_cfg_bin_editor_core::{Database, parse_database};
use memmap2::Mmap;

pub fn parse_gamefile(file_path: &Path) -> Option<Database> {
    let file = File::open(file_path).unwrap();

    let mmap = unsafe { Mmap::map(&file).unwrap() };

    parse_database(&mmap).ok()
}