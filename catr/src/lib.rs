use std::{error::Error, fs::File, io::{self, BufRead, BufReader}};

use clap::{App, Arg};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("OFFBLACK")
        .about("Rust cat")
        .arg(
            Arg::with_name("number_lines")
                .short("n")
                .long("number")
                .help("Number lines")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("number_nonblank_lines")
                .short("b")
                .long("number-nonblank")
                .help("Number nonblank lines")
                .conflicts_with("number_lines")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("files")
                .help("Files to process")
                .value_name("FILE")
                .multiple(true)
                .default_value("-")
        )
        .get_matches();
    
    Ok(
        Config {
            files: matches.values_of_lossy("files").unwrap(),
            number_lines: matches.is_present("number_lines"),
            number_nonblank_lines: matches.is_present("number_nonblank_lines"),       
        }
    )
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}


pub fn run(config: Config) -> MyResult<()> {
    for file in config.files {
        match open(&file) {
            Err(err) => eprintln!("Failed to open {}: {}", file, err),
            Ok(file_handle) => {
                let mut line_no = 1;
                for opt_line in file_handle.lines() {
                    if let Ok(line) = opt_line {
                        if config.number_lines {
                            println!("{:>6}\t{line}", line_no);
                            line_no += 1;
                        } else if config.number_nonblank_lines {
                            if line.is_empty() {
                                println!();
                            } else {
                                println!("{:>6}\t{line}", line_no);
                                line_no += 1;
                            }
                        } else {
                            println!("{line}");
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
