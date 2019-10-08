extern crate clap;
use clap::{App, Arg};
use std::path::Path;

mod rsgrep;
use rsgrep::*;

fn main() {
    let matches = App::new("rsgrep")
        .version("0.4.0")
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
                .help("Search directories recursively")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("location")
                .short("l")
                .long("location")
                .help("Print filename and line number")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("followlinks")
                .short("f")
                .long("followlinks")
                .help("Follow links")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case insensitive search")
                .takes_value(false),
        )
        .get_matches();

    let mut string = String::from(matches.value_of("string").unwrap());
    let path = matches.value_of("path").unwrap();

    let config = Config {
        recursive: matches.is_present("recursive"),
        location: matches.is_present("location"),
        followlinks: matches.is_present("followlinks"),
        insensitive: matches.is_present("insensitive"),
    };

    let path = Path::new(path);
    if config.insensitive {
        string = string.to_lowercase();
    }
    search(&config, &string, path, true);
}
