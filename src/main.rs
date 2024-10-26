use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator};
use dialoguer::Confirm;
use skim::prelude::*;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;
use std::{fs, path::PathBuf};

mod args;
mod config;

use args::Args;
use config::check;

fn main() {
    let args = Args::parse();
    let mut files = args.paths.clone();
    let home_dir = match env::var("HOME") {
        Ok(home_dir) => home_dir,
        Err(e) => {
            eprintln!("Error getting home directory. Create the environment variable named $HOME with your home path in it. : {e}");
            exit(1);
        }
    };

    let home_dir = PathBuf::from(home_dir);
    let config_file = home_dir.join(".config/BetterReMove/config.toml");
    let trash_dir = check(&config_file);
    if trash_dir == PathBuf::from("null") {
        return;
    }

    if std::env::args().len() == 1 {
        Args::command().print_help().unwrap();
        println!();
        std::process::exit(0);
    }

    if args.delete_trash_contents {
        let confirmed = Confirm::new()
            .with_prompt("Are you sure you want to delete all your trash ?")
            .default(false)
            .interact()
            .unwrap();
        if confirmed {
            for entry in fs::read_dir(trash_dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    match fs::remove_dir_all(&path) {
                        Ok(()) => (),
                        Err(e) => eprintln!("Error removing directory {}: {}", path.display(), e),
                    };
                } else {
                    match fs::remove_file(&path) {
                        Ok(()) => (),
                        Err(e) => eprintln!("Error removing file {}: {}", path.display(), e),
                    };
                }
            }
        }
        return;
    }

    if args.trash_path_reveal {
        println!(
            "This is the current trash directory.\n{}",
            trash_dir.display()
        );
    }

    if let Some(shell) = args.generate_completions {
        generate_completions(shell);
        return;
    }

    if args.new_trash_path.is_some() {
        if args.new_trash_path.clone().unwrap().is_file() {
            println!("The new trash path must be a directory.");
            return;
        }
        let new_trash_path = args.new_trash_path.unwrap().to_str().unwrap().to_string();
        let new_config = format!("path_to_trash = '{new_trash_path}'");
        fs::write(config_file, new_config).unwrap();
        return;
    }

    if args.fzf {
        let options = SkimOptionsBuilder::default().multi(true).build().unwrap();

        let selected_items =
            Skim::run_with(&options, None).map_or_else(Vec::new, |out| out.selected_items);

        for item in &selected_items {
            files.push(PathBuf::from(item.text().into_owned()));
        }
    }

    trashing(files, &trash_dir, &args);
}

fn generate_completions<G: Generator>(gen: G) {
    let mut cmd = Args::command();
    let Ok(bin_name) = env::current_exe() else {
        eprintln!("Cannot get binary name");
        exit(1);
    };
    let bin_name = bin_name.file_name().unwrap().to_string_lossy().to_string();
    generate(gen, &mut cmd, &bin_name, &mut io::stdout());
}

fn trashing(files: Vec<PathBuf>, trash_dir: &Path, args: &Args) {
    let home_dir = match env::var("HOME") {
        Ok(home_dir) => home_dir,
        Err(e) => {
            eprintln!("Error getting home directory. Create the environment variable named $HOME with your home path in it. : {e}");
            exit(1);
        }
    };
    let home_dir = PathBuf::from(home_dir);
    let restore_config_file = home_dir.join(".config/BetterReMove/original_path.toml");
    if !restore_config_file.exists() {
        fs::create_dir_all(restore_config_file.parent().unwrap()).unwrap();
        File::create(&restore_config_file).unwrap();
    }

    for file in files {
        if file.exists() {
            let path_to_trash = trash_dir.join(file.file_name().unwrap().to_str().unwrap());
            let absolute_file_path = fs::canonicalize(&file).unwrap();
            if path_to_trash.exists() {
                let mut i = 1;
                loop {
                    let path_to_trash = trash_dir.join(
                        file.file_name().unwrap().to_str().unwrap().to_owned() + &i.to_string(),
                    );
                    if !path_to_trash.exists() {
                        fs::rename(&file, path_to_trash).unwrap();
                        break;
                    }
                    i += 1;
                }
                let mut restore_file = OpenOptions::new()
                    .append(true)
                    .open(&restore_config_file)
                    .unwrap();

                if let Err(e) = writeln!(
                    restore_file,
                    "{} : {}",
                    absolute_file_path.display(),
                    file.file_name().unwrap().to_str().unwrap().to_owned() + &i.to_string()
                ) {
                    eprintln!("Couldn't write to file: {e}");
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
                        trash_dir.join(file.file_name().unwrap().to_str().unwrap()),
                    ) {
                        Ok(()) => continue,
                        Err(e) => eprintln!("Error moving directory: {e}"),
                    }
                }
            } else if args.force {
                fs::remove_file(&file).unwrap();
            } else {
                match fs::rename(
                    &file,
                    trash_dir.join(file.file_name().unwrap().to_str().unwrap()),
                ) {
                    Ok(()) => continue,
                    Err(e) => eprintln!(
                        "Error moving file: {} {}",
                        e,
                        trash_dir
                            .join(file.file_name().unwrap().to_str().unwrap())
                            .display()
                    ),
                }
            }
        } else {
            println!("File or directory does not exist : {}", file.display());
        }
    }
}

fn restore() {}
