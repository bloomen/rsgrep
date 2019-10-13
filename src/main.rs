use clap::{App, Arg};
use regex::Regex;
use std::env;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use termcolor as tc;

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
        .arg(
            Arg::with_name("warnings")
                .short("w")
                .long("warnings")
                .help("Show warnings")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("relative")
                .short("t")
                .long("relative")
                .help("Print relative filenames")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("regex")
                .short("e")
                .long("regex")
                .help("Interpret the search string as a regular expression")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("color")
                .short("c")
                .long("color")
                .help("Output colored strings")
                .takes_value(false),
        )
        .get_matches();

    let mut string = String::from(matches.value_of("string").unwrap());
    let path = matches.value_of("path").unwrap();
    let color = matches.is_present("color");

    let color_choice = if color {
        tc::ColorChoice::Always
    } else {
        tc::ColorChoice::Never
    };

    let mut output = Output {
        color,
        loc_color: tc::ColorSpec::new().set_fg(Some(tc::Color::Cyan)).clone(),
        warn_color: tc::ColorSpec::new().set_fg(Some(tc::Color::Yellow)).clone(),
        err_color: tc::ColorSpec::new().set_fg(Some(tc::Color::Red)).clone(),
        stdout: tc::StandardStream::stdout(color_choice),
        stderr: tc::StandardStream::stderr(color_choice),
    };

    let mut current_dir = PathBuf::new();
    match env::current_dir() {
        Ok(p) => {
            current_dir.push(p);
        }
        Err(err) => {
            let mut err_writer = ErrWriter::new(&mut output);
            writeln!(
                &mut err_writer.stream(),
                "<rsgrep> Error: Unable to determine current working directory ({})",
                err
            )
            .unwrap();
        }
    }

    let mut regex = None;
    if matches.is_present("regex") {
        match Regex::new(&string) {
            Ok(re) => {
                regex = Some(re);
            }
            Err(err) => {
                let mut err_writer = ErrWriter::new(&mut output);
                writeln!(
                    &mut err_writer.stream(),
                    "<rsgrep> Error: Unable to parse regular expression ({}): {}",
                    err,
                    string
                )
                .unwrap();
                return;
            }
        }
    }

    let config = Config {
        cwd: current_dir,
        recursive: matches.is_present("recursive"),
        location: matches.is_present("location"),
        followlinks: matches.is_present("followlinks"),
        insensitive: matches.is_present("insensitive") && regex.is_none(),
        warnings: matches.is_present("warnings"),
        relative: matches.is_present("relative"),
        regex,
    };

    let path = Path::new(path);
    if config.insensitive {
        string = string.to_lowercase();
    }

    search(&config, &mut output, &string, path, true);
}
