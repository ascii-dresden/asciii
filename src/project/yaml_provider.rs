use chrono::prelude::*;
#[allow(unused_imports)]
use yaml_rust::{Yaml, yaml::Hash as YamlHash};

use crate::util::yaml::{parse_dmy_date, parse_dmy_date_range};

pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug, PartialOrd, Ord, PartialEq, Eq)]
    pub enum FieldError {
        #[error("The expected field is missing")]
        Missing,

        #[error("The field has an invalid value or type")]
        Invalid(String),
    }

    impl FieldError {
        pub fn invalid(e: &str) -> FieldError {
            FieldError::Invalid(e.to_owned())
        }
    }

    pub type FieldResult<T> = Result<T, FieldError>;

    pub trait FieldResultExt<T> {
        /// Tries an alternative only if the original is actually Missing.
        /// 
        /// This makes sure we don't accidentally fall back to an old spec value if the original is invalid.
        fn if_missing_try<F: FnOnce() -> FieldResult<T>>(self, f: F) -> FieldResult<T>;
    }

    impl<T> FieldResultExt<T> for FieldResult<T> {
        fn if_missing_try<F: FnOnce() -> FieldResult<T>>(self, f: F) -> FieldResult<T> {
            log::debug!("{:?}", self.as_ref().err());
            if let Err(FieldError::Missing) = self {
                f()
            } else {
                self
            }
        }
    }
}

pub use error::{FieldResult, FieldError};

/// Enables access to structured data via a simple path
///
/// A path can be something like `users/clients/23/name`
/// but also  `users.clients.23.name`
pub trait YamlProvider {
    /// You only need to implement this.
    //fn data(&self) -> impl PathAccessible {
    fn data(&self) -> &Yaml;

    /// Wrapper around `get_path()`.
    ///
    /// Splits path string
    /// and replaces `Yaml::Null` and `Yaml::BadValue`.
    fn get<'a>(&'a self, paths: &str) -> Option<&'a Yaml> {
        paths.split('|').filter_map(|path|
            self.get_direct(self.data(), path)
        ).next()
    }

    /// Wrapper around `get_path()`.
    ///
    /// Splits path string
    /// and replaces `Yaml::Null` and `Yaml::BadValue`.
    fn get_direct<'a>(&'a self, data: &'a Yaml, path: &str) -> Option<&'a Yaml> {
        // TODO: this can be without copying
        debug_assert!(!path.chars().any(char::is_whitespace), "paths shouldn't contain whitespaces {:?}", path);
        let path = path.split(|p| p == '/' || p == '.')
            .filter(|k| !k.is_empty())
            .collect::<Vec<&str>>();
        match self.get_path(data, &path) {
            Some(&Yaml::BadValue) |
            Some(&Yaml::Null) => None,
            content => content,
        }
    }

    /// Returns content at `path` in the yaml document.
    /// TODO: make this generic over the type of data to support more than just `Yaml`.
    fn get_path<'a>(&'a self, data: &'a Yaml, path: &[&str]) -> Option<&'a Yaml> {
        if let Some((&path, remainder)) = path.split_first() {
            match *data {
                // go further into the rabbit hole
                Yaml::Hash(ref hash) => {
                    if remainder.is_empty() {
                        hash.get(&Yaml::String(path.to_owned()))
                    } else {
                        hash.get(&Yaml::String(path.to_owned()))
                            .and_then(|c| self.get_path(c, remainder))
                    }
                }
                // interpret component as index
                Yaml::Array(ref vec) => {
                    if let Ok(index) = path.parse::<usize>() {
                        if remainder.is_empty() {
                            vec.get(index)
                        } else {
                            vec.get(index).and_then(|c| self.get_path(c, remainder))
                        }
                    } else { None }
                },
                // return none, because the path is longer than the data structure
                _ => None
            }
        } else {
            None
        }
    }

    /// Gets the field for a given path.
    fn field<'a, T, F> (&'a self, path: &str, err: &str, parser: F) -> FieldResult<T>
    where F: FnOnce(&'a Yaml) -> Option<T> {
        let res = self.get(path);
        log::debug!("{}::get({:?}) -> {:?}", module_path!(), path, res);
        match res {
            None => Err(FieldError::Missing),
            Some(ref node) => match parser(node) {
                None => Err(FieldError::Invalid(lformat!("{} ({:?})", err, node))),
                Some(parsed) => FieldResult::Ok(parsed),
            }
        }
    }

    /// Gets a `&str` value.
    ///
    /// Same mentality as `yaml_rust`, only returns `Some`, if it's a `Yaml::String`.
    fn get_str<'a>(&'a self, path: &str) -> FieldResult<&'a str> {
        self.field(path, "not a string", Yaml::as_str)
    }

    /// Gets an `Int` value.
    ///
    /// Same mentality as `yaml_rust`, only returns `Some`, if it's a `Yaml::Int`.
    fn get_int<'a>(&'a self, path: &str) -> FieldResult<i64> {
        self.field(path, "not an integer", Yaml::as_i64)
    }

    /// Gets a Date in `dd.mm.YYYY` format.
    fn get_dmy(&self, path: &str) -> FieldResult<Date<Utc>> {
        self.field(path, "not a date", |x| x.as_str().and_then(parse_dmy_date))
    }

    /// Gets a Date in `dd.mm.YYYY` or `dd-dd.mm.YYYY` format.
    fn get_dmy_legacy_range(&self, path: &str) -> FieldResult<Date<Utc>> {
        self.field(path, "neither date nor date range", |n| {
            n.as_str().and_then(|v| {
                parse_dmy_date(v).or_else(|| parse_dmy_date_range(v))
                })
        })
    }

    /// Gets a `Bool` value.
    ///
    /// **Careful** this is a bit sweeter then ordinary `YAML1.2`,
    /// this will interpret `"yes"` and `"no"` as booleans, similar to `YAML1.1`.
    /// Actually it will interpret any string but `"yes"` als `false`.
    fn get_bool(&self, path: &str) -> FieldResult<bool> {
        self.field(path, "not a boolean", |y| y
            .as_bool()
                // allowing it to be a str: "yes" or "no"
                .or_else(|| y
                    .as_str()
                        .map(|yes_or_no| yes_or_no.to_lowercase().as_str() == "yes"))
        )
    }

    /// Gets `Some(Yaml::Hash)` or `None`.
    //pub fn get_hash<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a BTreeMap<Yaml,Yaml>> {
    fn get_hash<'a>(&'a self, path: &str) -> FieldResult<&'a YamlHash> {
        self.field(path, "not a hash", Yaml::as_hash)
    }

    /// Gets a `Float` value.
    ///
    /// Also takes a `Yaml::I64` and reinterprets it.
    fn get_f64(&self, path: &str) -> FieldResult<f64> {
        self.field(path, "not a float", |y| {
            y.as_f64().or_else(|| y.as_i64().map(|y| y as f64))
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::yaml::parse;

    struct TestProvider {
        yaml: Yaml,
    }

    impl TestProvider {
        pub fn parse(src: &str) -> Self {
            Self {
                yaml: parse(src).unwrap()
            }
        }
    }

    impl YamlProvider for TestProvider {
        fn data(&self) -> &Yaml {
            &self.yaml
        }
    }


    static NO_FALLBACK_PATH: &str = r#"
    offer:
        date: 07.11.2019
    "#;

    static FALLBACK_PATH: &str = r#"
    offer_date: 08.11.2019
    "#;

    #[test]
    fn find_fallback_paths() {
        let no_fallback = TestProvider::parse(NO_FALLBACK_PATH);
        let fallback = TestProvider::parse(FALLBACK_PATH);

        assert_eq!(no_fallback.get_str("offer.date|offer_date"), FieldResult::Ok("07.11.2019"));
        assert_eq!(fallback.get_str("offer.date|offer_date"), FieldResult::Ok("08.11.2019"));

        assert_eq!(no_fallback.get_str("offer.date"), FieldResult::Ok("07.11.2019"));
        assert_eq!(fallback.get_str("offer_date"), FieldResult::Ok("08.11.2019"));

        assert_eq!(no_fallback.get_str("offer_date"), FieldResult::Err(FieldError::Missing));
        assert_eq!(fallback.get_str("offer.date"), FieldResult::Err(FieldError::Missing));
    }

    #[test]
    #[should_panic]
    fn paths_forbid_whitespaces() {
        let fallback = TestProvider::parse(FALLBACK_PATH);
        assert_eq!(fallback.get_str("offer.date | offer_date"), FieldResult::Ok("08.11.2019"));
    }

}
