use std::{collections::HashMap, fs::{self, File}, path::Path};

use crate::{
    CHARA_REQUIRED_FILES, CHARA_ROOT_PATH, DATABASES, TEXT_DATABASES_ROOT, TEXT_REQUIRED_FILES, TEXT_ROOT_PATH, text::TEXT_LANGUAGES
};

pub fn create_required_files(output_folder: &Path) {
    if let Ok(true) = fs::exists(output_folder) {
        fs::remove_dir_all(output_folder).unwrap();
    }

    fs::create_dir_all(output_folder).unwrap();

    for database_path in DATABASES {
        let path = output_folder.to_path_buf().join(database_path);

        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).unwrap();
        
        File::create(&path).unwrap();
    }

    for language in TEXT_LANGUAGES {
        let path = output_folder.to_path_buf().join(TEXT_DATABASES_ROOT).join(format!("{language}.sqlite"));

        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).unwrap();
        
        File::create(&path).unwrap();
    }
}

pub fn check_chara_files_existence(extraction_root_path: &Path) -> Option<HashMap<&'static str, String>> {
    let chara_root = extraction_root_path.to_path_buf().join(CHARA_ROOT_PATH);

    match fs::exists(&chara_root) {
        Ok(bool) => if !bool { return Some(HashMap::new()) },
        Err(_) => return None,
    }

    let mut files: HashMap<&str, String> = HashMap::new();

    for (identifier, rule) in CHARA_REQUIRED_FILES.iter() {
        let regex = regex::Regex::new(rule).unwrap();

        for file in fs::read_dir(&chara_root).ok()? {
            let file = file.ok()?;
            let file_name = file.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            if regex.is_match(&file_name_str) {
                files.insert(identifier, file_name_str.to_string());
            }
        }
    }

    Some(files)
}

pub fn check_text_files_existence(extraction_root_path: &Path) -> Option<HashMap<&'static str, HashMap<&'static str, String>>> {
    let text_root = extraction_root_path.to_path_buf().join(TEXT_ROOT_PATH);

    match fs::exists(&text_root) {
        Ok(bool) => if !bool { return Some(HashMap::new()) },
        Err(_) => return None,
    }

    let mut files: HashMap<&str, HashMap<&str, String>> = HashMap::new();

    for (language, rules) in TEXT_REQUIRED_FILES.iter() {
        files.insert(
            *language, 
            rules.clone()
                .into_iter()
                .map(|(k, v)| (k, v.to_owned()))
                .collect()
        );

        let language_path = text_root.join(language);

        let dir = fs::read_dir(&language_path).ok();

        if dir.is_none() {
            return Some(HashMap::new()); // We're missing a language, extract everything
        };

        for file in dir? { // We now know it is some
            for (identifier, rule) in rules.iter() {
                let regex = regex::Regex::new(rule).unwrap();

                let file = file.as_ref().ok()?;
                let file_name = file.file_name();
                let file_name_str = file_name.to_string_lossy();
                
                if regex.is_match(&file_name_str) {
                    let value = files
                        .get_mut(language)?
                        .get_mut(identifier)?;

                    *value = file_name_str.to_string();
                }
            }
        } 
    }

    Some(files)
}