use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use dialoguer::Confirm;
use std::env;
use std::fs::File;
use std::io;
use std::{
    fs::{self},
    path::PathBuf,
};
use toml::Value;

struct Config {
    path_to_trash: String,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = match env::var("HOME") {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Error getting home directory. Check your configuration file or create the environment variable named $HOME with your home path in it.");
                return Config {
                    path_to_trash: String::from("null"),
                };
            }
        };
        Config {
            path_to_trash: format!(
                "{home_dir}/.local/share/BetterReMove/trash/",
                home_dir = home_dir
            ),
        }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(
        short = 't',
        long = "trash-path",
        help = "Reveal the trash path",
        conflicts_with_all = &["force", "new_trash_path", "paths", "generate_completions", "delete_trash_contents"]
    )]
    trash_path_reveal: bool,

    #[arg(
        short = 'd',
        long = "delete-trash-contents",
        help = "Deletes the trash's contents",
        conflicts_with_all = &["force", "new_trash_path", "paths", "generate_completions", "trash_path_reveal"]
    )]
    delete_trash_contents: bool,

    #[arg(
        long = "set-trash-path",
        value_name = "VALUE",
        conflicts_with_all = &["force", "trash_path_reveal", "paths", "generate_completions", "delete_trash_contents"],
        help = "Files to remove"
    )]
    new_trash_path: Option<PathBuf>,

    #[arg(
        long = "generate-completions",
        value_name = "SHELL",
        conflicts_with_all = &["force", "trash_path_reveal", "paths", "new_trash_path", "delete_trash_contents"],
        help = "Generate shell completions"
    )]
    generate_completions: Option<Shell>,

    #[arg(
        short = 'f',
        long = "force",
        help = "Force remove file(s) without moving to trash"
    )]
    force: bool,

    #[arg(help = "Files to remove")]
    paths: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let files = args.paths.clone();
    let home_dir = match env::var("HOME") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Error getting home directory. Create the environment variable named $HOME with your home path in it.");
            return;
        }
    };
    let config_file = PathBuf::from(format!(
        "{home_dir}/.config/BetterReMove/config.toml",
        home_dir = home_dir
    ));
    let trash_dir = check_config(config_file.clone());
    if trash_dir == "null" {
        return;
    }

    if args.delete_trash_contents {
        let confirmed = Confirm::new()
            .with_prompt("Are you sure you want to erase the trash ?")
            .default(false)
            .interact()
            .unwrap();
        if confirmed {
            for entry in fs::read_dir(trash_dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    match fs::remove_dir_all(path.clone()) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Error removing directory {}: {}", path.display(), e),
                    };
                } else {
                    match fs::remove_file(path.clone()) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Error removing file {}: {}", path.display(), e),
                    };
                }
            }
        }
        return;
    }

    if args.trash_path_reveal {
        println!("This is the current trash directory.\n{}", trash_dir);
    }

    if let Some(shell) = args.generate_completions {
        generate_completions(shell);
        return;
    }

    if args.new_trash_path.clone().is_some() {
        if args.new_trash_path.clone().unwrap().is_file() {
            println!("The new trash path must be a directory.");
            return;
        }
        let new_trash_path = args.new_trash_path.unwrap().to_str().unwrap().to_string();
        let new_config = format!("path_to_trash = '{}'", new_trash_path);
        fs::write(config_file, new_config).unwrap();
        return;
    }

    trashing(files, trash_dir, args);
}

fn check_config(config_file: PathBuf) -> String {
    let default_config = Config::default();
    let default_config_str = format!("path_to_trash = '{}'", default_config.path_to_trash);
    if !config_file.exists() {
        fs::create_dir_all(config_file.parent().unwrap()).unwrap();

        File::create(config_file.clone()).unwrap();
        fs::write(config_file.clone(), default_config_str.clone()).unwrap();
    }

    let config_str = fs::read_to_string(config_file.clone()).unwrap();
    let parsed_config = config_str
        .parse::<Value>()
        .ok()
        .and_then(|r| match r {
            Value::Table(table) => Some(table),
            _ => None,
        })
        .unwrap();
    let mut trash_dir = String::from("null");
    if parsed_config.get("path_to_trash").is_some() {
        trash_dir = parsed_config
            .get("path_to_trash")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        if !PathBuf::from(&trash_dir).exists() {
            match fs::create_dir_all(&trash_dir) {
                Ok(_) => (),
                Err(e) => eprintln!("Error creating trash directory: {}", e),
            };
        } else if PathBuf::from(&trash_dir).is_file() {
            println!("The trash path must be a directory.");
            return "null".to_string();
        } else if !trash_dir.ends_with('/') {
            trash_dir.push('/');
        }
    } else {
        println!("Error parsing config file.");
        let confirmed = Confirm::new()
            .with_prompt("Do you want to reset the config file ?")
            .default(false)
            .interact()
            .unwrap();
        if confirmed {
            fs::write(config_file.clone(), default_config_str).unwrap();
            return "null".to_string();
        }
    }
    trash_dir
}

fn generate_completions<G: Generator>(gen: G) {
    let mut cmd = Args::command();
    let bin_name = env::current_exe()
        .expect("Failed to get binary name")
        .file_name()
        .expect("Failed to get binary name")
        .to_string_lossy()
        .to_string();
    generate(gen, &mut cmd, &bin_name, &mut io::stdout());
}

fn trashing(files: Vec<PathBuf>, trash_dir: String, args: Args) {
    for file in files {
        if file.exists() {
            let path_to_trash = [
                trash_dir.clone(),
                file.file_name().unwrap().to_str().unwrap().to_string(),
            ]
            .join("");
            if PathBuf::from(&path_to_trash).exists() {
                let mut i = 1;
                loop {
                    let path_to_trash = [
                        trash_dir.clone(),
                        file.file_name().unwrap().to_str().unwrap().to_string(),
                        i.to_string(),
                    ]
                    .join("");
                    if !PathBuf::from(&path_to_trash).exists() {
                        fs::rename(&file, path_to_trash).unwrap();
                        break;
                    }
                    i += 1;
                }
            } else if file.is_dir() {
                let confirmed = Confirm::new()
                    .with_prompt(
                        "The file you are trying to trash or remove is a directory. Are you sure ?",
                    )
                    .default(false)
                    .interact()
                    .unwrap();

                if confirmed {
                    if args.force {
                        fs::remove_dir_all(&file).unwrap();
                        continue;
                    }

                    match fs::rename(
                        &file,
                        trash_dir.clone()
                            + "/"
                            + file.file_name().unwrap().to_str().unwrap()
                            + "/",
                    ) {
                        Ok(_) => continue,
                        Err(e) => eprintln!("Error moving directory: {}", e),
                    }
                }
            } else if args.force {
                fs::remove_file(&file).unwrap();
            } else {
                match fs::rename(
                    &file,
                    trash_dir.clone() + file.file_name().unwrap().to_str().unwrap(),
                ) {
                    Ok(_) => continue,
                    Err(e) => eprintln!(
                        "Error moving file: {} {}",
                        e,
                        trash_dir.clone() + file.file_name().unwrap().to_str().unwrap(),
                    ),
                }
            }
        } else {
            println!("File or directory does not exist : {}", file.display());
        }
    }
}
