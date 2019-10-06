extern crate clap;
use clap::{App, Arg};
use std::fs;
use std::io::prelude::*;
use std::io;
use std::path::Path;

struct Config {
    pub recursive: bool,
    pub location: bool,
}

fn path_to_string(path: &Path) -> String {
    match path.to_str() {
        Some(name) => {
            String::from(name)
        }
        None => {
            String::from("<None>")
        }
    }
}

fn search_file(config: &Config, string: &str, path: &Path) {
    let filename = path_to_string(path);
    match fs::File::open(path) {
        Ok(file) => {
            let reader = io::BufReader::new(file);
            for (i, line) in reader.lines().enumerate() {
                match line {
                    Ok(line) => {
                        if line.contains(string) {
                            if config.location {
                                println!("[{}:{}]{}", filename, i + 1, line);
                            } else {
                                println!("{}", line);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: Reading from file ({}): {}", err, filename);
                        break;
                    }
                }
            }
        }
        Err(err) => {
            println!("Error: Cannot open file ({}): {}", err, filename);
        }
    }
}

fn search_dir(config: &Config, string: &str, dir: &Path) {
    match fs::read_dir(dir) {
        Ok(mut entries) => {
            while let Some(entry) = entries.next() {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() {
                            search_file(&config, string, &path);
                        } else if config.recursive && path.is_dir() {
                            search_dir(&config, string, &path);
                        }
                    }
                    Err(err) => {
                        println!("Error: Iterating directory ({}): {}", err, path_to_string(dir));
                    }
                }
            }
        }
        Err(err) => {
            println!("Error: Cannot iterate directory ({}): {}", err, path_to_string(dir));
        }
    }
}

fn main() {
    let matches = App::new("rsgrep")
        .version("0.2.0")
        .about("A simple version of grep implemented in Rust")
        .author("Christian Blume")
        .arg(
            Arg::with_name("string")
                .help("The string to search for")
                .required(true),
        )
        .arg(
            Arg::with_name("path")
                .help("The path to search in (file or directory)")
                .required(true),
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .help("Whether to search directories recursively")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("location")
                .short("l")
                .long("location")
                .help("Whether to print filename and linenumber")
                .takes_value(false),
        )
        .get_matches();

    let string = matches.value_of("string").unwrap();
    let inpath = matches.value_of("path").unwrap();
    let recursive = matches.is_present("recursive");
    let location = matches.is_present("location");

    let config = Config{recursive, location};

    let path = Path::new(inpath);
    if path.is_file() {
        search_file(&config, string, &path);
    } else if path.is_dir() {
        search_dir(&config, string, &path);
    } else {
        println!("Error: Cannot open path: {}", inpath);
    }
}
