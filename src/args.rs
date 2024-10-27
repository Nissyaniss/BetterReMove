use std::path::PathBuf;

use clap::{arg, command, Parser};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(author, version, about)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    #[arg(
		short = 't',
		long = "trash-path",
		help = "Reveal the trash path",
		conflicts_with_all = &["force", "new_trash_path", "paths", "generate_completions", "delete_trash_contents", "fzf", "restored_files"]
	)]
    pub trash_path_reveal: bool,

    #[arg(
		short = 'd',
		long = "delete-trash-contents",
		help = "Deletes the trash's contents",
		conflicts_with_all = &["force", "new_trash_path", "paths", "generate_completions", "trash_path_reveal", "fzf", "restored_files"]
	)]
    pub delete_trash_contents: bool,

    #[arg(
		short = 'r',
		long = "restore",
		help = "File(s) to be restored from trash",
		conflicts_with_all = &["force", "new_trash_path", "paths", "generate_completions", "trash_path_reveal", "fzf", "delete_trash_contents"]
	)]
    pub restored_files: Vec<PathBuf>,

    #[arg(
		long = "set-trash-path",
		value_name = "path",
		conflicts_with_all = &["force", "trash_path_reveal", "paths", "generate_completions", "delete_trash_contents", "fzf", "restored_files"],
		help = "Files to remove"
	)]
    pub new_trash_path: Option<PathBuf>,

    #[arg(
		long = "generate-completions",
		value_name = "SHELL",
		conflicts_with_all = &["force", "trash_path_reveal", "paths", "new_trash_path", "delete_trash_contents", "fzf", "restored_files"],
		help = "Generate shell completions"
	)]
    pub generate_completions: Option<Shell>,

    #[arg(
        short = 'f',
        long = "force",
        help = "Force remove file(s) without moving to trash"
    )]
    pub force: bool,

    #[arg(long = "fzf", help = "Display files in fzf", conflicts_with = "paths")]
    pub fzf: bool,

    #[arg(help = "Files to remove")]
    pub paths: Vec<PathBuf>,
}
