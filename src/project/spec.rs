#![allow(dead_code)]

pub mod project{
    use util::yaml;
    use util::yaml::Yaml;
    use chrono::{Date,UTC};

    pub fn name(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "event/name")
            .or( yaml::get_str(yaml, "event"))
    }

    pub fn date(yaml:&Yaml) -> Option<Date<UTC>>{
        yaml::get_dmy(&yaml, "event/date")
        .or(yaml::get_dmy(&yaml, "created"))
    }

    pub fn manager(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "manager")
    }

    pub fn format(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "format")
    }

    pub fn canceled(yaml:&Yaml) -> bool{
        yaml::get_bool(yaml, "canceled").unwrap_or(false)
    }

    pub fn salary(yaml:&Yaml) -> Option<f64>{
        yaml::get_f64(yaml, "hours/salary")
    }
}

pub mod client{
    use util::yaml;
    use util::yaml::Yaml;
    use config::ConfigReader;

    pub fn email(yaml:&Yaml) -> Option<&str> {
        yaml::get_str(&yaml, "client/email")
    }

    pub fn address(yaml:&Yaml) -> Option<&str> {
        yaml::get_str(&yaml, "client/address")
    }

    pub fn title(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(&yaml, "client/title")
        // old spec
        .or( yaml::get_str(&yaml, "client").and_then(|c|c.lines().next()))
    }

    pub fn last_name(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(&yaml, "client/last_name")
        // old spec
        .or( yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(1)))
    }

    pub fn full_name(yaml:&Yaml) -> Option<String>{
        let first = yaml::get_str(&yaml, "client/first_name");
        let last  = last_name(&yaml);
        first.and(last).and(
            Some(format!("{} {}", first.unwrap_or(""), last.unwrap_or(""))))
    }

    pub fn addressing(yaml:&Yaml, config:&ConfigReader) -> Option<String>{
        return if let Some(title) = title(&yaml){
            let last_name = last_name(&yaml);

            let lang = config.get_str("defaults/lang");

            let gend = config.get_str(&(
                    "gender_matches/".to_owned() + &title.to_lowercase()));

            let addr = config.get_str(&(
                    "lang_addressing/".to_owned() + &lang.to_lowercase() + "/"
                    + &gend.to_lowercase()));

            last_name.and(
                Some(format!("{} {} {}", addr, title, last_name.unwrap_or(""))))
        } else { None }
    }
}

pub mod date {
    use chrono::*;
    use regex::Regex;
    use util;
    use util::yaml;
    use util::yaml::Yaml;

    pub fn date(yaml:&Yaml) -> Option<Date<UTC>>{
        yaml::get_dmy(&yaml, "event/date")
        .or(yaml::get_dmy(&yaml, "created"))
        .or(yaml::get_dmy(&yaml, "date"))
    }

    pub fn payed(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "invoice/payed_date")
        // old spec
        .or( yaml::get_dmy(yaml, "payed_date"))
    }

    pub fn offer(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "offer/date")
    }

    pub fn invoice(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "invoice/date")
        // old spec
        .or( yaml::get_dmy(yaml, "invoice_date"))
    }

    pub fn event(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "event/dates/0/begin")
    }

    // TODO packed to deep?
    pub fn events(yaml:&Yaml) -> Option< Vec< (Option<Date<UTC>>,Option<Date<UTC>>) > > {
        yaml::get(yaml, "event/dates/")
            .and_then(|e|e.as_vec())
            .map(|v| v.iter()
                 .map(|e| (
                         yaml::get_dmy(e, "begin"),
                         yaml::get_dmy(e, "end").or( yaml::get_dmy(e, "begin"))
                         )
                     )
                 .collect::<Vec<(Option<Date<UTC>>,Option<Date<UTC>>)>>()
                )
    }
}

pub mod offer{
    use chrono::*;
    use util::yaml;
    use util::yaml::Yaml;

    pub fn number(yaml:&Yaml) -> Option<String> {
        let num = appendix(&yaml).unwrap_or(1);
        super::date::offer(&yaml)
            .map(|d| d.format("A%Y%m%d").to_string())
            .map(|s| format!("{}-{}", s, num))

        // old spec
        .or( yaml::get_string(&yaml, "manumber"))
    }

    pub fn appendix(yaml:&Yaml) -> Option<i64> {
        yaml::get_int(&yaml, "offer/appendix")
    }
}

pub mod invoice{
    use util::yaml;
    use util::yaml::Yaml;

    pub fn number(yaml:&Yaml) -> Option<i64> {
        yaml::get_int(&yaml, "invoice/number")
        // old spec
        .or( yaml::get_int(&yaml, "rnumber"))
    }

    pub fn number_str(yaml:&Yaml) -> Option<String> {
        number(&yaml).map(|n| format!("R{:03}", n))
    }
}

pub mod hours {
    use util::yaml;
    use util::yaml::Yaml;


    pub fn total(yaml:&Yaml) -> Option<f64> {
        caterers(&yaml).map(|vec|vec.iter()
            .map(|&(_,h)| h)
            .fold(0f64,|acc, h| acc + h)
            )
    }

    pub fn caterers(yaml:&Yaml) -> Option<Vec<(String, f64)>> {
        yaml::get_hash(&yaml, "hours/caterers")
            .map(|h|h
                 .iter()
                 .map(|(c, h)| (// argh, those could be int or float, grrr
                         c.as_str().unwrap_or("").to_owned(),
                         h.as_f64().or( // sorry for this
                             h.as_i64().map(|f|f as f64 )
                             ).unwrap_or(0f64)))
                 .collect::<Vec<(String,f64)>>()
                 )
    }
}

