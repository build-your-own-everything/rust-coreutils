use std::{error::Error, fs::{self, File}, io::{self, BufRead, BufReader}, mem};

use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

fn find_lines<T: BufRead>(
    mut file: T, 
    pattern: &Regex,
    invert_match: bool
) -> MyResult<Vec<String>> {
    let mut matches = Vec::new();
    let mut line = String::new();

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        if pattern.is_match(&line) ^ invert_match {
            matches.push(mem::take(&mut line));
        }
        line.clear();
    }
    Ok(matches)
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut results = Vec::new();

    for path in paths {
        match path.as_str() {
            "-" => results.push(Ok(path.to_string())),
            _ => match fs::metadata(path) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        if recursive {
                            for entry in WalkDir::new(path)
                                .into_iter()
                                .flatten()
                                .filter(|e| e.file_type().is_file())
                            {
                                results.push(Ok(entry
                                    .path()
                                    .display()
                                    .to_string()));
                            }
                        } else {
                            results.push(
                                Err(format!("{path} is a directory").into())
                            );
                        }
                    } else if metadata.is_file() {
                        results.push(Ok(path.to_string()));
                    }
                },
                Err(e) => results.push(Err(format!("{path}: {e}").into())),
            }
        }
    }
    results
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("grepr")
        .about("Rust grep")
        .version("0.1.0")
        .author("OFFBLACK")
        .arg(
            Arg::with_name("pattern")
                .help("Search pattern")
                .value_name("PATTERN")
                .required(true)
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Count occurences")
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive")
        )
        .arg(
            Arg::with_name("invert")
                .short("v")
                .long("invert-match")
                .help("Invert match")
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .help("Recursive search")
        )
        .arg(
            Arg::with_name("files")
                .multiple(true)
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-")
        )
        .get_matches();

    let pattern = matches.value_of("pattern").unwrap();
    let pattern = RegexBuilder::new(pattern)
        .case_insensitive(matches.is_present("insensitive"))
        .build()
        .map_err(|_| format!("Invalid pattern \"{pattern}\""))?;

    Ok(Config {
        pattern,
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invert"),
        files: matches.values_of_lossy("files").unwrap(),
    }) 
}

pub fn run(config: Config) -> MyResult<()> {
    let entries = find_files(&config.files, config.recursive);
    let num_files = entries.len();
    let print = |fname: &str, content: &str| {
        if num_files > 1 { print!("{fname}:{content}") }
        else { print!("{content}") }
    };
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{e}"),
            Ok(filename) => match open(&filename) {
                Err(e) => eprintln!("{filename}: {e}"),
                Ok(file) => {
                    match find_lines(
                        file, &config.pattern, 
                        config.invert_match
                    ) {
                        Err(e) => eprintln!("{e}"),
                        Ok(matches) => {
                            if config.count {
                                print(
                                    &filename, 
                                    &format!("{}\n", matches.len())
                                );
                            } else {
                                for line in &matches {
                                    print(&filename, line);
                                }
                            }
                        }
                    }
                }
            },
        }
    }
    Ok(())
}
