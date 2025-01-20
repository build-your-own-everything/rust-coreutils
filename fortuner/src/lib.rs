use std::{error::Error, fs::{self, File}, io::{BufRead, BufReader}, path::{Path, PathBuf}};

use clap::{App, Arg};
use rand::{rngs::{StdRng, ThreadRng}, seq::SliceRandom, SeedableRng};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
pub struct Fortune {
    source: String,
    text: String,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("fortuner")
        .about("Rust fortune")
        .author("OFFBLACK")
        .version("0.1.0")
        .arg(
            Arg::with_name("sources")
                .multiple(true)
                .value_name("FILE")
                .help("Input file(s)")
                .required(true)
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Ignore case for -m patterns")
        )
        .arg(
            Arg::with_name("pattern")
                .short("m")
                .long("pattern")
                .value_name("PATTERN")
                .help("Pattern")
        )
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .help("Random seed")
                .value_name("SEED")
        )
        .get_matches();

    let pattern = matches
        .value_of("pattern")
        .map(|p| {
            RegexBuilder::new(p)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
            .map_err(|_| format!("Invalid --pattern \"{p}\""))
        })
        .transpose()?;

    let seed = matches.value_of("seed")
        .map(|s| s.parse().map_err(|_| format!("\"{s}\" not a valid integer")))
        .transpose()?;


    Ok(Config {
        sources: matches.values_of_lossy("sources").unwrap(),
        pattern,
        seed    
    })
}

pub fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
        
    for path in paths {
        match fs::metadata(path) {
            Err(e) => return Err(format!("{path}: {e}").into()),
            Ok(_) => files.extend(
                WalkDir::new(path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file() )
                    .map(|e| e.path().into())
            ),
        }
    }

    files.sort();
    files.dedup();

    Ok(files)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut fortunes = Vec::new();
    let mut buffer = Vec::new();
    
    for path in paths {
        let basename = path.file_name().unwrap().to_string_lossy().into_owned();
        let file = File::open(path).map_err(|e| {
            format!("{}: {}", path.to_string_lossy().into_owned(), e)
        })?;

        for line in BufReader::new(file).lines().filter_map(Result::ok) {
            if line == "%" {
                if !buffer.is_empty() {
                    fortunes.push(Fortune {
                        source: basename.clone(),
                        text: buffer.join("\n")
                    });
                    buffer.clear();
                }
            } else {
                buffer.push(line.to_string());
            }
        }
    }
    Ok(fortunes)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    if let Some(seed) = seed {
        let mut seed = StdRng::seed_from_u64(seed);
        fortunes.choose(&mut seed).map(|f| f.text.clone())
    } else {
        let mut seed = rand::thread_rng();
        fortunes.choose(&mut seed).map(|f| f.text.clone())
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;
    if fortunes.is_empty() { 
        println!("No fortunes found") 
    } else {
        if let Some(pattern) = config.pattern {
            let mut prev_source = None;
            for fortune in fortunes {
                pattern.captures(&fortune.text)
                    .map(|_| {
                        if prev_source.as_ref()
                            .map_or(true, |s| s != &fortune.source) {
                            eprintln!("({})\n%", fortune.source);
                            prev_source = Some(fortune.source.clone());
                        }
                        println!("{}\n%", fortune.text)
                    });
            }
        } else {
            pick_fortune(&fortunes, config.seed)
                    .map(|f| println!("{}", f));
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::find_files;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }
}
