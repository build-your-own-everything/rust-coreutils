use crate::EntryType::*;
use clap::{Arg, App};
use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use std::error::Error;


type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .about("Rust find")
        .author("OFFBLACK")
        .version("0.1.0")
        .arg(
            Arg::with_name("name")
                .multiple(true)
                .short("n")
                .long("name")
                .help("Name")
                .value_name("NAME")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("type")
                .multiple(true)
                .short("t")
                .long("type")
                .possible_values(&["f", "d", "l"])
                .help("Entry type")
                .value_name("TYPE")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("paths")
                .default_value(".")
                .multiple(true)
                .value_name("PATH")
                .help("Search paths")
        )
       .get_matches();

    let names = matches.values_of_lossy("name")
        .map(|vals| {
            vals.into_iter()
                .map(|name| {
                    Regex::new(&name)
                        .map_err(|_| format!("Invalid --name \"{}\"", name))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    
    let entry_types = matches.values_of_lossy("type")
        .map(|vals|
            vals.iter()
                .map(|val| match val.as_str() {
                    "d" => Dir,
                    "f" => File,
                    "l" => Link,
                    _ => unreachable!("Invalid type")
                })
                .collect()
        )
        .unwrap_or_default();

    Ok(Config {
        paths: matches.values_of_lossy("paths").unwrap(),
        names,
        entry_types,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let type_filter = |entry: &DirEntry| {
        config.entry_types.is_empty()
            || config
                .entry_types
                .iter()
                .any(|entry_type| match entry_type {
                    File => entry.file_type().is_file(),
                    Dir => entry.file_type().is_dir(),
                    Link => entry.file_type().is_symlink(),
                })
    };

    let name_filter = |entry: &DirEntry| {
        config.names.is_empty()
            || config
                .names
                .iter()
                .any(|re| re.is_match(&entry.file_name().to_string_lossy()))
    };

    for path in &config.paths {
        let entries = WalkDir::new(path)
            .into_iter()
            .filter_map(|entry| {
                match entry {
                    Ok(entry) => Some(entry),
                    Err(e) => {
                        eprintln!("{e}");
                        None
                    }
                }
            })
            .filter(type_filter)
            .filter(name_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();

        println!("{}", entries.join("\n"))
    }
    Ok(())
}
