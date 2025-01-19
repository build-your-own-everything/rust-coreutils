use std::{
    cmp::Ordering::*, error::Error, fs::File, 
    io::{self, BufRead, BufReader}
};
use Col::*;

use clap::{Arg, App};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    file1: String,
    file2: String,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
    insensitive: bool,
    delimiter: String,
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)
            .map_err(|e| format!("{filename}: {e}"))?)))
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("commr")
        .about("Rust comm")
        .version("0.1.0")
        .author("OFFBLACK")
        .arg(
            Arg::with_name("file1")
                .value_name("FILE1")
                .help("Input file 1")
                .required(true)
        )
        .arg(
            Arg::with_name("file2")
                .value_name("FILE2")
                .help("Input file 2")
                .required(true)
        )
        .arg(
            Arg::with_name("suppress_col1")
                .short("1")
                .help("Suppress printing of column 1")
        )
        .arg(
            Arg::with_name("suppress_col2")
                .short("2")
                .help("Suppress printing of column 2")
        )
        .arg(
            Arg::with_name("suppress_col3")
                .short("3")
                .help("Suppress printing of column 3")
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive comparison of lines") 
        )
        .arg(
            Arg::with_name("delimiter")
                .short("d")
                .long("output-delimiter")
                .help("Output delimiter")
                .value_name("DELIM")
                .default_value("\t")
                .takes_value(true)
        )
        .get_matches();

    Ok(Config {
        file1: matches.value_of("file1").unwrap().to_string(),
        file2: matches.value_of("file2").unwrap().to_string(),
        show_col1: !matches.is_present("suppress_col1"),
        show_col2: !matches.is_present("suppress_col2"),
        show_col3: !matches.is_present("suppress_col3"),
        insensitive: matches.is_present("insensitive"),
        delimiter: matches.value_of("delimiter").unwrap().to_string(),
    })
}

enum Col<'a> {
    Col1(&'a str),
    Col2(&'a str),
    Col3(&'a str),
}

pub fn run(config: Config) -> MyResult<()> {
    if &config.file1 == "-" && &config.file2 == "-" {
        return Err("Both input files cannot be STDIN (\"-\")".into())
    }

    let case = |line: String| {
        if config.insensitive {
            line.to_lowercase()
        } else {
            line
        }
    };

    let mut lines1 = open(&config.file1)?
        .lines()
        .filter_map(Result::ok)
        .map(case);

    let mut lines2 = open(&config.file2)?
        .lines()
        .filter_map(Result::ok)
        .map(case);
    
    let mut line1 = lines1.next();
    let mut line2 = lines2.next();

    let print = |col: Col| {
        let mut cols = Vec::new();
        match col {
            Col1(val) => {
                if config.show_col1 {
                    cols.push(val);
                }
            },
            Col2(val) => {
                if config.show_col2 {
                    if config.show_col1 {
                        cols.push("")
                    }
                    cols.push(val);
                }
            },
            Col3(val) => {
                if config.show_col3 {
                    if config.show_col1 {
                        cols.push("");
                    }
                    if config.show_col2 {
                        cols.push("");
                    }
                    cols.push(val);
                }
            }
        }
        
        if !cols.is_empty() {
            println!("{}", cols.join(&config.delimiter));
        }
    };

    while line1.is_some() || line2.is_some() {
        match (&line1, &line2) {
            (Some(val1), Some(val2)) => match val1.cmp(val2) {
                Equal => {
                    print(Col3(val1));
                    line1 = lines1.next();
                    line2 = lines2.next();
                },
                Less => {
                    print(Col1(val1));
                    line1 = lines1.next();
                },
                Greater => {
                    print(Col2(val2));
                    line2 = lines2.next();
                },
            },
            (Some(val1), None) => {
                print(Col1(val1));
                line1 = lines1.next();
            },
            (None, Some(val2)) => {
                print(Col2(val2));
                line2 = lines2.next();
            }
            _ => {},
        }
    }

    Ok(())
}
