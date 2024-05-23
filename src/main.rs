use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use dialoguer::Confirm;
use skim::prelude::*;
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process::exit;
use std::{fs, path::PathBuf};
use toml::Value;

struct Config {
	path_to_trash: PathBuf,
}

impl Default for Config {
	fn default() -> Self {
		let home_dir: String;
		if !cfg!(windows) {
			home_dir = match env::var("HOME") {
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
		} else if cfg!(windows) {
			home_dir = match env::var("UserProfile") {
				Ok(home_dir) => home_dir,
				Err(e) => {
					eprintln!("Error getting home directory. Create the environment variable named UserProfile with your home path in it. : {e}");
					exit(1);
				}
			};
			let home_dir = PathBuf::from(home_dir);
			Self {
				path_to_trash: home_dir.join(r".BetterReMove\trash\"),
			}
		} else {
			return Self {
				path_to_trash: PathBuf::from("null"),
			};
		}
	}
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
#[allow(clippy::struct_excessive_bools)]
struct Args {
	#[arg(
		short = 't',
		long = "trash-path",
		help = "Reveal the trash path",
		conflicts_with_all = &["force", "new_trash_path", "paths", "generate_completions", "delete_trash_contents", "fzf"]
	)]
	trash_path_reveal: bool,

	#[arg(
		short = 'd',
		long = "delete-trash-contents",
		help = "Deletes the trash's contents",
		conflicts_with_all = &["force", "new_trash_path", "paths", "generate_completions", "trash_path_reveal", "fzf"]
	)]
	delete_trash_contents: bool,

	#[arg(
		long = "set-trash-path",
		value_name = "path",
		conflicts_with_all = &["force", "trash_path_reveal", "paths", "generate_completions", "delete_trash_contents", "fzf"],
		help = "Files to remove"
	)]
	new_trash_path: Option<PathBuf>,

	#[arg(
		long = "generate-completions",
		value_name = "SHELL",
		conflicts_with_all = &["force", "trash_path_reveal", "paths", "new_trash_path", "delete_trash_contents", "fzf"],
		help = "Generate shell completions"
	)]
	generate_completions: Option<Shell>,

	#[arg(
		short = 'f',
		long = "force",
		help = "Force remove file(s) without moving to trash"
	)]
	force: bool,

	#[arg(long = "fzf", help = "Display files in fzf", conflicts_with = "paths")]
	fzf: bool,

	#[arg(help = "Files to remove")]
	paths: Vec<PathBuf>,
}

fn main() {
	let args = Args::parse();
	let mut files = args.paths.clone();
	let home_dir: String;
	if !cfg!(windows) {
		home_dir = match env::var("HOME") {
			Ok(home_dir) => home_dir,
			Err(e) => {
				eprintln!("Error getting home directory. Create the environment variable named $HOME with your home path in it. : {e}");
				exit(1);
			}
		};
	} else if cfg!(windows) {
		home_dir = match env::var("UserProfile") {
			Ok(home_dir) => home_dir,
			Err(e) => {
				eprintln!("Error getting home directory. Create the environment variable named UserProfile with your home path in it. : {e}");
				exit(1);
			}
		};
	} else {
		eprintln!("Critical error getting home directory.");
		exit(1);
	}

	let home_dir = PathBuf::from(home_dir);
	let config_file = home_dir.join(".config/BetterReMove/config.toml");
	let trash_dir = check_config(&config_file);
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
					match fs::remove_dir_all(path.clone()) {
						Ok(()) => (),
						Err(e) => eprintln!("Error removing directory {}: {}", path.display(), e),
					};
				} else {
					match fs::remove_file(path.clone()) {
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

fn check_config(config_file: &Path) -> PathBuf {
	let default_config = Config::default();
	let default_config_str = format!(
		"path_to_trash = '{}'",
		default_config
			.path_to_trash
			.to_str()
			.map_or_else(|| panic!("Error getting default trash path"), |s| s)
	);
	if !config_file.exists() {
		fs::create_dir_all(config_file.parent().unwrap()).unwrap();

		File::create(config_file).unwrap();
		fs::write(config_file, default_config_str.clone()).unwrap();
	}

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
	for file in files {
		if file.exists() {
			let path_to_trash = trash_dir.join(file.file_name().unwrap().to_str().unwrap());
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
