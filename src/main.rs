extern crate clap;
use clap::{App, Arg};
use std::path::Path;

mod rsgrep;
use rsgrep::*;

fn main() {
    let matches = App::new("rsgrep")
        .version("0.3.0")
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
                .help("Whether to print filename and line number")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("followlinks")
                .short("f")
                .long("followlinks")
                .help("Whether to follow links")
                .takes_value(false),
        )
        .get_matches();

    let string = matches.value_of("string").unwrap();
    let path = matches.value_of("path").unwrap();

    let config = Config {
        recursive: matches.is_present("recursive"),
        location: matches.is_present("location"),
        followlinks: matches.is_present("followlinks"),
    };

    let path = Path::new(path);
    search(&config, string, path);
}
