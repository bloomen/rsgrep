use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub struct Config {
    pub recursive: bool,
    pub location: bool,
    pub followlinks: bool,
    pub insensitive: bool,
    pub warnings: bool,
}

fn path_to_string(path: &Path) -> String {
    match path.to_str() {
        Some(name) => String::from(name),
        None => String::from("<None>"),
    }
}

fn canonicalize(path: &Path) -> Option<PathBuf> {
    match path.canonicalize() {
        Ok(p) => Some(p),
        Err(err) => {
            println!(
                "<rsgrep> Error: Cannot resolve path ({}): {}",
                err,
                path_to_string(path)
            );
            None
        }
    }
}

fn resolve_path(config: &Config, path: &Path) -> Option<PathBuf> {
    if config.followlinks {
        canonicalize(path)
    } else {
        match fs::read_link(path) {
            Ok(_) => None,
            Err(_) => canonicalize(path),
        }
    }
}

fn print_line(config: &Config, filename: &str, i: usize, line: &str) {
    if config.location {
        println!("[{}:{}]{}", filename, i + 1, line);
    } else {
        println!("{}", line);
    }
}

fn search_file(config: &Config, string: &str, path: &Path) {
    if !path.exists() {
        println!(
            "<rsgrep> Error: Path does not exist: {}",
            path_to_string(path)
        );
        return;
    }
    let filename = path_to_string(path);
    match fs::File::open(path) {
        Ok(file) => {
            let reader = io::BufReader::new(file);
            for (i, line) in reader.lines().enumerate() {
                match line {
                    Ok(line) => {
                        if config.insensitive {
                            if line.to_lowercase().contains(string) {
                                print_line(config, &filename, i + 1, &line);
                            }
                        } else if line.contains(string) {
                            print_line(config, &filename, i + 1, &line);
                        }
                    }
                    Err(err) => {
                        println!("<rsgrep> Error: Reading from file ({}): {}", err, filename);
                        break;
                    }
                }
            }
        }
        Err(err) => {
            println!("<rsgrep> Error: Cannot open file ({}): {}", err, filename);
        }
    }
}

fn search_dir(config: &Config, string: &str, dir: &Path) {
    if !dir.exists() {
        println!(
            "<rsgrep> Error: Path does not exist: {}",
            path_to_string(dir)
        );
        return;
    }
    match fs::read_dir(dir) {
        Ok(mut entries) => {
            while let Some(entry) = entries.next() {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        search(&config, string, &path, false);
                    }
                    Err(err) => {
                        println!(
                            "<rsgrep> Error: Iterating directory ({}): {}",
                            err,
                            path_to_string(dir)
                        );
                    }
                }
            }
        }
        Err(err) => {
            println!(
                "<rsgrep> Error: Cannot iterate directory ({}): {}",
                err,
                path_to_string(dir)
            );
        }
    }
}

pub fn search(config: &Config, string: &str, path: &Path, initial: bool) {
    if !path.exists() {
        println!(
            "<rsgrep> Error: Path does not exist: {}",
            path_to_string(path)
        );
        return;
    }
    match resolve_path(&config, path) {
        Some(path) => {
            if path.is_file() {
                search_file(&config, string, &path);
            } else if path.is_dir() {
                if initial || config.recursive {
                    search_dir(&config, string, &path);
                }
            } else {
                println!(
                    "<rsgrep> Error: Cannot open path: {}",
                    path_to_string(&path)
                );
            }
        }
        None => {
            if config.warnings {
                println!("<rsgrep> Warning: Ignoring path: {}", path_to_string(path));
            }
        }
    }
}
