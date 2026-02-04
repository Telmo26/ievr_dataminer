use std::{error::Error, fs::File, io::Read};

use reqwest::blocking::Client;

pub struct Settings {
    pub output_folder: String,
    pub extraction_folder: String,
 
    pub game_folder: Option<String>,   
    pub threads: Option<i64>,
    pub memory: Option<f64>
}

impl Settings {
    pub fn parse(settings_path: &str) -> Result<Self, SettingsError> {
        let mut settings_file = File::open(settings_path).map_err(|e| SettingsError::IOError(e))?;

        let mut settings = String::new();
        settings_file.read_to_string(&mut settings).map_err(|e| SettingsError::IOError(e))?;

        println!("{settings}");

        settings = settings.replace(r"\", r"\\"); // This solves escaping issues on Windows

        let settings_table = match settings.parse::<toml::Table>() {
            Ok(table) => table,
            Err(_) => return Err(SettingsError::TomlParseError)
        };

        let output_folder = match &settings_table["datamining"]["output_folder"] {
            toml::Value::String(s) if !s.is_empty() => s.clone(),
            value => return Err(SettingsError::OutputFolderError(value.clone()))
        };

        let extraction_folder = match &settings_table["datamining"]["extraction_folder"] {
            toml::Value::String(s) if !s.is_empty() => s.clone(),
            value => return Err(SettingsError::ExtractionFolderError(value.clone()))
        };

        let game_folder = match &settings_table["extraction"]["game_folder"] {
            toml::Value::String(s) if !s.is_empty() => Some(s.clone()),
            _ => None,
        };

        let threads = match &settings_table["extraction"]["threads"] {
            toml::Value::String(s) if s == "auto" => None,
            toml::Value::Integer(i) => Some(*i),
            _ => {
                eprintln!("Incorrect value for the threads parameter, treating is as \"auto\"");
                None
            }
        };

        let memory = match &settings_table["extraction"]["memory"] {
            toml::Value::String(s) if s == "auto" => None,
            toml::Value::Float(i) => Some(*i),
            _ => {
                eprintln!("Incorrect value for the memory parameter, treating is as \"auto\"");
                None
            }
        };

        Ok(Settings {
            output_folder,
            game_folder,
            extraction_folder,

            threads,
            memory
        })
    }

    pub fn download() -> Result<(), Box<dyn std::error::Error>> {
        // 1. Define the URL and the destination path
        let url = "https://raw.githubusercontent.com/Telmo26/ievr_dataminer/refs/heads/main/settings.toml";
        let target_path = "settings.toml";

        // 2. Create a client (GitHub likes a User-Agent header)
        let client = Client::builder()
            .user_agent("IEVR Dataminer")
            .build()?;

        // 3. Make the request
        let mut response = client.get(url).send()?;

        if response.status().is_success() {
            // 4. Create the local file and stream the content into it
            let mut dest = File::create(target_path)?;
            std::io::copy(&mut response, &mut dest)?;
            Ok(())
        } else {
            Err(Box::new(SettingsError::DownloadError(response.status().as_u16())))
        }

        
    }
}

#[derive(Debug)]
pub enum SettingsError {
    IOError(std::io::Error),
    TomlParseError,
    OutputFolderError(toml::Value),
    ExtractionFolderError(toml::Value),
    DownloadError(u16),
}

impl Error for SettingsError {}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(e) => write!(f, "Filesystem error: {e}"),
            Self::TomlParseError => write!(f, "Invalid TOML format"),
            Self::OutputFolderError(path) => write!(f, "Incorrect output folder: {path}"),
            Self::ExtractionFolderError(path) => write!(f, "Incorrect extraction folder: {path}"),
            Self::DownloadError(code) => write!(f, "Download error, response code: {code}")
        }
    }
} 