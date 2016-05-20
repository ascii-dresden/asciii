//! Implements the ascii invoicer project file specification.

#![allow(dead_code)]
#![allow(unused_imports)]

use chrono::Datelike;

use util::yaml;
use util::yaml::Yaml;
use currency::Currency;
use super::{Project, ProductResult, ProductError};
use ::storage::Storable;

pub type SpecResult<'a> = Result<(), Vec<&'a str>>;

#[export_macro]
macro_rules! try_some {
    ($expr:expr) => (match $expr {
        Some(val) => val,
        None => return None,
    });
}

/// Fields that are accessible but are not directly found in the file format.
/// This is used to get fields that are computed through an ordinary `get("responsible")`
custom_derive! {
    #[derive(Debug,
             IterVariants(VirtualFields), IterVariantNames(VirtualFieldNames),
             EnumFromStr
             )]
    pub enum VirtualField{
        /// Usually `storage`, or in legacy part of `signature`
        Responsible,
        /// Pretty version of `invoice/number`: "`R042`"
        InvoiceNumber,
        /// Pretty version of `invoice/number` including year: "`R2016-042`"
        InvoiceNumberLong,
        ///Overall Cost Project, including taxes
        Name,
        Final,
        Age,
        Year,
        Caterers,
        ClientFullName,
        Invalid
    }
}

impl<'a> From<&'a str> for VirtualField{
    fn from(s:&'a str) -> VirtualField{
        s.parse::<VirtualField>().unwrap_or(VirtualField::Invalid)
    }
}

impl VirtualField{
    pub fn get(&self,project:&Project) -> Option<String>{
        match *self{
            VirtualField::Responsible       => project::manager(project.yaml()).map(|s|s.to_owned()),
            VirtualField::InvoiceNumber     => invoice::number_str(project.yaml()),
            VirtualField::InvoiceNumberLong => invoice::number_long_str(project.yaml()),
            VirtualField::Name              => project::name(project.yaml()).map(|s|s.to_owned()),
            VirtualField::Final             => project.sum_sold().map(|c|c.to_string()),
            VirtualField::Age               => project.age().map(|a|format!("{} days", a)),
            VirtualField::Year              => project.date().map(|d|d.year().to_string()),

            VirtualField::Caterers       => hours::caterers_string(project.yaml()),
            VirtualField::ClientFullName => client::full_name(project.yaml()),
            VirtualField::Invalid           => None,

            //_ => None
        }
    }
}

//TODO there may be cases where an f64 can't be converted into Currency
pub fn to_currency(f:f64) -> ProductResult<Currency> {
    Currency::new().symbol('€').coin(f).ok_or(ProductError::InvalidPrice)
}

fn field_exists<'a>(yaml:&Yaml, paths:&[&'a str]) -> Vec<&'a str>{
    paths.iter()
        .filter(|path| yaml::get(yaml,path).is_none())
        .cloned()
        .collect::<Vec<&'a str>>()
}

//stage 0
/// Stage 0: the project itself
pub mod project{
    use util::yaml;
    use util::yaml::Yaml;
    use chrono::{Date,UTC};
    use super::hours;

    pub fn name(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "event/name")
            // old spec
            .or_else(|| yaml::get_str(yaml, "event"))
    }

    pub fn date(yaml:&Yaml) -> Option<Date<UTC>>{
        super::date::date(yaml)
    }

    pub fn manager(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "manager")
        // old spec
        .or_else(|| yaml::get_str(yaml, "signature").and_then(|c|c.lines().last()))
    }

    pub fn format(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "format")
    }

    pub fn canceled(yaml:&Yaml) -> bool{
        yaml::get_bool(yaml, "canceled").unwrap_or(false)
    }

    pub fn validate(yaml:&Yaml) -> bool{
        name(yaml).is_some() &&
        date(yaml).is_some() &&
        manager(yaml).is_some() &&
        format(yaml).is_some() &&
        hours::salary(yaml).is_some()
    }
}

/// Everything about the client
pub mod client{
    use util::yaml;
    use util::yaml::Yaml;
    use config::ConfigReader;

    pub fn email(yaml:&Yaml) -> Option<&str> {
        yaml::get_str(yaml, "client/email")
    }

    pub fn address(yaml:&Yaml) -> Option<&str> {
        yaml::get_str(yaml, "client/address")
    }

    pub fn title(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "client/title")
        // old spec
        .or_else(|| yaml::get_str(yaml, "client").and_then(|c|c.lines().nth(0)))
    }

    pub fn first_name(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "client/first_name")
        // old spec
        //.or_else(|| yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(0)))
    }

    pub fn last_name(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "client/last_name")
        // old spec
        .or_else(|| yaml::get_str(yaml, "client").and_then(|c|c.lines().nth(1)))
    }

    pub fn full_name(yaml:&Yaml) -> Option<String>{
        let first = yaml::get_str(yaml, "client/first_name");
        let last  = last_name(yaml);
        first.and(last).and(
            Some(format!("{} {}", first.unwrap_or(""), last.unwrap_or(""))))
    }

    pub fn addressing(yaml:&Yaml, config:&ConfigReader) -> Option<String>{
        if let Some(title) = title(yaml){
            let last_name = last_name(yaml);

            let lang = config.get_str("defaults/lang")
                .expect("Faulty config: defaults/lang does not contain a value");

            let gend_path = "gender_matches/".to_owned() + &title.to_lowercase();
            let gend = config.get_str(&gend_path)
                .expect(&format!("Faulty config: {} does not contain a value",gend_path));

            let addr_path = "lang_addressing/".to_owned() + &lang.to_lowercase() + "/" + gend;
            let addr = config.get_str(&addr_path)
                .expect(&format!("Faulty config: {} does not contain a value",addr_path));

            last_name.and(
                Some(format!("{} {} {}", addr, title, last_name.unwrap_or(""))))
        } else { None }
    }

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        let mut errors = super::field_exists(yaml, &[
                 "client/email",
                 "client/address",
                 "client/last_name",
                 "client/first_name",
        ]);

        if title(yaml).is_none(){       errors.push("client_title");}
        if first_name(yaml).is_none(){  errors.push("client_first_name");}
        if last_name(yaml).is_none(){   errors.push("client_last_name");}

        if !errors.is_empty(){
            return Err(errors);
        }

        Ok(())
    }
}

/// All kinds of dates
pub mod date {
    use chrono::*;
    use regex::Regex;
    use util;
    use util::yaml;
    use util::yaml::Yaml;

    /// When is the first event
    ///
    /// Fallbacks: "created" -> "date"
    pub fn date(yaml:&Yaml) -> Option<Date<UTC>>{
        event(yaml)
        .or_else(||yaml::get_dmy(yaml, "created"))
        .or_else(||yaml::get_dmy(yaml, "date"))
        // probably the dd-dd.mm.yyyy format
        .or_else(||yaml::get_str(yaml, "date").and_then(|s|util::yaml::parse_dmy_date_range(s)))
    }

    /// When was the project payed
    pub fn payed(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "invoice/payed_date")
        // old spec
        .or_else(|| yaml::get_dmy(yaml, "payed_date"))
    }

    /// When were the wages payed
    pub fn wages(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "hours/wages_date")
        // old spec
        .or_else(|| yaml::get_dmy(yaml, "wages_date"))
    }

    /// When was the offer created
    pub fn offer(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "offer/date")
    }

    /// When was the invoice created
    pub fn invoice(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "invoice/date")
        // old spec
        .or_else(|| yaml::get_dmy(yaml, "invoice_date"))
    }

    /// Date of first event
    pub fn event(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "event/dates/0/begin")
    }

    // TODO packed to deep? Clippy says YES, remove this allow!
    pub type DateRange = (Option<Date<UTC>>,Option<Date<UTC>>);
    pub type DateRanges =  Vec< DateRange > ;
    pub fn events(yaml:&Yaml) -> Option<DateRanges> {
        yaml::get(yaml, "event/dates/")
            .and_then(|e|e.as_vec())
            .map(|v| v.iter()
                 .map(|e| (
                         yaml::get_dmy(e, "begin"),
                         yaml::get_dmy(e, "end").or_else(|| yaml::get_dmy(e, "begin"))
                         )
                     )
                 .collect::<Vec<(Option<Date<UTC>>,Option<Date<UTC>>)>>()
                )
    }
}

/// Stage 1: requirements for an offer
pub mod offer{
    use chrono::*;
    use util::yaml;
    use util::yaml::Yaml;

    pub fn number(yaml:&Yaml) -> Option<String> {
        let num = appendix(yaml).unwrap_or(1);
        super::date::offer(yaml)
            .map(|d| d.format("A%Y%m%d").to_string())
            .map(|s| format!("{}-{}", s, num))

        // old spec
        .or_else(|| yaml::get_string(yaml, "manumber"))
    }

    pub fn appendix(yaml:&Yaml) -> Option<i64> {
        yaml::get_int(yaml, "offer/appendix")
    }

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        if super::project::canceled(yaml){
            return Err(vec!["canceled"])
        }

        let mut errors = super::field_exists(yaml, &[
                 "offer/date",
                 "offer/appendix",
                 "manager",
        ]);
        if super::date::offer(yaml).is_none(){
            errors.push("offer_date_format");}

        if !errors.is_empty(){
            return Err(errors);
        }

        Ok(())
    }
}

/// Stage 2: requirements for an invoice
pub mod invoice{
    use util::yaml;
    use util::yaml::Yaml;
    use chrono::Datelike;

    /// plain access to `invoice/number`
    pub fn number(yaml:&Yaml) -> Option<i64> {
        yaml::get_int(yaml, "invoice/number")
        // old spec
        .or_else(|| yaml::get_int(yaml, "rnumber"))
    }

    pub fn number_str(yaml:&Yaml) -> Option<String> {
        number(yaml).map(|n| format!("R{:03}", n))
    }

    pub fn number_long_str(yaml:&Yaml) -> Option<String> {
        let year = try_some!(super::date::invoice(yaml)).year();
        // TODO Length or format should be a setting
        number(yaml).map(|n| format!("R{}-{:03}", year, n))
    }

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        let mut errors = super::field_exists(yaml,&[
                                   "invoice/number",
                                   "invoice/date",
        ]);

        //if super::offer::validate(yaml).is_err() {errors.push("offer")}
        if super::date::invoice(yaml).is_none(){ errors.push("invoice_date_format");}

        if !errors.is_empty(){
            return Err(errors);
        }

        Ok(())
    }
}

/// Stage 3: requirements to archive
pub mod archive{
    use util::yaml;
    use util::yaml::Yaml;

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        let mut errors = Vec::new();
        if super::date::payed(yaml).is_none(){ errors.push("payed_date");}
        //if super::date::wages(yaml).is_none(){ errors.push("wages_date");} // TODO validate WAGES_DATE also
        if !errors.is_empty(){
            return Err(errors);
        }

        Ok(())
    }
}

pub mod hours {
    use currency::Currency;
    use util::yaml;
    use util::yaml::Yaml;
    use super::to_currency;
    use project::{ProductResult, ProductError};

    pub fn salary(yaml:&Yaml) -> Option<Currency>{
        yaml::get_f64(yaml, "hours/salary")
            .and_then(|s|to_currency(s).ok())
    }

    pub fn total(yaml:&Yaml) -> Option<f64> {
        caterers(yaml).map(|vec|vec.iter()
            .map(|&(_,h)| h)
            .fold(0f64,|acc, h| acc + h)
            )
    }

    pub fn caterers_string(yaml:&Yaml) -> Option<String> {
        caterers(yaml).map(|v| v.iter() .map(|t| format!("{}: ({})", t.0,t.1) )
                           .collect::<Vec<String>>().join(", "))
    }

    pub fn caterers(yaml:&Yaml) -> Option<Vec<(String, f64)>> {
        yaml::get_hash(yaml, "hours/caterers")
            .map(|h|h
                 .iter()
                 .map(|(c, h)| (// argh, those could be int or float, grrr
                         c.as_str().unwrap_or("").to_owned(),
                         h.as_f64().or_else(|| // sorry for this
                             h.as_i64().map(|f|f as f64 )
                             ).unwrap_or(0f64)))
                 .collect::<Vec<(String,f64)>>()
                 )
    }
}

pub mod products{
    use std::collections::BTreeMap;
    use currency::Currency;
    use util::yaml;
    use util::yaml::Yaml;
    use project::product::{Product, InvoiceItem, ProductUnit};
    use project::{ProductResult, ProductError};
    use super::to_currency;

    pub fn invoice_items(yaml:&Yaml) -> ProductResult<Vec<InvoiceItem>>{
        let products = try!(yaml::get_hash(yaml, "products")
                            .ok_or(ProductError::UnknownFormat))
            .iter()
            .map(|(desc,values)| InvoiceItem::from_desc_and_value(desc, values))
            .collect::< ProductResult<Vec<InvoiceItem> >>()
            ;

        products
    }

    pub fn all_by_tax(yaml:&Yaml)// -> ProductResult<Vec<InvoiceItem>>
    {
    }

    pub fn sum_offered(items:&[InvoiceItem]) -> Currency{
        items.iter()
             .fold(Currency::from_str("1.00€").unwrap(),
             |acc, item| acc + &item.item.price * item.amount_offered)
    }

    pub fn sum_sold(items:&[InvoiceItem]) -> Currency{
        items.iter()
             .fold(Currency::from_str("1.00€").unwrap(),
             |acc, item| acc + &item.item.price * item.amount_sold)
    }

}

