# brm (Better ReMove)
`brm` is a command line deletion tool to replace the default and unsafe `rm` (if I delete a file I will never recover it).
The idea was stolen from [rip](https://github.com/nivekuil/rip) (so this README is heavily inspired from them).

Deleted files are send by default to `$HOME/.local/share/BetterReMove/trash` you can change it later (see [Usage](https://github.com/Nissyaniss/BetterReMove#usage))

**`brm` is only available for Linux at the moment. Windows support is planned but no MacOS version is planned at the moment (if you want to add it create a PR i will be happy to merge)**

## THIS IS MY FIRST RUST PROJECT BE INDULGENT I'M NOT SMART

## Installation

Get the binary from [release](https://github.com/Nissyaniss/BetterReMove/releases) and move it to your `/usr/local/bin` (or any folder in your `PATH`)
```bash
$ mv brm /usr/local/bin
```

**I hope i get to upload this onto the arch AUR repo but it is not the case at the moment**

## Usage

```text
Usage: brm [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  Files to remove

Options:
  -t, --trash-path                    Reveal the trash path
  -d, --delete-trash-contents         Deletes the trash's contents
      --set-trash-path <path>        Files to remove
      --generate-completions <SHELL>  Generate shell completions [possible values: bash, elvish, fish, powershell, zsh]
  -f, --force                         Force remove file(s) without moving to trash
  -h, --help                          Print help
```

`-f` is to remove files like rm.

`brm` doesn't care if what you are removing is a directory but will ask if this is wanted (even when `-f` is used), if this is not what you want it to behave like you can create an issue or a PR.

```bash
$ brm test_directory
The file you are trying to trash or remove is a directory. Are you sure ? [y/N]
```

```bash
$ brm -f test_directory
The file you are trying to trash or remove is a directory. Are you sure ? [y/N]
```

If the file name is already in the trash folder `brm` will add a number to it
```bash
$ ls /home/user/.local/BetterReMove/trash/
test
$ brm test
$ ls /home/user/.local/BetterReMove/trash/
test
test1
```

If you want to change the default trash folder you can either change it in the config at `$HOME/.config/BetterReMove/config.toml` or use the `--set-trash-path` like this :

```bash
$ brm --set-trash-path /home/user/new_trash/
$ brm -t
This is the current trash directory.
/home/user/new_trash/
```

You can also generate completions for your favorite shell like this :
```bash
$ brm --generate-completions zsh
# For zsh, you can use bash, elvish, fish and powershell too
```

To empty your trash you can use the `-d` option like this :
```bash
$ brm -d
Are you sure you want to erase the trash ? [y/N]
```

### THERE IS CURRENTLY NO WAY TO RESTORE FILE TO THERE ORIGINAL PATH

I will love to do so but like said above this is my first Rust project and the code is bad as it is, and i cannot see a solution involving a lot of work. But it is planned.

## Planned things to do

- [x] Make the code great again (i think its done)
- [ ] Windows support
- [ ] `fzf` integration
- [ ] Restore files to their original place
- [ ] MacOS support ?

## Credit

- [clap](https://github.com/clap-rs/clap) for all option, completions ect
- [dialoguer](https://github.com/console-rs/dialoguer) for the [Y/N] question
- [toml](https://github.com/toml-rs/toml) for all things toml related
