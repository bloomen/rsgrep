extern crate clap;
use clap::{App, Arg};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

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
            Arg::with_name("filename")
                .help("The file to search in")
                .required(true),
        )
        .get_matches();

    let string = matches.value_of("string").unwrap();
    let filename = matches.value_of("filename").unwrap();

    let file = File::open(filename).expect("Could not open file for reading");
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        let str = line.unwrap();
        if str.contains(string) {
            println!("{}:{}", i + 1, str);
        }
    }
}
