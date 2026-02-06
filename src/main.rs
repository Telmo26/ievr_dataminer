use crossbeam::channel::{self, Receiver, Sender};
use rusqlite::Connection;

mod characters;
mod text;
mod common;
mod file_operations;
mod settings;
mod tools;

use std::{collections::HashMap, fs, io::Write, path::{Path, PathBuf}, process::exit, sync::Arc, thread::{self, JoinHandle}};

use file_operations::{
    create_required_files,
    check_chara_files_existence,
    check_text_files_existence,
};

use characters::{
    populate_character_data,
    CHARA_ROOT_PATH,
    CHARA_REQUIRED_FILES,
};

use text::{
    populate_text_data,
    TEXT_ROOT_PATH,
    TEXT_REQUIRED_FILES,
};

use settings::Settings;

use tools::Tools;

const DATABASES: [&str; 2] = [
    "characters.sqlite",
    "skills.sqlite",
];

const TEXT_DATABASES_ROOT: &str = "text";

fn main() {
    // If the settings file does not exist, we download it
    if !fs::exists("settings.toml").unwrap() {
        println!("Settings file not found, downloading from GitHub...");
        match Settings::download() {
            Ok(_) => println!("Download completed, please update the contents of the settings file and restart the program."),
            Err(e) => eprintln!("Download failed because of \n{e}\nAborting"),
        }

        pause();
        exit(1)
    }

    // We parse the settings
    let settings = match Settings::parse("settings.toml") {
        Ok(s) => s,
        Err(e) => { eprintln!("Aborting because of {e}") ; exit(1) }
    };

    // We compute the paths
    let output_folder_path = PathBuf::from(&settings.output_folder);
    let extraction_root_path = Arc::new(PathBuf::from(&settings.extraction_folder));

    // We create the database files
    create_required_files(&output_folder_path);

    // We setup the required channels for communication between the threads
    let (char_name_req_tx, char_name_req_rx) = channel::unbounded();

    // We verify the presence of all required files
    let mut rules_to_extract = get_missing_character_rules(&extraction_root_path);    
    rules_to_extract.extend(get_missing_text_rules(&extraction_root_path));

    #[cfg(debug_assertions)]
    println!("Rules not fullfilled: {:#?}", rules_to_extract);

    // We initialize the required tools
    let tools = Tools::new();

    // We extract missing files
    if !rules_to_extract.is_empty() {
        println!("Missing game files, starting game extraction...\n");
        match tools.extract(&settings, rules_to_extract) {
            Ok(_) => println!("\nGame extraction done.\n"),
            Err(e) => {
                eprintln!("Impossible to extract game files due to: {e}\nAborting...");
                exit(1);
            }
        };
    }

    println!("Starting game data mining...");

    // We start the different threads
    let character_thread = create_character_thread(&output_folder_path, &extraction_root_path, char_name_req_tx);
    let text_thread = create_text_thread(&output_folder_path, &extraction_root_path, char_name_req_rx);

    // We wait for the program to finish
    let _ = character_thread.join();
    let _ = text_thread.join();

    println!("\nGame data mining done. Please open the \"{}\" folder to get the databases.", &settings.output_folder);
}

fn create_character_thread(output_folder_path: &Path, extraction_root_path: &Arc<PathBuf>, char_name_req_tx: Sender<(i32, i32)>) -> JoinHandle<()> {
    let character_database = Connection::open(output_folder_path.join(DATABASES[0])).unwrap();

    let chara_requested_files = check_chara_files_existence(&extraction_root_path).unwrap();

    let extraction_path_clone = extraction_root_path.clone();
    thread::spawn(move || {
        populate_character_data(&extraction_path_clone, character_database, chara_requested_files, char_name_req_tx);
    })
}

fn create_text_thread(output_folder_path: &Path, extraction_root_path: &Arc<PathBuf>, char_name_req_rx: Receiver<(i32, i32)>) -> JoinHandle<()> {
    let text_requested_files = check_text_files_existence(&extraction_root_path).unwrap();

    let mut text_databases = HashMap::with_capacity(text_requested_files.len());
    for language in text_requested_files.keys() {
        text_databases.insert(
            *language, 
            Connection::open(output_folder_path.join(TEXT_DATABASES_ROOT).join(format!("{language}.sqlite"))).unwrap()
        );
    }

    let extraction_path_clone = extraction_root_path.clone();
    thread::spawn(move || {
        populate_text_data(&extraction_path_clone,text_databases, text_requested_files, char_name_req_rx);
    })
}

fn get_missing_character_rules(extraction_root_path: &Arc<PathBuf>) -> Vec<&'static str> {
    match check_chara_files_existence(extraction_root_path) {
        Some(files) => {
            let mut missing_rules = Vec::new();

            if files.len() < CHARA_REQUIRED_FILES.len() { // We compute the rules not fullfilled
                for (identifier, rule) in CHARA_REQUIRED_FILES.iter() {
                    if !files.contains_key(identifier) {
                        missing_rules.push(*rule);
                    }
                }
            }

            missing_rules
        }
        None => {
            eprintln!("Error while reading the extracted game files.");
            exit(1);
        }
    }
}

fn get_missing_text_rules(extraction_root_path: &Arc<PathBuf>) -> Vec<&'static str> {
    match check_text_files_existence(&extraction_root_path) {
        Some(files) => {
            let mut missing_rules = Vec::new();

            for (language, rules) in TEXT_REQUIRED_FILES.iter() {
                if !files.contains_key(language) { // If there's a whole language missing
                    for &value in rules.values() { missing_rules.push(value); }
                    break;
                }

                if files[language].len() < rules.len() {
                    for (&identifier, &rule) in rules.iter() {
                        if !files[language].contains_key(identifier) {
                            missing_rules.push(rule);
                        }
                    }
                }
            }
            
            missing_rules
        }
        None => {
            eprintln!("Error while reading the extracted game files.");
            exit(1);
        }
    }
}

fn pause() {
    let mut stdout = std::io::stdout();
    print!("Press Enter to continue...");
    
    // We must manually flush stdout because print! doesn't 
    // add a newline, and the prompt might not show up immediately.
    stdout.flush().unwrap();

    let mut _unused = String::new();
    std::io::stdin().read_line(&mut _unused).expect("Failed to read line");
}