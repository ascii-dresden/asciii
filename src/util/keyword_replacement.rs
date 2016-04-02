//! Simple templating functionality through keyword replacement.
//! 
//! Replaces `__KEYWORDS__` in Strings.
use regex::{Regex,Captures};
use std::ops::Deref;

/// Simple template style keyword replacement.
///
/// This allows replacing a known set of keywords looking like `__THIS__`.
/// Here it is implemented for `Deref<Target=str>`.
pub trait IsKeyword {
    /// Checks if the whole string is a keyword
    fn is_keyword(&self) -> bool;
    /// Captures keywords from string.
    fn get_keyword(&self) -> Option<String>;
    /// Well, it lists the keywords in a string, duh!
    fn list_keywords(&self) -> Vec<String>;

    /// This one is usefull.
    /// 
    /// Takes a clorsure that replaces keywords.
    /// **Careful**, this replaces either way!
    /// If you get a keywords you don't want to replace,
    /// please place it back where you got it from.
    ///
    /// # Example
    /// ```ignore
    /// .map_keywords|keyword| match data.get(keyword){
    ///     Some(content) => String::from(*content),
    ///     None => format!("__{}__", keyword)
    /// }
    /// ```
    ///
    fn map_keywords<F>(&self, closure: F) -> String where F:Fn(&str) -> String;// -> Option<String>;
}

static REGEX: &'static str = r"__([0-9A-Z-]*)__*";

/// Allows very simplistic `__KEYWORD__` replacement.
impl<U:Deref<Target=str>> IsKeyword for U {

    /// Checks if the whole string is a keyword
    fn is_keyword(&self) -> bool{
        Regex::new(REGEX).expect("broken regex").is_match(self)
    }

    /// Captures keywords from string.
    fn get_keyword(&self) -> Option<String> {
        Regex::new(REGEX).expect("broken regex")
            .captures(&self)
            .and_then(|caps| caps.at(1).map(|c| c.to_owned()))
    }

    /// Well, it lists the keywords in a string, duh!
    fn list_keywords(&self) -> Vec<String>{
        Regex::new(REGEX).expect("broken regex")
            .captures_iter(&self)
            .map(|c|c.at(1).unwrap().to_owned())
            .collect()
    }

    /// This one is usefull.
    /// 
    /// Takes a clorsure that replaces keywords.
    /// **Careful**, this replaces either way!
    /// If you get a keywords you don't want to replace,
    /// please place it back where you got it from.
    ///
    /// # Example
    /// ```ignore
    /// .map_keywords|keyword| match data.get(keyword){
    ///     Some(content) => String::from(*content),
    ///     None => format!("__{}__", keyword)
    /// }
    /// ```
    ///
    fn map_keywords<F>(&self, closure: F) -> String
        where F:Fn(&str) -> String{
        Regex::new(REGEX).expect("broken regex")
            .replace_all(&self, |caps:&Captures| {
                closure(caps.at(1).unwrap())
            })
    }
}
