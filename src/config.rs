use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
    process::exit,
};

use dialoguer::Confirm;
use toml::Value;

pub struct Config {
    path_to_trash: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = match env::var("HOME") {
            Ok(home_dir) => home_dir,
            Err(e) => {
                eprintln!("Error getting home directory. Create the environment variable named $HOME with your home path in it. : {e}");
                exit(1);
            }
        };
        let home_dir = PathBuf::from(home_dir);
        Self {
            path_to_trash: home_dir.join(".local/share/BetterReMove/trash/"),
        }
    }
}

fn check(config_file: &Path, default_config_str: String) {
    if !config_file.exists() {
        fs::create_dir_all(config_file.parent().unwrap()).unwrap();
        File::create(config_file).unwrap();
        fs::write(config_file, default_config_str.clone()).unwrap();
    }
}

pub fn get_trash_dir(config_file: &Path) -> PathBuf {
    let default_config = Config::default();
    let default_config_str = format!(
        "path_to_trash = '{}'",
        default_config
            .path_to_trash
            .to_str()
            .map_or_else(|| panic!("Error getting default trash path"), |s| s)
    );
    check(config_file, default_config_str.clone());
    let config_str = fs::read_to_string(config_file).unwrap();
    let parsed_config = config_str
        .parse::<Value>()
        .ok()
        .and_then(|r| match r {
            Value::Table(table) => Some(table),
            _ => None,
        })
        .unwrap();
    let mut trash_dir = PathBuf::from("null");
    if parsed_config.get("path_to_trash").is_some() {
        trash_dir = PathBuf::from(
            parsed_config
                .get("path_to_trash")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        );
        if !PathBuf::from(&trash_dir).exists() {
            match fs::create_dir_all(&trash_dir) {
                Ok(()) => (),
                Err(e) => eprintln!("Error creating trash directory: {e}"),
            };
        } else if PathBuf::from(&trash_dir).is_file() {
            println!("The trash path must be a directory.");
            return PathBuf::from("null");
        }
    } else {
        println!("Error parsing config file.");
        let confirmed = Confirm::new()
            .with_prompt("Do you want to reset the config file ?")
            .default(false)
            .interact()
            .unwrap();
        if confirmed {
            fs::write(config_file, default_config_str).unwrap();
            return PathBuf::from("null");
        }
    }
    trash_dir
}
