use clap::Parser;
use dialoguer::Confirm;
use std::{
    fs::{self},
    path::PathBuf,
};
#[allow(dead_code)]
struct Config {
    path_to_trash: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            path_to_trash: String::from("/mnt/c/Users/test/BetterReMove/trash/"),
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
        conflicts_with_all = &["force", "new_trash_path", "paths"]
    )]
    trash_path_reveal: bool,

    #[arg(
        long = "set-trash-path",
        value_name = "VALUE",
        conflicts_with_all = &["force", "trash_path_reveal", "paths"],
        help = "Files to remove"
    )]
    new_trash_path: Option<PathBuf>,

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
    let config_file = PathBuf::from(r"/mnt/c/Users/test/BetterReMove/config.toml"); // VALUE FOR DEVELOPMENT
    if !config_file.exists() {
        fs::write(
            config_file.clone(),
            "path_to_trash = '/mnt/c/Users/test/BetterReMove/trash'", // VALUE FOR DEVELOPMENT
        )
        .unwrap();
    }
    let config_str = fs::read_to_string(config_file.clone()).unwrap();
    let parsed_config: toml::Value = toml::from_str(&config_str).unwrap();
    let trash_dir = parsed_config["path_to_trash"].as_str().unwrap().to_string();

    if !PathBuf::from(&trash_dir).exists() {
        fs::create_dir(&trash_dir).unwrap();
    }

    if args.trash_path_reveal {
        println!("This is the current trash directory.\n{}", trash_dir);
    }

    if args.new_trash_path.clone() != None {
        let new_trash_path = args.new_trash_path.unwrap().to_str().unwrap().to_string();
        let new_config = format!("path_to_trash = '{}'", new_trash_path);
        fs::write(config_file, new_config).unwrap();
        return;
    }

    trashing(files, trash_dir, args);
}

fn trashing(files: Vec<PathBuf>, trash_dir: String, args: Args) {
    for file in files {
        if file.exists() {
            let path_to_trash = vec![
                trash_dir.clone(),
                file.file_name().unwrap().to_str().unwrap().to_string(),
            ]
            .join("");
            if PathBuf::from(&path_to_trash).exists() {
                let mut i = 1;
                loop {
                    let path_to_trash = vec![
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
                if args.force {
                    fs::remove_dir_all(&file).unwrap();
                    continue;
                }

                let confirmed = Confirm::new()
                    .with_prompt("The file you are trying to is a directory.\nAre you sure ?")
                    .default(false)
                    .interact()
                    .unwrap();

                if confirmed {
                    match fs::rename(
                        &file,
                        String::from(
                            trash_dir.clone() + "/" + &file.file_name().unwrap().to_str().unwrap() + "/",
                        ),
                    ) {
                        Ok(_) => continue,
                        Err(e) => eprintln!("Error moving directory: {}", e),
                    }
                }
            } else {
                if args.force {
                    fs::remove_file(&file).unwrap();
                } else {
                    match fs::rename(
                        &file,
                        String::from(
                            trash_dir.clone() + "/" + &file.file_name().unwrap().to_str().unwrap(),
                        ),
                    ) {
                        Ok(_) => continue,
                        Err(e) => eprintln!(
                            "Error moving file: {} {}",
                            e,
                            String::from(
                                trash_dir.clone()
                                    + &file.file_name().unwrap().to_str().unwrap()
                                    + "/",
                            ),
                        ),
                    }
                }
            }
        } else {
            println!("File or directory does not exist : {}", file.display());
        }
    }
}
