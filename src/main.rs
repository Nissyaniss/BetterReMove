use clap::Parser;
use std::{fs::{self}, path::PathBuf};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short('f'), exclusive(true))]
    force: bool,

    // #[arg(short('t'))]
    // trash_path_reveal: bool,

    #[arg(required(true))]
    paths: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let files = args.paths;
    let trash_dir  = String::from(r"/tmp/BetterReMove/");

    if !PathBuf::from(&trash_dir).exists() {
        fs::create_dir(&trash_dir).unwrap();
    }

    // if args.trash_path_reveal {
    //     println!("{}", trash_dir);
    // }

    for file in files {
        if file.exists() {
            if args.force {
                fs::remove_file(&file).unwrap();
            } else {
                let path_to_trash = vec![trash_dir.clone(), file.file_name().unwrap().to_str().unwrap().to_string()].join("");
                if !PathBuf::from(&path_to_trash).exists() {
                    fs::copy(&file, path_to_trash).unwrap();
                    fs::remove_file(&file).unwrap();
                } else {
                    let mut i = 1;
                    loop {
                        let path_to_trash = vec![trash_dir.clone(), file.file_name().unwrap().to_str().unwrap().to_string(), i.to_string()].join("");
                        if !PathBuf::from(&path_to_trash).exists() {
                            fs::copy(&file, path_to_trash).unwrap();
                            fs::remove_file(&file).unwrap();
                            break;
                        }
                        i += 1;
                    }
                }
            }
        } else {
            println!("File does not exist");
        }
    }
}