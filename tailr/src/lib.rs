use std::{error::Error, fs::File, io::{BufRead, BufReader, Read, Seek}};
use num::Zero;
use TakeValue::*;

use clap::{App, Arg};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<TakeValue>,
    quiet: bool,
}

#[derive(Debug, PartialEq)]
enum TakeValue {
    PlusZero,
    TakeNum(i64)
}

fn parse_num(val: &str) -> MyResult<TakeValue> {
    if val.starts_with("+") {
        if val.parse::<i64>()?.is_zero() {
            return Ok(PlusZero)
        }
        return Ok(TakeNum(val.parse()?))
    } else if val.starts_with("-") {
        return Ok(TakeNum(val.parse()?))
    } else {
        match val.parse::<i64>() {
            Ok(val) => return Ok(TakeNum(val * -1)),
            Err(_) => return Err(val.to_string().into()),
        }
    }
}

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let mut file = BufReader::new(File::open(filename)?);
    let mut line = String::new();
    let mut lines = 0;
    let mut bytes = 0i64;
    loop { 
        let bytes_read = file.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        lines += 1;
        bytes += bytes_read as i64;
        line.clear();
    }
    Ok((lines, bytes))
}

fn print_lines(
    mut file: impl BufRead,
    num_lines: &TakeValue,
    total_lines: i64,
) -> MyResult<()> {

    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut line_num = 0;
        let mut buf = String::new();
        loop {
            let bytes = file.read_line(&mut buf)?;
            if bytes == 0 {
                break;
            }
            if line_num >= start {
                print!("{buf}");
            }
            line_num += 1;
            buf.clear();
        }
        return Ok(())
    }
    Ok(())
}

fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_bytes: &TakeValue, 
    total_bytes: i64
) -> MyResult<()> {
    if let Some(start) = get_start_index(num_bytes, total_bytes) {
        file.seek(std::io::SeekFrom::Start(start))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        if !buf.is_empty() {
            print!("{}", String::from_utf8_lossy(&buf));
        }
    }
    Ok(())
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        PlusZero => if total > 0 { Some(0) } else { None },
        TakeNum(num) => {
            if num == &0 || total == 0 || *num > total {
                None
            } else {
                let start = if *num < 0 { total + num } else { num - 1 };
                Some(if start < 0 { 0 } else { start as u64 })
            }
        }
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("tailr")
        .about("Rust tail")
        .author("OFFBLACK")
        .version("0.1.0")
        .arg(
            Arg::with_name("files")
                .help("Input file(s)")
                .multiple(true)
                .required(true)
                .value_name("FILES")
        )
        .arg(
            Arg::with_name("lines")
                .help("Output last K lines")
                .short("n")
                .long("lines")
                .default_value("-10")
                .value_name("LINES")
        )
        .arg(
            Arg::with_name("bytes")
                .help("Output last K bytes")
                .short("c")
                .long("bytes")
                .value_name("BYTES")
                .conflicts_with("lines")
        )
        .arg(
            Arg::with_name("quiet")
                .help("Suppress printing of headers")
                .short("q")
                .long("quiet")
        )
        .get_matches();

    let lines = matches
        .value_of("lines")
        .map(parse_num)
        .unwrap()
        .map_err(|e| format!("illegal line count -- {e}"))?;
        

    let bytes = matches
        .value_of("bytes")
        .map(parse_num)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {e}"))?;

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        bytes,
        quiet: matches.is_present("quiet")
    })
}

pub fn run(config: Config) -> MyResult<()> {
    for (id, filename) in config.files.iter().enumerate() {
        match File::open(&filename) {
            Err(e) => eprintln!("{filename}: {e}"),
            Ok(file) => {
                let (total_lines, total_bytes) = count_lines_bytes(&filename)?;
                let file = BufReader::new(file);
                if !config.quiet && config.files.len() > 1 {
                    if id == 0 {
                        println!("==> {} <==", filename);
                    } else {
                        println!("\n==> {} <==", filename);
                    }
                }
                if let Some(ref take_val) = config.bytes {
                    print_bytes(file, &take_val, total_bytes)?;
                } else {
                    print_lines(file, &config.lines, total_lines)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{count_lines_bytes, parse_num, TakeValue::*};

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // A leading "+" should result in a positive number
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // An explicit "-" value should result in a negative number
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // Zero is zero
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // Plus zero is special
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // Test boundaries
        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // A floating-point value is invalid
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // Any non-integer string is invalid
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
}
