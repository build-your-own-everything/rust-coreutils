use std::{error::Error, fs, os::unix::fs::MetadataExt, path::PathBuf};

use chrono::{DateTime, Local};
use clap::{App, Arg};
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    long: bool,
    show_hidden: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("lsr")
        .about("Rust ls")
        .author("OFFBLACK")
        .version("0.1.0")
        .arg(
            Arg::with_name("paths")
                .help("Files and/or directories")
                .default_value(".")
                .multiple(true)
                .value_name("PATH"),
        )
        .arg(
            Arg::with_name("show_hidden")
                .short("a")
                .long("all")
                .help("Show all files"),
        )
        .arg(
            Arg::with_name("long")
                .short("l")
                .long("long")
                .help("Long listing"),
        )
        .get_matches();

    Ok(Config {
        paths: matches.values_of_lossy("paths").unwrap(),
        show_hidden: matches.is_present("show_hidden"),
        long: matches.is_present("long"),
    })
}

fn find_files(paths: &[String], show_hidden: bool) -> MyResult<Vec<PathBuf>> {
    let mut results = Vec::new();
    for path in paths {
        match fs::metadata(path) {
            Err(e) => eprintln!("{path}: {e}"),
            Ok(file) if file.is_file() => {
                results.push(PathBuf::from(path));
            }
            Ok(dir) if dir.is_dir() => {
                for file in fs::read_dir(path)? {
                    let file = file?;
                    if show_hidden || !file.file_name().to_string_lossy().starts_with(".") {
                        results.push(PathBuf::from(file.path()));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(results)
}

fn format_output(paths: &[PathBuf]) -> MyResult<String> {
    let fmt = "{:<}{:<} {:>} {:<} {:<} {:>} {:<} {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let metadata = path.metadata()?;

        let uid = metadata.uid();
        let user = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| uid.to_string());
        let gid = metadata.gid();
        let group = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| gid.to_string());

        let last_modified: DateTime<Local> = DateTime::from(metadata.modified()?);

        table.add_row(
            Row::new()
                .with_cell(if path.is_dir() { "d" } else { "-" })
                .with_cell(format_mode(metadata.mode()))
                .with_cell(metadata.nlink())
                .with_cell(user)
                .with_cell(group)
                .with_cell(metadata.len())
                .with_cell(last_modified.format("%b %d %y %H:%M"))
                .with_cell(path.display()),
        );
    }

    Ok(format!("{}", table))
}

fn format_mode(mode: u32) -> String {
    let mut result = String::new();

    const BIT_MASKS: [u32; 9] = [
        0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001,
    ];

    for chunk in BIT_MASKS.chunks(3) {
        if let [r, w, x] = chunk {
            result.push_str(
                format!(
                    "{}{}{}",
                    if r & mode != 0 { "r" } else { "-" },
                    if w & mode != 0 { "w" } else { "-" },
                    if x & mode != 0 { "x" } else { "-" },
                )
                .as_str(),
            );
        }
    }
    result
}

pub fn run(config: Config) -> MyResult<()> {
    let paths = find_files(&config.paths, config.show_hidden)?;
    if config.long {
        println!("{}", format_output(&paths)?);
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{find_files, format_mode};

    #[test]
    fn test_find_files() {
        // Find all non-hidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        // Find all entries in a directory including hidden
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }
}
