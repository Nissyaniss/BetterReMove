use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator};
use dialoguer::Confirm;
use skim::prelude::*;
use std::env;
use std::fmt::Write as Write2;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::process::exit;
use std::{fs, path::PathBuf};
use toml::Value;
use utils::Utils;

mod args;
mod config;
mod utils;

use args::Args;

fn main() {
	let args = Args::parse();
	let mut files = args.paths.clone();
	let utils = Utils::new();

	if utils.trash_dir == PathBuf::from("null") {
		println!("Error getting the config file trash_dir !");
		exit(1);
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
			for entry in fs::read_dir(utils.trash_dir).unwrap() {
				let path = entry.unwrap().path();
				match fs::remove_dir_all(&path) {
					Ok(()) => (),
					Err(e) => {
						if path.is_dir() {
							eprintln!("Error removing directory {}: {}", path.display(), e);
						} else {
							eprintln!("Error removing file {}: {}", path.display(), e);
						}
					}
				}
			}
		}
		return;
	}

	if args.trash_path_reveal {
		println!(
			"This is the current trash directory :\n{}",
			utils.trash_dir.display()
		);
	}

	if let Some(shell) = args.generate_completions {
		generate_completions(shell);
		exit(0);
	}

	if args.new_trash_path.is_some() {
		// todo add check for config file
		if args.new_trash_path.clone().unwrap().is_file() {
			println!("The new trash path must be a directory !");
			exit(1);
		}
		let new_trash_path = args.new_trash_path.unwrap().to_str().unwrap().to_string();
		fs::write(
			utils.config_file,
			format!("path_to_trash = '{new_trash_path}'"),
		)
		.unwrap();
		exit(0);
	}

	if args.fzf {
		let options = SkimOptionsBuilder::default().multi(true).build().unwrap();

		let selected_items =
			Skim::run_with(&options, None).map_or_else(Vec::new, |out| out.selected_items);

		for item in &selected_items {
			files.push(PathBuf::from(item.text().into_owned()));
		}
	}

	if !args.restored_files.is_empty() {
		restore(&args.restored_files, &utils);
		exit(0);
	}

	trashing(files, &utils, &args);
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

fn trashing(files: Vec<PathBuf>, utils: &Utils, args: &Args) {
	for file in files {
		if file.exists() {
			if file.is_dir() {
				let confirmed = Confirm::new()
					.with_prompt(
						"The file you are trying to trash or remove is a directory. Are you sure ?",
					)
					.default(false)
					.interact()
					.unwrap();

				if confirmed {
					move_remove_file(args.force, &file, utils);
				}
			} else {
				move_remove_file(args.force, &file, utils);
			}
		} else {
			println!("File or directory does not exist : {}", file.display());
		}
	}
}

fn move_remove_file(force: bool, file: &PathBuf, utils: &Utils) {
	let absolute_file_path = fs::canonicalize(file).unwrap();

	if !utils.restore_config_file.exists() {
		fs::create_dir_all(utils.restore_config_file.parent().unwrap()).unwrap();
		File::create(&utils.restore_config_file).unwrap();
	}

	if force {
		fs::remove_file(file).unwrap();
		return;
	}
	let path_to_trash = utils
		.trash_dir
		.join(file.file_name().unwrap().to_str().unwrap());
	let mut i = 1;
	if path_to_trash.exists() {
		loop {
			let path_to_trash = utils
				.trash_dir
				.join(file.file_name().unwrap().to_str().unwrap().to_owned() + &i.to_string());
			if !path_to_trash.exists() {
				fs::rename(file, path_to_trash).unwrap();
				break;
			}
			i += 1;
		}
	} else {
		fs::rename(file, path_to_trash).unwrap();
	}
	let mut restore_file = OpenOptions::new()
		.append(true)
		.open(&utils.restore_config_file)
		.unwrap();

	if let Err(e) = writeln!(
		restore_file,
		"{} = '{}'",
		file.file_name().unwrap().to_str().unwrap().to_owned() + &i.to_string(),
		absolute_file_path.display()
	) {
		eprintln!("Couldn't write to file: {e}");
	}
}

fn restore(restore_files: &Vec<PathBuf>, utils: &Utils) {
	for file in restore_files {
		if utils.restore_config_file.exists() {
			let config_str = fs::read_to_string(utils.restore_config_file.clone()).unwrap();
			let mut parsed_config = config_str
				.parse::<Value>()
				.ok()
				.and_then(|r| match r {
					Value::Table(table) => Some(table),
					_ => None,
				})
				.unwrap();
			let file_name = file.file_name().unwrap().to_str().unwrap();
			if parsed_config.get(file_name).is_some() {
				let restore_path = PathBuf::from(
					parsed_config
						.get(file_name)
						.unwrap()
						.as_str()
						.unwrap()
						.to_string(),
				);
				if restore_path.exists() {
					println!(
						"File with the same name at the same place already exists ! {}",
						restore_path.display()
					);
					exit(1);
				}
				parsed_config.remove(file_name);
				fs::rename(utils.trash_dir.join(file.as_path()), restore_path).unwrap();
				let mut new_restore_config_file = String::new();
				for (key, string) in parsed_config {
					match writeln!(
						new_restore_config_file,
						"{key} = '{}'",
						string.as_str().unwrap()
					) {
						Ok(()) => (),
						Err(e) => {
							println!("Cannot write new_restore_config_file : {e}");
						}
					}
				}
				fs::write(utils.restore_config_file.clone(), new_restore_config_file).unwrap();
			} else {
				println!("Restore path not found for {} !", file.display());
			}
		}
	}
}
