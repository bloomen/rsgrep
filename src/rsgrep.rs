use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use regex::Regex;

pub struct Config {
    pub cwd: PathBuf,
    pub recursive: bool,
    pub location: bool,
    pub followlinks: bool,
    pub insensitive: bool,
    pub warnings: bool,
    pub relative: bool,
    pub regex: Option<Regex>,
}

fn path_to_string(config: &Config, path: &Path) -> String {
    if config.relative {
        match path.strip_prefix(&config.cwd).unwrap().to_str() {
            Some(name) => String::from(name),
            None => String::from("<None>"),
        }
    } else {
        match path.to_str() {
            Some(name) => String::from(name),
            None => String::from("<None>"),
        }
    }
}

fn canonicalize(config: &Config, path: &Path) -> Option<PathBuf> {
    match path.canonicalize() {
        Ok(p) => Some(p),
        Err(err) => {
            println!(
                "<rsgrep> Error: Cannot resolve path ({}): {}",
                err,
                path_to_string(config, path)
            );
            None
        }
    }
}

fn resolve_path(config: &Config, path: &Path) -> Option<PathBuf> {
    if config.followlinks {
        canonicalize(config, path)
    } else {
        match fs::read_link(path) {
            Ok(_) => None,
            Err(_) => canonicalize(config, path),
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

fn is_binary(file: &mut fs::File) -> bool {
    let mut buffer: Vec<u8> = vec![];
    if file.take(1024 as u64).read_to_end(&mut buffer).is_ok() {
        let content_type = content_inspector::inspect(&buffer);
        content_type == content_inspector::ContentType::BINARY
    } else {
        false
    }
}

fn matches(config: &Config, string: &str, line: &str) -> bool {
    match &config.regex {
        Some(re) => {
            return re.is_match(line);
        }
        None => {
            if config.insensitive {
                return line.to_lowercase().contains(string);
            } else {
                return line.contains(string);
            }
        }
    }
}

fn search_file(config: &Config, string: &str, path: &Path) {
    if !path.exists() {
        println!(
            "<rsgrep> Error: Path does not exist: {}",
            path_to_string(config, path)
        );
        return;
    }
    let filename = path_to_string(config, path);
    match fs::File::open(path) {
        Ok(mut file) => {
            if is_binary(&mut file) {
                if config.warnings {
                    println!("<rsgrep> Warning: Ignoring binary file: {}", path_to_string(&config, path));
                }
                return
            }
            if let Err(err) = file.seek(io::SeekFrom::Start(0)) {
                println!("<rsgrep> Error: Cannot seek in file ({}): {}", err, filename);
                return
            };
            let reader = io::BufReader::new(file);
            for (i, line) in reader.lines().enumerate() {
                match line {
                    Ok(line) => {
                        if matches(&config, string, &line) {
                            print_line(config, &filename, i + 1, &line);
                        }
                    }
                    Err(err) => {
                        if config.warnings {
                            println!("<rsgrep> Warning: Problem reading from file ({}): {}", err, filename);
                        }
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
            path_to_string(config, dir)
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
                            path_to_string(config, dir)
                        );
                    }
                }
            }
        }
        Err(err) => {
            println!(
                "<rsgrep> Error: Cannot iterate directory ({}): {}",
                err,
                path_to_string(config, dir)
            );
        }
    }
}

pub fn search(config: &Config, string: &str, path: &Path, initial: bool) {
    if !path.exists() {
        println!(
            "<rsgrep> Error: Path does not exist: {}",
            path_to_string(config, path)
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
                    path_to_string(config, &path)
                );
            }
        }
        None => {
            if config.warnings {
                println!("<rsgrep> Warning: Ignoring path: {}", path_to_string(config, path));
            }
        }
    }
}
