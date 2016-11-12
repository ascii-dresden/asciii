//! Trait based reimplementation of Spec (WIP)
//!

/// Enables access to structured data via a simple path
///
/// A path can be something like `users/clients/23/name`
/// but also  `users.clients.23.name`
pub trait ProvidesData {
    /// You only need to implement this.
    //fn data(&self) -> impl PathAccessible {
    fn data<'a>(&'a self) -> &'a Yaml;

    /// Wrapper around `get_path()`.
    ///
    /// Splits path string
    /// and replaces `Yaml::Null` and `Yaml::BadValue`.
    fn get<'a>(&'a self, key:&str) -> Option<&'a Yaml> {
        // TODO this can be without copying
        let path = key.split(|p| p == '/' || p == '.')
                      .filter(|k|!k.is_empty())
                      .collect::<Vec<&str>>();

        match self.get_path(self.data(), &path) {
            Some(&Yaml::BadValue) |
            Some(&Yaml::Null) => None,
            content => content
        }

    }

    /// Returns content at `path` in the yaml document.
    /// TODO make this generic over the type of data to support more than just `Yaml`.
    fn get_path<'a>(&'a self, data:&'a Yaml, path:&[&str]) -> Option<&'a Yaml>{
        if let Some((&key, remainder)) = path.split_first() {
            match *data {
                // go further into the rabit hole
                Yaml::Hash(ref hash) => {
                    if remainder.is_empty(){
                        hash.get(&Yaml::String(key.to_owned()))
                    } else {
                        hash.get(&Yaml::String(key.to_owned()))
                            .and_then(|c| self.get_path(c, remainder))
                    }
                },
                // interpret component as index
                Yaml::Array(ref vec) => {
                    if let Ok(index) = key.parse::<usize>() {
                        if remainder.is_empty(){
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
    fn get_str<'a>(&'a self, path:&str) -> Option<&'a str> {
        self.get(path).and_then(|y|y.as_str())
    }

    /// Gets an `Int` value.
    ///
    /// Same mentality as `yaml_rust`, only returns `Some`, if it's a `Yaml::Int`.
    fn get_int<'a>(&'a self, path:&str) -> Option<i64> {
        self.get(path).and_then(|y|y.as_i64())
    }

    fn field_exists<'a>(&'a self, paths: &[&'a str]) -> Vec<&'a str> {
        paths.into_iter()
            .map(|i|*i)
            .filter(|path| self.get(path).is_none())
            .collect::<Vec<&'a str>>()
    }

}

/// This is WIP and may replace the sub modules `spec`.
pub trait Validatable {
    fn validate(&self) -> SpecResult;
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    fn errors<'a>(&'a self) -> Option<Vec<&'a str>>{
        self.validate().err()
    }
}

/// Provide the basics every Project should have.
pub trait Project: ProvidesData {
    // TODO reevaluate if these fields really belong here
    fn name(&self) -> Option<&str>;
    fn date(&self) -> Option<Date<UTC>>;
    fn format(&self) -> Option<String>; // Version
    fn canceled(&self) -> bool;
    fn responsible(&self) -> Option<&str>;
}

impl Validatable for Project {
    fn validate(&self) -> SpecResult {
        let mut errors = Vec::new();
        if self.name().is_none(){errors.push("name")}
        if self.date().is_none(){errors.push("date")}
        if self.responsible().is_none(){errors.push("manager")}
        if self.format().is_none(){errors.push("format")}
        //if hours::salary().is_none(){errors.push("salary")}

        if errors.is_empty(){ Ok(()) }
        else { Err(errors) }
    }
}


pub trait Offerable: Project {

    fn appendix(&self) -> Option<i64> {
        self.get_int("offer.appendix")
    }

    /// When was the offer created
    fn date(&self) -> Option<Date<UTC>> {
        yaml::get_dmy(self.data(), "offer.date")
    }


    fn number(&self) -> Option<String> {
        let num = self.appendix().unwrap_or(1);
        Offerable::date(self)
            //.map(|d| d.format("%Y%m%d").to_string())
            .map(|d| d.format("A%Y%m%d").to_string())
            .map(|s| format!("{}-{}", s, num))

        // old spec
        .or_else(|| self.get_str("manumber").map(|s|s.to_string()))
    }

}

impl Validatable for Offerable {
    fn validate(&self) -> SpecResult {
        if Project::canceled(self) {
            return Err(vec!["canceled"]);
        }

        let mut errors = self.field_exists(&["offer/date", "offer/appendix", "manager"]);
        if Offerable::date(self).is_none() {
            errors.push("offer_date_format");
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())

    }
}


