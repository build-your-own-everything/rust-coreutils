use clap::{App, Arg};
use std::{error::Error, fs::File, io::{self, BufRead, BufReader, Write}};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .about("Rust uniq")
        .version("0.1.0")
        .author("OFFBLACK")
        .arg(
            Arg::with_name("count")
                .help("number lines")
                .short("c")
                .long("count")
        )
        .arg(
            Arg::with_name("in_file")
                .value_name("INPUT")
                .help("Input file")
                .default_value("-")
        )
        .arg(
            Arg::with_name("out_file")
                .help("Output file")
                .value_name("OUTPUT")
        )
        .get_matches();

    Ok(Config {
        in_file: matches.value_of_lossy("in_file").unwrap().to_string(),
        out_file: matches.value_of("out_file").map(|v| v.to_string()),
        count: matches.is_present("count")
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file)
        .map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut prev_line = String::new();
    let mut count = 0u64;
    let mut outfile: Box<dyn Write> = match config.out_file {
        Some(out_file) => Box::new(File::create(out_file)?),
        None => Box::new(io::stdout()),
    };
    let mut output = |count: u64, line: &str| -> MyResult<()> {
        if count > 0 {
            match config.count {
                true => write!(outfile, "{:>4} {}", count, line)?,
                false => write!(outfile, "{line}")?,
            };
        }
        Ok(())
    };
    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        if line.trim_end() != prev_line.trim_end() {
            output(count, &prev_line)?;
            prev_line = line.clone();
            count = 0;
        }

        count += 1;
        line.clear();
    }

    output(count, &prev_line)?;
    Ok(())
}
