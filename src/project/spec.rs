#![allow(dead_code)]

pub type SpecResult<'a> = Result<(), Vec<&'a str>>;

pub mod validate{
    use util::yaml;
    use util::yaml::Yaml;

    pub fn existence<'a>(yaml:&Yaml, mut paths:Vec<&'a str>) -> Vec<&'a str>{
        paths.drain(..)
            .filter(|path| yaml::get(yaml,path).is_none())
            .collect::<Vec<&'a str>>()
    }

    pub fn invoice<'a>(yaml:&Yaml) -> Vec<&'a str>{
        let mut errors = existence(&yaml,vec![
                                   "invoice/number",
                                   "invoice/date",
                                   "invoice/payed_date",
        ]);
        if super::date::invoice(&yaml).is_none(){
            errors.push("invoice_date_format");}
        errors
    }
}

//stage 0
pub mod project{
    use util::yaml;
    use util::yaml::Yaml;
    use chrono::{Date,UTC};

    pub fn name(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "event/name")
            // old spec
            .or( yaml::get_str(yaml, "event"))
    }

    pub fn date(yaml:&Yaml) -> Option<Date<UTC>>{
        super::date::date(&yaml)
    }

    pub fn manager(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "manager")
        // old spec
        .or( yaml::get_str(&yaml, "signature").and_then(|c|c.lines().last()))
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

    pub fn validate(yaml:&Yaml) -> bool{
        name(&yaml).is_some() &&
        date(&yaml).is_some() &&
        manager(&yaml).is_some() &&
        format(&yaml).is_some() &&
        salary(&yaml).is_some()
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
        .or( yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(0)))
    }

    pub fn first_name(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(&yaml, "client/first_name")
        // old spec
        //.or( yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(0)))
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

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        let mut errors = super::validate::existence(&yaml, vec![
                 "client/email",
                 "client/address",
                 "client/last_name",
                 "client/first_name",
        ]);

        if title(&yaml).is_none(){       errors.push("client_title");}
        if first_name(&yaml).is_none(){  errors.push("client_first_name");}
        if last_name(&yaml).is_none(){   errors.push("client_last_name");}

        if !errors.is_empty(){
            return Err(errors);
        }

        Ok(())
    }
}

pub mod date {
    use chrono::*;
    use regex::Regex;
    use util;
    use util::yaml;
    use util::yaml::Yaml;

    pub fn date(yaml:&Yaml) -> Option<Date<UTC>>{
        yaml::get_dmy(&yaml, "event/dates/0/begin")
        .or(yaml::get_dmy(&yaml, "created"))
        .or(yaml::get_dmy(&yaml, "date"))
        // probably the dd-dd.mm.yyyy format
        .or(yaml::get_str(&yaml, "date").and_then(|s|util::yaml::parse_fwd_date_range(s)))
    }

    pub fn payed(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "invoice/payed_date")
        // old spec
        .or( yaml::get_dmy(yaml, "payed_date"))
    }

    pub fn wages(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "hours/wages_date")
        // old spec
        .or( yaml::get_dmy(yaml, "wages_date"))
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

//stage 1
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

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        // TODO validate products
        let mut errors = super::validate::existence(&yaml, vec![
                 "offer/date",
                 "offer/appendix",
        ]);
        if super::date::offer(&yaml).is_none(){
            errors.push("offer_date_format");}

        if !errors.is_empty(){
            return Err(errors);
        }

        Ok(())
    }
}

//stage 2
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

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        let mut errors = super::validate::existence(&yaml,vec![
                                   "invoice/number",
                                   "invoice/date",
        ]);

        if super::offer::validate(&yaml).is_err() {errors.push("offer")}
        if super::date::invoice(&yaml).is_none(){ errors.push("invoice_date_format");}

        if !errors.is_empty(){
            return Err(errors);
        }

        Ok(())
    }
}

//stage 3
pub mod archive{
    use util::yaml;
    use util::yaml::Yaml;

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        let mut errors = Vec::new();
        if super::date::payed(&yaml).is_none(){ errors.push("payed_date");}
        if super::date::wages(&yaml).is_none(){ errors.push("wages_date");}
        if !errors.is_empty(){
            return Err(errors);
        }

        Ok(())
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

pub mod products{
    use util::yaml;
    use util::yaml::Yaml;
    use std::collections::BTreeMap;
    use project::product::{Product, InvoiceItem, ProductUnit};

    pub fn all(yaml:&Yaml) -> Option<Vec<InvoiceItem>>{
        yaml::get_hash(yaml, "products")
            .map(|hmap| hmap.iter()
                 .map(|(desc,values)|{
                     let product =
                         match *desc {
                             yaml::Yaml::Hash(_) => {
                                 Product{
                                     name:  yaml::get_str(desc, "name").unwrap_or("unnamed"),
                                     unit:  yaml::get_str(desc, "unit"),
                                     price: yaml::get_f64(desc, "price").unwrap_or(0.0),
                                     tax:   yaml::get_f64(desc, "tax").unwrap_or(0.19),
                                 }
                             },
                             yaml::Yaml::String(ref name) => {
                                 Product{
                                     name:  name,
                                     unit:  yaml::get_str(values, "unit"),
                                     price: yaml::get_f64(values, "price").unwrap_or(0.0),
                                     tax:   yaml::get_f64(values, "tax").unwrap_or(0.19),
                                 }
                             }
                             _ => unreachable!()
                         };
                     InvoiceItem {
                         amount_offered: yaml::get_f64(values, "amount").unwrap_or(0f64),
                         amount_sold: yaml::get_f64(values, "").unwrap_or(0f64),
                         item: product
                     }

                 })
                 .collect::<Vec<InvoiceItem>>()
                )
    }
}

#[cfg(test)]
mod tests{
    use util::yaml;
    use util::yaml::YamlError;

static CLIENT_TEST_DOC:&'static str =
r#"
client:
  title:      Herr # Frau, Professor, Professorin
  first_name: Graf
  last_name:  Zahl

  email: this.man@example.com
  address: |
    Graf Zahl
    Nummernhöllenstraße 666
    01234 Countilvania
"#;

static OFFER_TEST_DOC:&'static str =
r#"
offer:
  date: 07.11.2014
  appendix: 1
"#;

static INVOICE_TEST_DOC:&'static str =
r#"
invoice:
  number: 41
  date: 06.12.2014
  payed_date: 08.12.2014
"#;

    #[test]
    fn validate_stage1(){
        let doc = yaml::parse(CLIENT_TEST_DOC).unwrap();
        assert!(super::client::validate(&doc).is_ok());
    }

    #[test]
    fn validate_stage2(){
        let doc = yaml::parse(OFFER_TEST_DOC).unwrap();
        let errors = super::offer::validate(&doc);
        println!("{:#?}", errors);
        assert!(errors.is_ok());
    }

    #[test]
    fn validate_stage3(){
        let doc = yaml::parse(INVOICE_TEST_DOC).unwrap();
        let errors = super::validate::invoice(&doc);
        println!("{:#?}", errors);
        assert!(errors.is_empty());
    }

    //#[test]
    //fn validate_stage4(){
    //    let doc = yaml::parse(INVOICE_TEST_DOC).unwrap();
    //    assert!(super::payed::validate(&doc));
    //}

    //#[test]
    //fn validate_stage5(){
    //    let doc = yaml::parse(CLIENT_TEST_DOC).unwrap();
    //    assert!(super::validate::wages(&doc));
    //}

}
