use std::{path::PathBuf, process::exit};

use homedir::my_home;

use crate::config::get_trash_dir;

pub struct Utils {
    pub config_file: PathBuf,
    pub trash_dir: PathBuf,
    pub restore_config_file: PathBuf,
}

impl Utils {
    pub fn new() -> Self {
        let home_dir = match my_home() {
            Ok(home_dir) => home_dir.unwrap(),
            Err(e) => {
                eprintln!("Error getting home directory. Create the environment variable named $HOME with your home path in it. : {e}");
                exit(1)
            }
        };
        let config_file = home_dir.join(".config/BetterReMove/config.toml");
        let restore_config_file = home_dir.join(".config/BetterReMove/original_path.toml");
        Self {
            config_file: config_file.clone(),
            trash_dir: get_trash_dir(&config_file),
            restore_config_file,
        }
    }
}
