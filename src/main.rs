extern crate clap;
use clap::{App, Arg};
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

struct Config {
    pub recursive: bool,
    pub location: bool,
}

fn search_file(config: &Config, string: &str, path: &Path) {
    let filename = path.to_str().unwrap();
    let file = fs::File::open(filename).expect("Could not open file for reading");
    let reader = BufReader::new(file);
    for (i, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        if line.contains(string) {
            if config.location {
                println!("[{}:{}]{}", filename, i + 1, line);
            } else {
                println!("{}", line);
            }
        }
    }
}

fn search_dir(config: &Config, string: &str, dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            search_file(&config, string, &path);
        } else if config.recursive && path.is_dir() {
            search_dir(&config, string, &path);
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
    let path = matches.value_of("path").unwrap();
    let recursive = matches.is_present("recursive");
    let location = matches.is_present("location");

    let config = Config{recursive, location};

    let path = Path::new(path);
    if path.is_file() {
        search_file(&config, string, &path);
    } else if path.is_dir() {
        search_dir(&config, string, &path);
    } else {
        panic!("Could not open path");
    }
}
