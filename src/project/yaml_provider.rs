use chrono::prelude::*;
use yaml_rust::{Yaml, yaml::Hash as YamlHash};

use crate::util::yaml::{parse_dmy_date, parse_dmy_date_range};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldValue<T> {
    Missing,
    Invalid(String),
    Ok(T)
}

impl<T> FieldValue<T> {
    pub fn ok(self) -> Option<T> {
        match self {
            FieldValue::Ok(v) => Some(v),
            _ => None,
        }
    }

    pub fn or_else<F: FnOnce() -> FieldValue<T>>(self, f: F) -> FieldValue<T> {
        match self {
            FieldValue::Ok(v) => FieldValue::Ok(v),
            FieldValue::Invalid(x) => FieldValue::Invalid(x),
            _ => f()
        }
    }

    pub fn valid_else<F: FnOnce() -> FieldValue<T>>(self, f: F) -> FieldValue<T> {
        match self {
            FieldValue::Ok(v) => FieldValue::Ok(v),
            _ => f()
        }
    }

    pub fn and_parse<U, E: ToString, F: FnOnce(T) -> Result<U, E>>(self, f: F) -> FieldValue<U> {
        match self {
            FieldValue::Ok(v) => match f(v) {
                Err(e) => FieldValue::Invalid(e.to_string()),
                Ok(x) => FieldValue::Ok(x)
            }
            FieldValue::Invalid(m) => FieldValue::Invalid(m),
            FieldValue::Missing => FieldValue::Missing,
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> FieldValue<U> {
        match self {
            FieldValue::Ok(v) => FieldValue::Ok(f(v)),
            FieldValue::Invalid(m) => FieldValue::Invalid(m),
            FieldValue::Missing => FieldValue::Missing,
        }
    }

    pub fn unwrap_or(self, def: T) -> T {
        match self {
            FieldValue::Ok(v) => v,
            _ => def,
        }
    }

    pub fn filter_map<U, F: FnOnce(T) -> Option<U>>(self, f: F) -> FieldValue<U> {
        match self {
            FieldValue::Ok(v) => match f(v) {
                None => FieldValue::Missing,
                Some(x) => FieldValue::Ok(x)
            }
            FieldValue::Invalid(m) => FieldValue::Invalid(m),
            FieldValue::Missing => FieldValue::Missing,
        }
    }

    pub fn unwrap(self) -> T {
        self.ok().unwrap()
    }

    pub fn is_ok(&self) -> bool {
        if let FieldValue::Ok(_) = self { true } else { false }
    }
}
    


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
        ).nth(0)
    }

    /// Wrapper around `get_path()`.
    ///
    /// Splits path string
    /// and replaces `Yaml::Null` and `Yaml::BadValue`.
    fn get_direct<'a>(&'a self, data: &'a Yaml, path: &str) -> Option<&'a Yaml> {
        // TODO: this can be without copying
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
    fn field<'a, T, F> (&'a self, path: &str, err: &str, parser: F) -> FieldValue<T>
    where F: FnOnce(&'a Yaml) -> Option<T> {
        match self.get(path) {
            None => FieldValue::Missing,
            Some(ref node) => match parser(node) {
                None => FieldValue::Invalid(err.to_string()),
                Some(parsed) => FieldValue::Ok(parsed)
            }
        }
    }

    /// Gets a `&str` value.
    ///
    /// Same mentality as `yaml_rust`, only returns `Some`, if it's a `Yaml::String`.
    fn get_str<'a>(&'a self, path: &str) -> FieldValue<&'a str> {
        self.field(path, "not a string", Yaml::as_str)
    }

    /// Gets an `Int` value.
    ///
    /// Same mentality as `yaml_rust`, only returns `Some`, if it's a `Yaml::Int`.
    fn get_int<'a>(&'a self, path: &str) -> FieldValue<i64> {
        self.field(path, "not an integer", Yaml::as_i64)
    }

    /// Gets a Date in `dd.mm.YYYY` format.
    fn get_dmy(&self, path: &str) -> FieldValue<Date<Utc>> {
        self.field(path, "not a date", |x| x.as_str().and_then(parse_dmy_date))
    }


    /// Gets a Date in `dd.mm.YYYY` or `dd-dd.mm.YYYY` format.
    fn get_dmy_legacy_range(&self, path: &str) -> FieldValue<Date<Utc>> {
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
    fn get_bool(&self, path: &str) -> FieldValue<bool> {
        self.field(path, "not a boolean", |y| y
                   .as_bool()
                   // allowing it to be a str: "yes" or "no"
                   .or_else(|| y
                            .as_str()
                            .map(|yes_or_no|
                                match yes_or_no.to_lowercase().as_ref() {
                                    "yes" => true,
                                    //"no" => false,
                                    _ => false
                                })
                   )
        )
    }

    /// Gets `Some(Yaml::Hash)` or `None`.
    //pub fn get_hash<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a BTreeMap<Yaml,Yaml>> {
    fn get_hash<'a>(&'a self, path: &str) -> FieldValue<&'a YamlHash> {
        self.field(path, "not a hash", Yaml::as_hash)
    }

    /// Gets a `Float` value.
    ///
    /// Also takes a `Yaml::I64` and reinterprets it.
    fn get_f64(&self, path: &str) -> FieldValue<f64> {
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


    static NO_FALLBACK_PATH: &'static str = r#"
    offer:
        date: 07.11.2019
    "#;

    static FALLBACK_PATH: &'static str = r#"
    offer_date: 08.11.2019
    "#;

    #[test]
    fn find_fallback_paths() {
        let no_fallback = TestProvider::parse(NO_FALLBACK_PATH);
        let fallback = TestProvider::parse(FALLBACK_PATH);

        assert_eq!(no_fallback.get_str("offer.date|offer_date"), FieldValue::Ok("07.11.2019"));
        assert_eq!(fallback.get_str("offer.date|offer_date"), FieldValue::Ok("08.11.2019"));

        assert_eq!(no_fallback.get_str("offer.date"), FieldValue::Ok("07.11.2019"));
        assert_eq!(fallback.get_str("offer_date"), FieldValue::Ok("08.11.2019"));

        assert_eq!(no_fallback.get_str("offer_date"), FieldValue::Missing);
        assert_eq!(fallback.get_str("offer.date"), FieldValue::Missing);


    }


}
