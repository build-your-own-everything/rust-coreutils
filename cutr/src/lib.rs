use clap::{App, Arg};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use regex::Regex;

use crate::Extract::*;
use std::{error::Error, fs::File, io::{self, BufRead, BufReader}, num::NonZeroUsize, ops::Range};

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

pub fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?)))
    }
}

fn parse_index(input: &str) -> Result<usize, String> {
    let value_err = || format!("illegal list value: \"{}\"", input);
    input
        .starts_with('+')
        .then(|| Err(value_err()))
        .unwrap_or_else(|| {
            input
                .parse::<NonZeroUsize>()
                .map(|n| usize::from(n) - 1)
                .map_err(|_| value_err())
        })
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    let range_re = Regex::new(r"^(\d+)-(\d+)$").unwrap();
    range
        .split(',')
        .into_iter()
        .map(|val| {
            parse_index(val).map(|n| n..n+1).or_else(|e| {
                range_re.captures(val).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 > n2 {
                        return Err(format!(
                            "First number in range ({}) \
                            must be lower than second number ({})",
                            n1 + 1,
                            n2 + 1
                        ))
                    }
                    Ok(n1..n2+1)
                })
            })
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)

}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .about("Rust cut")
        .author("OFFBLACK")
        .version("0.1.0")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-")
        )
        .arg(
            Arg::with_name("delimiter")
                .short("d")
                .long("delim")
                .value_name("DELIMITER")
                .help("Field delimiter")
                .default_value("\t")
        )
        .arg(
            Arg::with_name("bytes")
                .short("b")
                .long("bytes")
                .value_name("BYTES")
                .help("Selected bytes")
                .conflicts_with_all(&["chars", "fields"])
        )
        .arg(
            Arg::with_name("chars")
                .short("c")
                .long("chars")
                .value_name("CHARS")
                .help("Selected characters")
                .conflicts_with_all(&["bytes", "fields"])
        )
        .arg(
            Arg::with_name("fields")
                .short("f")
                .long("fields")
                .value_name("FIELDS")
                .help("Selected fields")
                .conflicts_with_all(&["chars", "bytes"])
        )
        .get_matches();

    let delimiter = matches.value_of("delimiter").unwrap();
    let delim_bytes = delimiter.as_bytes();
    if delim_bytes.len() != 1 {
        return Err(
            From::from(format!("--delim \"{delimiter}\" must be a single byte"))
        );
    }
    let fields = matches.value_of("fields").map(parse_pos).transpose()?;
    let bytes = matches.value_of("bytes").map(parse_pos).transpose()?;
    let chars = matches.value_of("chars").map(parse_pos).transpose()?;
    let extract = if let Some(fields_pos) = fields {
        Fields(fields_pos)
    } else if let Some(bytes_pos) = bytes {
        Bytes(bytes_pos)
    } else if let Some(chars_pos) = chars {
        Chars(chars_pos)
    } else {
        return Err(From::from("Must have --fields, --bytes, or --chars"));
    };
    Ok({
        Config { 
            files: matches.values_of_lossy("files").unwrap(), 
            delimiter: *delim_bytes.first().unwrap(), 
            extract,
        }
    })
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let chars: Vec<_> = line.chars().collect();
    char_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|v| chars.get(v)))
        .collect()
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes: Vec<_> = line.bytes().collect();
    let selected: Vec<_> = byte_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|v| bytes.get(v)).copied())
        .collect();

    String::from_utf8_lossy(&selected).into_owned()
}

fn extract_fields<'a>(
    record: &'a StringRecord,
    field_pos: &[Range<usize>]
) -> Vec<&'a str> {
    field_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|v| record.get(v)))
        .collect()
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Ok(file) => match &config.extract {
                Fields(field_pos) => {
                    let mut reader = ReaderBuilder::new()
                        .delimiter(config.delimiter)
                        .has_headers(false)
                        .from_reader(file);

                    let mut writer = WriterBuilder::new()
                        .delimiter(config.delimiter)
                        .from_writer(io::stdout());
                        
                    for record in reader.records() {
                        let record = record?;
                        writer.write_record(extract_fields(
                            &record, field_pos,
                        ))?;
                    }
                },
                Chars(char_pos) => {
                    for line in file.lines() {
                        println!("{}", extract_chars(&line?, char_pos));
                    }
                },
                Bytes(byte_pos) => {
                    for line in file.lines() {
                        println!("{}", extract_bytes(&line?, byte_pos));
                    }
                }
            },
            Err(e) => eprintln!("{}: {e}", filename),
        }
    }
    Ok(())
}
