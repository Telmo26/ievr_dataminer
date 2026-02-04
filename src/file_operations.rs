use std::{fs::{self, File}, path::Path};

use crate::{
    DATABASES,

    CHARA_ROOT_PATH,
    CHARA_REQUIRED_FILES,

    TEXT_ROOT_PATH,
    TEXT_LANGUAGES,
    TEXT_REQUIRED_FILES,
};

pub fn create_required_files(output_folder: &Path) {
    if let Ok(true) = fs::exists(output_folder) {
        fs::remove_dir_all(output_folder).unwrap();
    }

    fs::create_dir_all(output_folder).unwrap();

    for database_path in DATABASES {
        let mut clone = output_folder.to_path_buf();
        clone.push(database_path);

        let parent = clone.parent().unwrap();
        fs::create_dir_all(parent).unwrap();
        
        File::create(&clone).unwrap();
    }
}

pub fn check_chara_files_existence(extraction_root_path: &Path) -> Option<Vec<(&'static str, String)>> {
    let chara_root = extraction_root_path.to_path_buf().join(CHARA_ROOT_PATH);

    match fs::exists(&chara_root) {
        Ok(bool) => if !bool { return Some(Vec::new()) },
        Err(_) => return None,
    }

    let mut files = Vec::with_capacity(CHARA_REQUIRED_FILES.len());

    for rule in CHARA_REQUIRED_FILES {
        let regex = regex::Regex::new(rule).unwrap();

        for file in fs::read_dir(&chara_root).ok()? {
            let file = file.ok()?;
            let file_name = file.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            if regex.is_match(&file_name_str) {
                files.push((rule, file_name_str.into_owned()));
            }
        }
    }

    Some(files)
}

pub fn check_text_files_existence(extraction_root_path: &Path) -> Option<Vec<(&'static str, String)>> {
    let text_root = extraction_root_path.to_path_buf().join(TEXT_ROOT_PATH);

    match fs::exists(&text_root) {
        Ok(bool) => if !bool { return Some(Vec::new()) },
        Err(_) => return None,
    }

    let mut files = Vec::with_capacity(TEXT_LANGUAGES.len() * TEXT_REQUIRED_FILES.len());

    for rule in TEXT_REQUIRED_FILES {
        let regex = regex::Regex::new(rule).unwrap();

        for language in TEXT_LANGUAGES {
            let language_path = text_root.join(language);

            let dir = fs::read_dir(&language_path).ok();
            
            if dir.is_none() { // We're missing a language, extract everything
                return Some(Vec::new())
            }

            for file in dir? { // We now know it is some
                let file = file.ok()?;
                let file_name = file.file_name();
                let file_name_str = file_name.to_string_lossy();
                
                if regex.is_match(&file_name_str) {
                    files.push((rule, language.to_owned() + "/" + &file_name_str));
                }
            }
        }  
    }

    Some(files)
}