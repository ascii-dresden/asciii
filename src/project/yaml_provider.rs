use chrono::prelude::*;
use yaml_rust::{Yaml, yaml::Hash as YamlHash};

use crate::util::yaml::parse_dmy_date;

use super::error::ErrorList;

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

    /// Gets a `&str` value.
    ///
    /// Same mentality as `yaml_rust`, only returns `Some`, if it's a `Yaml::String`.
    fn get_str<'a>(&'a self, path: &str) -> Option<&'a str> {
        self.get(path).and_then(Yaml::as_str)
    }

    /// Gets an `Int` value.
    ///
    /// Same mentality as `yaml_rust`, only returns `Some`, if it's a `Yaml::Int`.
    fn get_int<'a>(&'a self, path: &str) -> Option<i64> {
        self.get(path).and_then(Yaml::as_i64)
    }

    /// Gets a Date in `dd.mm.YYYY` format.
    fn get_dmy(&self, path: &str) -> Option<Date<Utc>> {
        self.get(path)
            .and_then(Yaml::as_str)
            .and_then(|d| parse_dmy_date(d))
    }

    /// Gets a `Bool` value.
    ///
    /// **Careful** this is a bit sweeter then ordinary `YAML1.2`,
    /// this will interpret `"yes"` and `"no"` as booleans, similar to `YAML1.1`.
    /// Actually it will interpret any string but `"yes"` als `false`.
    fn get_bool(&self, path: &str) -> Option<bool> {
        self.get(path)
            .and_then(|y| {
            y
                      .as_bool()
                      // allowing it to be a str: "yes" or "no"
                      .or_else(|| y.as_str()
                           .map( |yes_or_no|
                                 match yes_or_no.to_lowercase().as_ref() {
                                     "yes" => true,
                                     //"no" => false,
                                     _ => false
                                 })
                         )
        })
    }

    /// Gets `Some(Yaml::Hash)` or `None`.
    //pub fn get_hash<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a BTreeMap<Yaml,Yaml>> {
    fn get_hash<'a>(&'a self, path: &str) -> Option<&'a YamlHash> {
        self.get(path).and_then(Yaml::as_hash)
    }

    /// Gets a `Float` value.
    ///
    /// Also takes a `Yaml::I64` and reinterprets it.
    fn get_f64(&self, path: &str) -> Option<f64> {
        self.get(path)
            .and_then(|y| y.as_f64().or_else(|| y.as_i64().map(|y| y as f64)))
    }

    #[deprecated]
    fn field_exists<'a>(&'a self, paths: &[&'a str]) -> ErrorList {
        let mut errors = ErrorList::new();
        for err in paths.iter()
                        .cloned()
                        .filter(|path| self.get(path).is_none())
        {
            errors.push(err);
        }
        errors
    }

}

pub fn search_errors<'a, F: 'a>(src: &'a dyn YamlProvider, paths: &'a [&'a str], check: F) -> impl Iterator<Item = String> + 'a
    where F: Fn(&'a Yaml) -> bool
{
    paths.iter()
        .filter(move |path| src.get(path).map(|y| !check(y)).unwrap_or(true))
            .flat_map(|paths| paths.split('|').nth(0))
            .map(ToString::to_string)
}

// pub fn field_exists(_yaml: &Yaml) -> bool {
//    false 
// }

pub fn field_is_integer(yaml: &Yaml) -> bool {
    yaml.as_i64().is_some()
}

pub fn field_is_string(yaml: &Yaml) -> bool {
    yaml.as_str().is_some()
}

pub fn field_is_dmy(yaml: &Yaml) -> bool {
    yaml.as_str().and_then(parse_dmy_date).is_some()
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

        assert_eq!(no_fallback.get_str("offer.date|offer_date"), Some("07.11.2019"));
        assert_eq!(fallback.get_str("offer.date|offer_date"), Some("08.11.2019"));

        assert_eq!(no_fallback.get_str("offer.date"), Some("07.11.2019"));
        assert_eq!(fallback.get_str("offer_date"), Some("08.11.2019"));

        assert_eq!(no_fallback.get_str("offer_date"), None);
        assert_eq!(fallback.get_str("offer.date"), None);


    }


}