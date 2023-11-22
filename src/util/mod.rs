//! Utility functions that are needed all over the places.
use std::{
    env::{self, current_dir},
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::{self, Command, ExitStatus},
};

use bill::Currency;
use chrono::NaiveTime;

use self::dirs::home_dir;
use anyhow::{Context, Error};

pub mod dirs;
pub mod yaml;

/// Sets up logging initially.
///
/// After calling this function the global logger will look for the environment variable `ASCIII_LOG`.
pub fn setup_log() {
    let log_var = "ASCIII_LOG";
    let mut logger = env_logger::Builder::from_env(log_var);

    if env::var(log_var).is_err() {
        logger
            .format_level(true)
            .format_module_path(false)
            .filter_level(log::LevelFilter::Info);
    }

    logger.init();
}

/// Freezes the program until for inspection
pub fn freeze() {
    io::stdin().read_line(&mut String::new()).unwrap();
}

/// Asks for confirmation
pub fn really(msg: &str) -> bool {
    println!("{} [y/N]", msg);
    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() {
        return false;
    }
    ["yes", "y", "j", "ja", "oui", "si", "da"].contains(&answer.trim())
}

pub fn git_user_name() -> Option<String> {
    Command::new("git")
        .args(["config", "user.name"])
        .output()
        .map_err(|error| {
            log::error!("failed to execute process: {error}");
            error
        })
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_owned())
}

/// Shells out to print directory structure
pub fn ls(path: &str) {
    println!("find {}", path); // TODO: implement in here with walkdir
    let output = Command::new("find")
        .arg(path)
        .output()
        .unwrap_or_else(|error| panic!("failed to execute process: {error}"));
    println!("{}", String::from_utf8(output.stdout).unwrap());
}

/// Replaces `~` with `$HOME`, rust stdlib doesn't do this yet.
///
/// This is by far the most important function of all utility functions.
///
/// **TODO** add something like this to the stdlib
/// **TODO** ~ must be first character
pub fn replace_home_tilde(p: &Path) -> PathBuf {
    let path = p.to_str().unwrap();
    PathBuf::from(path.replace('~', home_dir().unwrap().to_str().unwrap()))
}

/// Opens the passed paths in the editor set int config.
///
/// This is by far the most important function of all utility functions.
//TODO: use https://crates.io/crates/open (supports linux, windows, mac)
pub fn pass_to_command<T: AsRef<OsStr>>(editor: Option<&str>, paths: &[T]) -> Result<(), Error> {
    let paths = paths
        .iter()
        .map(|o| PathBuf::from(&o))
        .filter(|p| p.exists())
        .collect::<Vec<PathBuf>>();

    if paths.is_empty() {
        log::warn!("non of the provided paths could be found")
    } else if let Some(editor) = editor {
        if paths.len() < 5
            || really(&format!(
                "you are about to open {} files\n{:#?}\nAre you sure about this?",
                paths.len(),
                paths
            ))
        {
            let editor_config = editor.split_whitespace().collect::<Vec<&str>>();

            let (editor_command, args) = editor_config.split_first().unwrap();
            log::info!(
                "launching {:?} with {:?} and {:?}",
                editor_command,
                args.join(" "),
                paths
            );

            Command::new(editor_command)
                .args(args)
                .args(&paths)
                .status()
                .context("failed to execute process")?;
        }
    } else {
        for path in paths {
            open::that(path).unwrap();
        }
    }

    Ok(())
}

/// Deletes the file if the passed in closure returns `true`
pub fn delete_file_if<F, P: AsRef<OsStr>>(path: P, confirmed: F) -> io::Result<()>
where
    F: Fn() -> bool,
{
    let path = PathBuf::from(&path);
    if confirmed() {
        log::debug!("$ rm {}", path.display());
        fs::remove_file(&path)
    } else {
        Ok(())
    }
}

/// takes a path that could be relative or contains a `~` and turn it into a path that exists
pub fn get_valid_path<T: AsRef<OsStr>>(p: T) -> Option<PathBuf> {
    let path = replace_home_tilde(Path::new(&p));
    let path = if path.is_relative() {
        if cfg!(target_arch = "wasm32") {
            Ok(PathBuf::from(std::env::var("PWD").expect("can't access $PWD")))
        } else {
            current_dir()
        }
        .unwrap()
        .join(path)
    } else {
        path
    };

    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Exits with the exit status of a child process.
pub fn exit(status: ExitStatus) -> ! {
    process::exit(status.code().unwrap_or(1));
}

/// One place to decide how to display currency
pub fn currency_to_string(currency: &Currency) -> String {
    currency.postfix().to_string()
}

/// Creates a currency from an `f64`
///
/// This is functionality which was explicitly left out of the `Claude` crate.
pub fn to_currency(f: f64) -> Currency {
    Currency {
        symbol: crate::CONFIG.get_char("currency"),
        value: (f * 1000.0) as i64,
    } / 10
}

/// Changes the extension of a given `Path`
pub fn to_local_file(file: &Path, ext: &str) -> PathBuf {
    let mut _tmpfile = file.to_owned();
    _tmpfile.set_extension(ext);
    Path::new(_tmpfile.file_name().unwrap()).to_owned()
}

/// Creates a `chrono::NaiveTime` from a string that looks like `23:59:58` or only `12:05`.
pub fn naive_time_from_str(string: &str) -> Option<NaiveTime> {
    let t: Vec<u32> = string
        .splitn(2, |p| p == '.' || p == ':')
        .map(|s| s.parse().unwrap_or(0))
        .collect();

    if let (Some(h), m) = (t.first(), t.get(1).unwrap_or(&0)) {
        if *h < 24 && *m < 60 {
            return Some(NaiveTime::from_hms(*h, *m, 0));
        }
    }

    None
}

#[test]
fn test_naive_time_from_str() {
    assert_eq!(Some(NaiveTime::from_hms(9, 15, 0)), naive_time_from_str("9.15"));
    assert_eq!(Some(NaiveTime::from_hms(9, 0, 0)), naive_time_from_str("9."));
    assert_eq!(Some(NaiveTime::from_hms(9, 0, 0)), naive_time_from_str("9"));
    assert_eq!(Some(NaiveTime::from_hms(23, 0, 0)), naive_time_from_str("23.0"));
    assert_eq!(Some(NaiveTime::from_hms(23, 59, 0)), naive_time_from_str("23.59"));
    assert_eq!(None, naive_time_from_str("24.0"));
    assert_eq!(None, naive_time_from_str("25.0"));
    assert_eq!(None, naive_time_from_str("0.60"));

    assert_eq!(Some(NaiveTime::from_hms(9, 15, 0)), naive_time_from_str("9:15"));
    assert_eq!(Some(NaiveTime::from_hms(9, 0, 0)), naive_time_from_str("9:"));
    assert_eq!(Some(NaiveTime::from_hms(9, 0, 0)), naive_time_from_str("9"));
    assert_eq!(Some(NaiveTime::from_hms(23, 0, 0)), naive_time_from_str("23:0"));
}
