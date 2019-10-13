use regex::Regex;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use tc::WriteColor;
use termcolor as tc;

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

pub struct Output {
    pub color: bool,
    pub loc_color: tc::ColorSpec,
    pub warn_color: tc::ColorSpec,
    pub err_color: tc::ColorSpec,
    pub stdout: tc::StandardStream,
    pub stderr: tc::StandardStream,
}

pub struct LocWriter<'a> {
    output: &'a mut Output,
}

impl<'a> LocWriter<'a> {
    pub fn new(output: &mut Output) -> LocWriter {
        if output.color {
            output.stdout.set_color(&output.loc_color).unwrap();
        }
        LocWriter { output }
    }
    pub fn stream(&mut self) -> &mut tc::StandardStream {
        &mut self.output.stdout
    }
}

impl<'a> Drop for LocWriter<'a> {
    fn drop(&mut self) {
        if self.output.color {
            self.output.stdout.reset().unwrap();
        }
    }
}

pub struct ErrWriter<'a> {
    output: &'a mut Output,
}

impl<'a> ErrWriter<'a> {
    pub fn new(output: &mut Output) -> ErrWriter {
        if output.color {
            output.stderr.set_color(&output.err_color).unwrap();
        }
        ErrWriter { output }
    }
    pub fn stream(&mut self) -> &mut tc::StandardStream {
        &mut self.output.stderr
    }
}

impl<'a> Drop for ErrWriter<'a> {
    fn drop(&mut self) {
        if self.output.color {
            self.output.stderr.reset().unwrap();
        }
    }
}

pub struct WarnWriter<'a> {
    output: &'a mut Output,
}

impl<'a> WarnWriter<'a> {
    pub fn new(output: &mut Output) -> WarnWriter {
        if output.color {
            output.stderr.set_color(&output.warn_color).unwrap();
        }
        WarnWriter { output }
    }
    pub fn stream(&mut self) -> &mut tc::StandardStream {
        &mut self.output.stderr
    }
}

impl<'a> Drop for WarnWriter<'a> {
    fn drop(&mut self) {
        if self.output.color {
            self.output.stderr.reset().unwrap();
        }
    }
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

fn canonicalize(config: &Config, mut output: &mut Output, path: &Path) -> Option<PathBuf> {
    match path.canonicalize() {
        Ok(p) => Some(p),
        Err(err) => {
            let mut err_writer = ErrWriter::new(&mut output);
            writeln!(
                &mut err_writer.stream(),
                "<rsgrep> Error: Cannot resolve path ({}): {}",
                err,
                path_to_string(config, path)
            )
            .unwrap();
            None
        }
    }
}

fn resolve_path(config: &Config, mut output: &mut Output, path: &Path) -> Option<PathBuf> {
    if config.followlinks {
        canonicalize(config, &mut output, path)
    } else {
        match fs::read_link(path) {
            Ok(_) => None,
            Err(_) => canonicalize(config, &mut output, path),
        }
    }
}

fn print_line(config: &Config, mut output: &mut Output, filename: &str, i: usize, line: &str) {
    if config.location {
        let mut loc_writer = LocWriter::new(&mut output);
        write!(
            &mut loc_writer.stream(),
            "{}:{}:",
            filename, i + 1
        ).unwrap();
    }
    println!("{}", line);
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
        Some(re) => re.is_match(line),
        None => {
            if config.insensitive {
                line.to_lowercase().contains(string)
            } else {
                line.contains(string)
            }
        }
    }
}

fn search_file(config: &Config, mut output: &mut Output, string: &str, path: &Path) {
    if !path.exists() {
        let mut err_writer = ErrWriter::new(&mut output);
        writeln!(
            &mut err_writer.stream(),
            "<rsgrep> Error: Path does not exist: {}",
            path_to_string(config, path)
        )
        .unwrap();
        return;
    }
    let filename = path_to_string(config, path);
    match fs::File::open(path) {
        Ok(mut file) => {
            if is_binary(&mut file) {
                if config.warnings {
                    let mut warn_writer = WarnWriter::new(&mut output);
                    writeln!(
                        &mut warn_writer.stream(),
                        "<rsgrep> Warning: Ignoring binary file: {}",
                        path_to_string(&config, path)
                    )
                    .unwrap();
                }
                return;
            }
            if let Err(err) = file.seek(io::SeekFrom::Start(0)) {
                let mut err_writer = ErrWriter::new(&mut output);
                writeln!(
                    &mut err_writer.stream(),
                    "<rsgrep> Error: Cannot seek in file ({}): {}",
                    err,
                    filename
                )
                .unwrap();
                return;
            };
            let reader = io::BufReader::new(file);
            for (i, line) in reader.lines().enumerate() {
                match line {
                    Ok(line) => {
                        if matches(&config, string, &line) {
                            print_line(config, &mut output, &filename, i + 1, &line);
                        }
                    }
                    Err(err) => {
                        if config.warnings {
                            let mut warn_writer = WarnWriter::new(&mut output);
                            writeln!(
                                &mut warn_writer.stream(),
                                "<rsgrep> Warning: Problem reading from file ({}): {}",
                                err,
                                filename
                            )
                            .unwrap();
                        }
                        break;
                    }
                }
            }
        }
        Err(err) => {
            let mut err_writer = ErrWriter::new(&mut output);
            writeln!(
                &mut err_writer.stream(),
                "<rsgrep> Error: Cannot open file ({}): {}",
                err,
                filename
            )
            .unwrap();
        }
    }
}

fn search_dir(config: &Config, mut output: &mut Output, string: &str, dir: &Path) {
    if !dir.exists() {
        let mut err_writer = ErrWriter::new(&mut output);
        writeln!(
            &mut err_writer.stream(),
            "<rsgrep> Error: Path does not exist: {}",
            path_to_string(config, dir)
        )
        .unwrap();
        return;
    }
    match fs::read_dir(dir) {
        Ok(mut entries) => {
            while let Some(entry) = entries.next() {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        search(&config, &mut output, string, &path, false);
                    }
                    Err(err) => {
                        let mut err_writer = ErrWriter::new(&mut output);
                        writeln!(
                            &mut err_writer.stream(),
                            "<rsgrep> Error: Iterating directory ({}): {}",
                            err,
                            path_to_string(config, dir)
                        )
                        .unwrap();
                    }
                }
            }
        }
        Err(err) => {
            let mut err_writer = ErrWriter::new(&mut output);
            writeln!(
                &mut err_writer.stream(),
                "<rsgrep> Error: Cannot iterate directory ({}): {}",
                err,
                path_to_string(config, dir)
            )
            .unwrap();
        }
    }
}

pub fn search(config: &Config, mut output: &mut Output, string: &str, path: &Path, initial: bool) {
    if !path.exists() {
        let mut err_writer = ErrWriter::new(&mut output);
        writeln!(
            &mut err_writer.stream(),
            "<rsgrep> Error: Path does not exist: {}",
            path_to_string(config, path)
        )
        .unwrap();
        return;
    }
    match resolve_path(&config, &mut output, path) {
        Some(path) => {
            if path.is_file() {
                search_file(&config, &mut output, string, &path);
            } else if path.is_dir() {
                if initial || config.recursive {
                    search_dir(&config, &mut output, string, &path);
                }
            } else {
                let mut err_writer = ErrWriter::new(&mut output);
                writeln!(
                    &mut err_writer.stream(),
                    "<rsgrep> Error: Cannot open path: {}",
                    path_to_string(config, &path)
                )
                .unwrap();
            }
        }
        None => {
            if config.warnings {
                let mut warn_writer = WarnWriter::new(&mut output);
                writeln!(
                    &mut warn_writer.stream(),
                    "<rsgrep> Warning: Ignoring path: {}",
                    path_to_string(config, path)
                )
                .unwrap();
            }
        }
    }
}
