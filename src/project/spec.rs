#![allow(dead_code)]
#![allow(unused_imports)]

use chrono::Datelike;

use util::yaml;
use util::yaml::Yaml;
use currency::Currency;
use super::Project;
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
            VirtualField::Year              => project.date().map(|d|d.year().to_string()),

            VirtualField::Caterers       => hours::caterers_string(project.yaml()),
            VirtualField::ClientFullName => client::full_name(project.yaml()),
            VirtualField::Invalid           => None,

            //_ => None
        }
    }
}

//TODO there may be cases where an f64 can't be converted into Currency
fn to_currency(f:f64) -> Currency {
    Currency::from_string(&format!("{:.*}", 2, f))
        .map_or( Currency(Some('€'), 0),
        |mut cur| {
            cur.0 = Some('€');
            cur
        }
        )
}

fn field_exists<'a>(yaml:&Yaml, paths:&[&'a str]) -> Vec<&'a str>{
    paths.iter()
        .filter(|path| yaml::get(yaml,path).is_none())
        .cloned()
        .collect::<Vec<&'a str>>()
}

//stage 0
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
        super::date::date(&yaml)
    }

    pub fn manager(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "manager")
        // old spec
        .or_else(|| yaml::get_str(&yaml, "signature").and_then(|c|c.lines().last()))
    }

    pub fn format(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(yaml, "format")
    }

    pub fn canceled(yaml:&Yaml) -> bool{
        yaml::get_bool(yaml, "canceled").unwrap_or(false)
    }

    pub fn validate(yaml:&Yaml) -> bool{
        name(&yaml).is_some() &&
        date(&yaml).is_some() &&
        manager(&yaml).is_some() &&
        format(&yaml).is_some() &&
        hours::salary(&yaml).is_some()
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
        .or_else(|| yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(0)))
    }

    pub fn first_name(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(&yaml, "client/first_name")
        // old spec
        //.or_else(|| yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(0)))
    }

    pub fn last_name(yaml:&Yaml) -> Option<&str>{
        yaml::get_str(&yaml, "client/last_name")
        // old spec
        .or_else(|| yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(1)))
    }

    pub fn full_name(yaml:&Yaml) -> Option<String>{
        let first = yaml::get_str(&yaml, "client/first_name");
        let last  = last_name(&yaml);
        first.and(last).and(
            Some(format!("{} {}", first.unwrap_or(""), last.unwrap_or(""))))
    }

    pub fn addressing(yaml:&Yaml, config:&ConfigReader) -> Option<String>{
        if let Some(title) = title(&yaml){
            let last_name = last_name(&yaml);

            let lang = config.get_str("defaults/lang")
                .expect("Faulty config: defaults/lang does not contain a value");

            let gend = config.get_str(&(
                    "gender_matches/".to_owned() + &title.to_lowercase()));

            let addr_path = "lang_addressing/".to_owned() + &lang.to_lowercase() + "/";
            let addr = config.get_str(&addr_path)
                .expect(&format!("Faulty config: {} does not contain a value",addr_path));

            last_name.and(
                Some(format!("{} {} {}", addr, title, last_name.unwrap_or(""))))
        } else { None }
    }

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        let mut errors = super::field_exists(&yaml, &[
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
        .or_else(||yaml::get_dmy(&yaml, "created"))
        .or_else(||yaml::get_dmy(&yaml, "date"))
        // probably the dd-dd.mm.yyyy format
        .or_else(||yaml::get_str(&yaml, "date").and_then(|s|util::yaml::parse_dmy_date_range(s)))
    }

    pub fn payed(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "invoice/payed_date")
        // old spec
        .or_else(|| yaml::get_dmy(yaml, "payed_date"))
    }

    pub fn wages(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "hours/wages_date")
        // old spec
        .or_else(|| yaml::get_dmy(yaml, "wages_date"))
    }

    pub fn offer(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "offer/date")
    }

    pub fn invoice(yaml:&Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "invoice/date")
        // old spec
        .or_else(|| yaml::get_dmy(yaml, "invoice_date"))
    }

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
        .or_else(|| yaml::get_string(&yaml, "manumber"))
    }

    pub fn appendix(yaml:&Yaml) -> Option<i64> {
        yaml::get_int(&yaml, "offer/appendix")
    }

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        if super::project::canceled(yaml){
            return Err(vec!["canceled"])
        }

        let mut errors = super::field_exists(&yaml, &[
                 "offer/date",
                 "offer/appendix",
                 "manager",
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
    use chrono::Datelike;

    /// plain access to `invoice/number`
    pub fn number(yaml:&Yaml) -> Option<i64> {
        yaml::get_int(&yaml, "invoice/number")
        // old spec
        .or_else(|| yaml::get_int(&yaml, "rnumber"))
    }

    pub fn number_str(yaml:&Yaml) -> Option<String> {
        number(&yaml).map(|n| format!("R{:03}", n))
    }

    pub fn number_long_str(yaml:&Yaml) -> Option<String> {
        let year = try_some!(super::date::invoice(&yaml)).year();
        // TODO Length or format should be a setting
        number(&yaml).map(|n| format!("R{}-{:03}", year, n))
    }

    pub fn validate(yaml:&Yaml) -> super::SpecResult {
        let mut errors = super::field_exists(&yaml,&[
                                   "invoice/number",
                                   "invoice/date",
        ]);

        //if super::offer::validate(&yaml).is_err() {errors.push("offer")}
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
        //if super::date::wages(&yaml).is_none(){ errors.push("wages_date");} // TODO validate WAGES_DATE also
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

    pub fn salary(yaml:&Yaml) -> Option<Currency>{
        yaml::get_f64(yaml, "hours/salary").map(to_currency)
    }

    pub fn total(yaml:&Yaml) -> Option<f64> {
        caterers(&yaml).map(|vec|vec.iter()
            .map(|&(_,h)| h)
            .fold(0f64,|acc, h| acc + h)
            )
    }

    pub fn caterers_string(yaml:&Yaml) -> Option<String> {
        caterers(yaml).map(|v| v.iter() .map(|t| format!("{}: ({})", t.0,t.1) )
                           .collect::<Vec<String>>().join(", "))
    }

    pub fn caterers(yaml:&Yaml) -> Option<Vec<(String, f64)>> {
        yaml::get_hash(&yaml, "hours/caterers")
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
    use super::to_currency;

    #[derive(Debug, PartialEq, Eq)]
    pub enum ProductError{
        //DuplicateProduct // not an error
        AmbiguousAmounts(String),
        MissingAmount(String),
        TooMuchReturned(String),
        UnknownFormat
    }

    pub type ProductResult<T> = Result<T, ProductError>;

    fn build_product<'a>(desc: &'a Yaml, values: &'a Yaml) -> ProductResult<Product<'a>> {
        let default_tax = 0.19;
        match *desc {
            yaml::Yaml::Hash(_) => {
                Ok(Product{
                    name:  yaml::get_str(desc, "name").unwrap_or("unnamed"),
                    unit:  yaml::get_str(desc, "unit"),
                    price: yaml::get_f64(desc, "price")
                        .map(to_currency).unwrap(),
                    tax:   yaml::get_f64(desc, "tax").unwrap_or(default_tax),
                })
            },
            yaml::Yaml::String(ref name) => {
                Ok(Product{
                    name:  name,
                    unit:  yaml::get_str(values, "unit"),
                    price: yaml::get_f64(values, "price")
                        .map(to_currency).unwrap(),
                    tax:   yaml::get_f64(values, "tax").unwrap_or(default_tax),
                })
            }
            _ => Err(ProductError::UnknownFormat)
        }
    }

    fn build_invoice_item<'a>(product:Product<'a>, values:&'a Yaml) -> ProductResult<InvoiceItem<'a>> {
        let offered = try!(yaml::get_f64(values, "amount").ok_or(ProductError::MissingAmount(product.name.to_owned())));
        let sold = yaml::get_f64(values, "sold");
        let sold =
            if let Some(returned) = yaml::get_f64(values, "returned"){
                // if "returned", there must be no "sold"
                if sold.is_some() {return Err(ProductError::AmbiguousAmounts(product.name.to_owned()));}
                if returned > offered {return Err(ProductError::TooMuchReturned(product.name.to_owned()));}
                offered - returned
            } else if let Some(sold) = sold {
                sold
            } else {
                offered
            };

        Ok(InvoiceItem {
            amount_offered: offered,
            amount_sold: sold,
            item: product
        })
    }

    #[allow(unknown_lints)] // this is just for clippy
    #[allow(option_map_unwrap_or_else)]
    pub fn all0(yaml:&Yaml) -> Vec<ProductResult<InvoiceItem>>{
        yaml::get_hash(yaml, "products")
            .map(|hmap| hmap.iter()
                 .map(|(desc,values)|{
                     build_product(&desc, &values)
                         .and_then(|product|
                              build_invoice_item(product, values))
                 }
                 ).collect::<Vec< ProductResult<InvoiceItem> >>())
            .unwrap_or_else(Vec::new)
    }

    pub fn all(yaml:&Yaml) -> ProductResult<Vec<InvoiceItem>>{
        let products = yaml::get_hash(yaml, "products").ok_or(ProductError::UnknownFormat).map(|products|
        products.iter()
                 .map(|(desc,values)|
                     build_product(&desc, &values)
                     .and_then(|product| build_invoice_item(product, &values))
                 ).collect::<Vec< ProductResult<InvoiceItem> >>());
        let mut list = Vec::new();

        // Why can't I do this with mapping?
        for product in try!(products){
            match product{
                Ok(item) => list.push(item),
                Err(err) => return Err(err)
            }
        }
        Ok(list)
    }

    pub fn sum_offered(items:&[InvoiceItem]) -> Currency{
        items.iter()
             .fold(Currency(Some('€'), 0),
             |acc, item| acc + item.item.price * item.amount_offered)
    }

    pub fn sum_sold(items:&[InvoiceItem]) -> Currency{
        items.iter()
             .fold(Currency(Some('€'), 0),
             |acc, item| acc + item.item.price * item.amount_sold)
    }

}

#[cfg(test)]
mod tests{
    use util::yaml;
    use util::yaml::YamlError;
    use currency::Currency;

    use super::products::ProductResult;
    use super::products::ProductError;

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
manager: somebody
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
        let errors = super::invoice::validate(&doc);
        println!("{:#?}", errors);
        assert!(errors.is_ok());
    }

static PRODUCT_TEST_DOC_VALID:&'static str =
r#"
---
cataloge:
  product: &coffee    { name: Kaffee, price: 2.5, unit: 1l  }
  product: &tea       { name: Tee,    price: 2.5, unit: 1l  }
  product: &water     { name: Wasser, price: 2.5, unit: 1l  }

products:
  *coffee: { amount: 5 }
  *tea: { amount: 6, sold: 2 }
  *water:
    amount: 6
    returned: 4
...
"#;

static PRODUCT_TEST_DOC_INVALID1:&'static str =
r#"
--- # sold and returend
cataloge:
  product: &tea       { name: Tee,    price: 2.5, unit: 1l  }
products:
  *tea: { amount: 6, sold: 2, returned: 4 }
...
"#;

static PRODUCT_TEST_DOC_INVALID2:&'static str =
r#"
--- # returning too much
cataloge:
  product: &tea { name: Tee, price: 2.5, unit: 1l }
products:
  *tea: { amount: 6, returned: 7 }
...
"#;

static PRODUCT_TEST_DOC_INVALID3:&'static str =
r#"
--- # returning too much
cataloge:
  product: &tea { name: Tee, price: 2.5, unit: 1l }
products:
  *tea: { returned: 7 }
...
"#;

#[test]
fn validate_products(){
    let doc = yaml::parse(PRODUCT_TEST_DOC_VALID).unwrap();

    println!("{:#?}",doc);
    let products = super::products::all(&doc).unwrap();
    println!("Products {:#?}",products);
    assert_eq!(products[0].item.name, "Kaffee");
    assert_eq!(products[0].amount_offered, 5f64);
    assert_eq!(products[0].amount_sold, 5f64);
    assert_eq!(products[0].cost_before_tax(), Currency(Some('€'), 1250));
    assert_eq!(products[0].cost_after_tax(), Currency(Some('€'), 1488));

    assert_eq!(products[1].item.name, "Tee");
    assert_eq!(products[1].amount_offered, 6f64);
    assert_eq!(products[1].amount_sold, 2f64);

    assert_eq!(products[2].item.name, "Wasser");
    assert_eq!(products[2].amount_offered, 6f64);
    assert_eq!(products[2].amount_sold, 2f64);
}

#[test]
fn validate_invalid_products(){
    let invalid1= yaml::parse(PRODUCT_TEST_DOC_INVALID1).unwrap();
    let invalid2= yaml::parse(PRODUCT_TEST_DOC_INVALID2).unwrap();
    let invalid3= yaml::parse(PRODUCT_TEST_DOC_INVALID3).unwrap();
    assert_eq!( super::products::all(&invalid1).unwrap_err(), ProductError::AmbiguousAmounts("Tee".to_owned()));
    assert_eq!( super::products::all(&invalid2).unwrap_err(), ProductError::TooMuchReturned("Tee".to_owned()));
    assert_eq!( super::products::all(&invalid3).unwrap_err(), ProductError::MissingAmount("Tee".to_owned()));
}

    //#[test]
    //fn validate_stage5(){
    //    let doc = yaml::parse(CLIENT_TEST_DOC).unwrap();
    //    assert!(super::validate::wages(&doc));
    //}

}
