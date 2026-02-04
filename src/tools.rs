use std::{fs::{self, File}, io::{self, BufWriter, Write}, os::unix::fs::PermissionsExt, path::PathBuf, process::{Command, exit}};

use reqwest::blocking::Client;

use crate::settings::Settings;

const TOOL_ROOT: &str = "tools";

const EXTRACTOR_NAME: &str = "ievr_toolbox";
const EXTRACTOR_URL: &str = "https://github.com/Telmo26/ievr_toolbox/releases/latest/download/ievr_toolbox";

pub struct Tools {
    tools_path: PathBuf,
}

impl Tools {
    pub fn new() -> Tools {
        let tools_path = PathBuf::from(TOOL_ROOT);

        if !fs::exists(&tools_path).unwrap() { fs::create_dir(&tools_path).unwrap() };
        
        Tools { 
            tools_path,
        }
    }

    pub fn extract(&self, settings: &Settings, regex_rules: Vec<&str>) -> Result<(), io::Error> {
        if let Some(ref game_folder) = settings.game_folder {
            if fs::exists(game_folder)? {
                let extractor_path = Self::download_latest_extractor(&self.tools_path).unwrap();

                let rules = File::create("rules.txt")?;
                let mut buf_writer = BufWriter::new(rules);
                
                for rule in regex_rules { writeln!(buf_writer, "{}", rule).unwrap(); }
                buf_writer.flush()?;

                let mut args = vec![
                    "-i".to_owned(), 
                    game_folder.clone(), 
                    "-o".to_owned(), 
                    settings.extraction_folder.to_owned(), 
                    "-r".to_owned(), 
                    "rules.txt".to_owned()
                ];

                if let Some(threads) = settings.threads { args.push("-t".to_owned()) ; args.push(threads.to_string()) }
                if let Some(memory) = settings.memory { args.push("-m".to_owned()) ; args.push(memory.to_string()) }

                let output = Command::new(&extractor_path)
                    .args(args)
                    .output()
                    .unwrap();

                if output.status.success() {
                    fs::remove_file("rules.txt")?;
                    return Ok(());
                }                
            }
        }

        Err(io::ErrorKind::NotFound.into())
    }

    fn download_latest_extractor(tools_root: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let extractor_path = tools_root.join(EXTRACTOR_NAME.to_owned() + match std::env::consts::OS {
            "linux" => "-linux64",
            "windows" => "-win64.exe",
            _ => {
                eprintln!("Unsupported OS");
                exit(1)
            }
        });

        if extractor_path.exists() {
            return Ok(extractor_path);
        }

        print!("Missing extractor, downloading... ");

        let url = EXTRACTOR_URL.to_owned() + match std::env::consts::OS {
            "linux" => "-linux64",
            "windows" => "-win64.exe",
            _ => {
                eprintln!("Unsupported OS");
                exit(1)
            }
        };

        let client = Client::builder()
            .user_agent("IEVR Dataminer")
            .build()?;

        let mut response = client.get(url).send()?;

        if response.status().is_success() {
            let mut file = std::fs::File::create(&extractor_path)?;
            response.copy_to(&mut file)?;

            #[cfg(target_os = "linux")]
            {
                let mut perms = file.metadata()?.permissions();
                perms.set_mode(755);
                file.set_permissions(perms).unwrap();
            }
        }

        println!("Extractor download complete.");

        Ok(extractor_path)
    }
}