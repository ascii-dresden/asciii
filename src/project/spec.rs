//! Implements the ascii invoicer project file specification.
//!
//! This does all of the heavy lifting.
//! The implementation is separated into sub-modules which take care of separate objectives.
//! Most of the functions in these modules take the `yaml` data directly as reference.
//! Each module contains a `validate()` function which ought to be kept up to date.

use chrono::Datelike;
use currency::Currency;

use util::yaml;
use util::yaml::Yaml;
use util::currency_to_string;
use storage::Storable;
use super::Project;

pub type SpecResult<'a> = Result<(), Vec<&'a str>>;

/// Fields that are accessible but are not directly found in the file format.
/// This is used to get fields that are computed through an ordinary `get("responsible")`
custom_derive! {
    #[derive(Debug,
             IterVariants(VirtualFields), IterVariantNames(VirtualFieldNames),
             EnumFromStr
             )]
    /// `Project::get()` allows accessing fields within the raw `yaml` data structure.
    /// Virtual fields are fields that are not present in the document but computed.
    ///
    /// `VirtualFields` is an automatically generated type that allows iterating of the variants of
    /// this Enum.
    pub enum VirtualField{
        /// Usually `storage`, or in legacy part of `signature`
        Responsible,
        /// Pretty version of `invoice/number`: "`R042`"
        OfferNumber,
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

impl<'a> From<&'a str> for VirtualField {
    fn from(s: &'a str) -> VirtualField {
        s.parse::<VirtualField>().unwrap_or(VirtualField::Invalid)
    }
}

impl VirtualField {
    pub fn get(&self, project: &Project) -> Option<String> {
        match *self {
            VirtualField::Responsible => project::manager(project.yaml()).map(|s| s.to_owned()),
            VirtualField::OfferNumber => offer::number(project.yaml()),
            VirtualField::InvoiceNumber => invoice::number_str(project.yaml()),
            VirtualField::InvoiceNumberLong => invoice::number_long_str(project.yaml()),
            VirtualField::Name => project::name(project.yaml()).map(|s| s.to_owned()),
            VirtualField::Final => project.sum_sold().map(|c| currency_to_string(&c)).ok(),
            VirtualField::Age => project.age().map(|a| format!("{} days", a)),
            VirtualField::Year => project.date().map(|d| d.year().to_string()),

            VirtualField::Caterers => hours::caterers_string(project.yaml()),
            VirtualField::ClientFullName => client::full_name(project.yaml()),
            VirtualField::Invalid => None,

            // _ => None
        }
    }
}

// TODO there may be cases where an f64 can't be converted into Currency
pub fn to_currency(f: f64) -> Currency {
    Currency(::CONFIG.get_char("currency"), (f * 1000.0) as i64) / 10
}

fn field_exists<'a>(yaml: &Yaml, paths: &[&'a str]) -> Vec<&'a str> {
    paths.into_iter()
        .map(|i|*i)
        .filter(|path| yaml::get(yaml, path).is_none())
        .collect::<Vec<&'a str>>()
}

// stage 0
/// Stage 0: the project itself
pub mod project {
    use util::yaml;
    use util::yaml::Yaml;
    use chrono::{Date, UTC};
    use super::hours;

    ///Returns the content of `/event/name` or...
    ///
    ///...that of the older formats `/event`
    pub fn name(yaml: &Yaml) -> Option<&str> {
        yaml::get_str(yaml, "event/name")
            // old spec
            .or_else(|| yaml::get_str(yaml, "event"))
    }

    /// Wrapper for `super::date::date()`
    pub fn date(yaml: &Yaml) -> Option<Date<UTC>> {
        super::date::date(yaml)
    }

    ///Returns the content of `/manager` or...
    ///
    ///...that of the older formats `/signature`
    pub fn manager(yaml: &Yaml) -> Option<&str> {
        yaml::get_str(yaml, "manager")
        // old spec
        .or_else(|| yaml::get_str(yaml, "signature").and_then(|c|c.lines().last()))
    }

    ///Returns the content of `/format`
    pub fn format(yaml: &Yaml) -> Option<&str> {
        yaml::get_str(yaml, "format")
    }

    ///Returns the content of `/canceled`
    pub fn canceled(yaml: &Yaml) -> bool {
        yaml::get_bool(yaml, "canceled").unwrap_or(false)
    }

    /// Validates if all of the functions in this module return `Some(_)`
    pub fn validate(yaml: &Yaml) -> super::SpecResult {
        let mut errors = Vec::new();
        if name(yaml).is_none(){errors.push("name")}
        if date(yaml).is_none(){errors.push("date")}
        if manager(yaml).is_none(){errors.push("manager")}
        if format(yaml).is_none(){errors.push("format")}
        if hours::salary(yaml).is_none(){errors.push("salary")}

        if errors.is_empty(){ Ok(()) }
        else { Err(errors) }
    }
}

/// Everything about the client
pub mod client {
    use util::yaml;
    use util::yaml::Yaml;

    ///Returns the content of `/client/email`
    pub fn email(yaml: &Yaml) -> Option<&str> {
        yaml::get_str(yaml, "client/email")
    }

    ///Returns the content of `/client/address`
    pub fn address(yaml: &Yaml) -> Option<&str> {
        yaml::get_str(yaml, "client/address")
    }

    ///Returns the content of `/client/title`
    pub fn title(yaml: &Yaml) -> Option<&str> {
        yaml::get_str(yaml, "client/title")
        // old spec
        .or_else(|| yaml::get_str(yaml, "client").and_then(|c|c.lines().nth(0)))
    }

    ///Returns the content of `/client/first_name`
    pub fn first_name(yaml: &Yaml) -> Option<&str> {
        yaml::get_str(yaml, "client/first_name")
        // old spec
        // .or_else(|| yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(0)))
    }

    ///Returns the content of `/client/last_name`
    pub fn last_name(yaml: &Yaml) -> Option<&str> {
        yaml::get_str(yaml, "client/last_name")
        // old spec
        .or_else(|| yaml::get_str(yaml, "client").and_then(|c|c.lines().nth(1)))
    }

    /// Combines `first_name` and `last_name`.
    pub fn full_name(yaml: &Yaml) -> Option<String> {
        let first = first_name(yaml);
        let last = last_name(yaml);
        first.and(last)
             .and(Some(format!("{} {}",
                               first.unwrap_or(""),
                               last.unwrap_or(""))))
    }

    /// Produces a standard salutation field.
    pub fn addressing(yaml: &Yaml) -> Option<String> {
        if let Some(title) = title(yaml).and_then(|title| title.split_whitespace().nth(0))
        // only the first word
        {
            let last_name = last_name(yaml);


            let lang = ::CONFIG.get_str("defaults/lang")
                .expect("Faulty config: defaults/lang does not contain a value");

            let gend_path = "gender_matches/".to_owned() + &title.to_lowercase();
            let gend = ::CONFIG.get_str(&gend_path)
                .expect(&format!("Faulty config: {} does not contain a value", gend_path));

            let addr_path = "lang_addressing/".to_owned() + &lang.to_lowercase() + "/" + gend;
            let addr = ::CONFIG.get_str(&addr_path)
                .expect(&format!("Faulty config: {} does not contain a value", addr_path));

            last_name.and(Some(format!("{} {} {}", addr, title, last_name.unwrap_or(""))))
        } else {
            None
        }
    }

    /// Validates the output of each of this modules functions.
    pub fn validate(yaml: &Yaml) -> super::SpecResult {
        let mut errors = super::field_exists(yaml,
                                             &[
                                             //"client/email", // TODO make this a requirement
                                             "client/address",
                                             "client/last_name",
                                             "client/first_name"
                                             ]);


        if let Some(title) = title(yaml) {
            if title.split_whitespace().count() > 1 {
                errors.push("client_title");
            }
        } else {
            errors.push("client_title");
        }

        if addressing(yaml).is_none() {
            errors.push("client_addressing");
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

/// All kinds of dates
pub mod date {
    use chrono::*;
    use util;
    use util::yaml;
    use util::yaml::Yaml;

    /// When is the first event
    ///
    /// Fallbacks: "created" -> "date"
    pub fn date(yaml: &Yaml) -> Option<Date<UTC>> {
        event(yaml)
        .or_else(||yaml::get_dmy(yaml, "created"))
        .or_else(||yaml::get_dmy(yaml, "date"))
        // probably the dd-dd.mm.yyyy format
        .or_else(||yaml::get_str(yaml, "date").and_then(|s|util::yaml::parse_dmy_date_range(s)))
    }

    /// When was the project payed
    pub fn payed(yaml: &Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "invoice/payed_date")
        // old spec
        .or_else(|| yaml::get_dmy(yaml, "payed_date"))
    }

    /// When were the wages payed
    pub fn wages(yaml: &Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "hours/wages_date")
        // old spec
        .or_else(|| yaml::get_dmy(yaml, "wages_date"))
    }

    /// When was the offer created
    pub fn offer(yaml: &Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "offer/date")
    }

    /// When was the invoice created
    pub fn invoice(yaml: &Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "invoice/date")
        // old spec
        .or_else(|| yaml::get_dmy(yaml, "invoice_date"))
    }

    /// Date of first event
    pub fn event(yaml: &Yaml) -> Option<Date<UTC>> {
        yaml::get_dmy(yaml, "event/dates/0/begin")
    }

    // TODO packed to deep? Clippy says YES, remove this allow!
    pub type DateRange = (Option<Date<UTC>>, Option<Date<UTC>>);
    pub type DateRanges = Vec<DateRange>;

    /// Produces a list of `DateRange`s for the event.
    pub fn events(yaml: &Yaml) -> Option<DateRanges> {
        yaml::get(yaml, "event/dates/")
            .and_then(|e| e.as_vec())
            .map(|v| {
                v.iter()
                    .map(|e| {
                        (yaml::get_dmy(e, "begin"),
                         yaml::get_dmy(e, "end").or_else(|| yaml::get_dmy(e, "begin")))
                    })
                    .collect::<Vec<(Option<Date<UTC>>, Option<Date<UTC>>)>>()
            })
    }
}

/// Stage 1: requirements for an offer
pub mod offer {
    use util::yaml;
    use util::yaml::Yaml;

    pub fn number(yaml: &Yaml) -> Option<String> {
        let num = appendix(yaml).unwrap_or(1);
        super::date::offer(yaml)
            .map(|d| d.format("A%Y%m%d").to_string())
            .map(|s| format!("{}-{}", s, num))

        // old spec
        .or_else(|| yaml::get_string(yaml, "manumber"))
    }

    pub fn appendix(yaml: &Yaml) -> Option<i64> {
        yaml::get_int(yaml, "offer/appendix")
    }

    pub fn validate(yaml: &Yaml) -> super::SpecResult {
        if super::project::canceled(yaml) {
            return Err(vec!["canceled"]);
        }

        let mut errors = super::field_exists(yaml, &["offer/date", "offer/appendix", "manager"]);
        if super::date::offer(yaml).is_none() {
            errors.push("offer_date_format");
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

/// Stage 2: requirements for an invoice
pub mod invoice {
    use util::yaml;
    use util::yaml::Yaml;
    use chrono::Datelike;

    /// plain access to `invoice/number`
    pub fn number(yaml: &Yaml) -> Option<i64> {
        yaml::get_int(yaml, "invoice/number")
        // old spec
        .or_else(|| yaml::get_int(yaml, "rnumber"))
    }

    pub fn number_str(yaml: &Yaml) -> Option<String> {
        number(yaml).map(|n| format!("R{:03}", n))
    }

    pub fn number_long_str(yaml: &Yaml) -> Option<String> {
        let year = try_some!(super::date::invoice(yaml)).year();
        // TODO Length or format should be a setting
        number(yaml).map(|n| format!("R{}-{:03}", year, n))
    }

    pub fn official(yaml: &Yaml) -> Option<String> {
        yaml::get_string(yaml, "invoice/official")
    }

    pub fn validate(yaml: &Yaml) -> super::SpecResult {
        let mut errors = super::field_exists(yaml, &["invoice/number"]);

        // if super::offer::validate(yaml).is_err() {errors.push("offer")}
        if super::date::invoice(yaml).is_none() {
            errors.push("invoice_date");
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

/// Stage 3: requirements to archive
pub mod archive {
    use util::yaml::Yaml;

    pub fn validate(yaml: &Yaml) -> super::SpecResult {
        let mut errors = Vec::new();
        if super::date::payed(yaml).is_none() { errors.push("payed_date"); }
        //if super::date::wages(yaml).is_none(){ errors.push("wages_date");} // TODO validate WAGES_DATE also
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

/// Everything related to service hours of a project
pub mod hours {
    use currency::Currency;
    use util::yaml;
    use util::yaml::Yaml;
    use super::to_currency;

    /// Salary
    pub fn salary(yaml: &Yaml) -> Option<Currency> {
        yaml::get_f64(yaml, "hours/salary").map(to_currency)
    }

    /// Full number of service hours
    /// XXX test this against old format
    pub fn total(yaml: &Yaml) -> Option<f64> {
        caterers(yaml).map(|vec| {
            vec.iter()
                .map(|&(_, h)| h)
                .fold(0f64, |acc, h| acc + h)
        })
        //.or_else(|| )
    }

    /// Nicely formated list of caterers with their respective service hours
    pub fn caterers_string(yaml: &Yaml) -> Option<String> {
        caterers(yaml).map(|v| {
            v.iter()
                .map(|t| format!("{}: ({})", t.0, t.1))
                .collect::<Vec<String>>()
                .join(", ")
        })
    }

    /// List of caterers and ther respective service hours
    pub fn caterers(yaml: &Yaml) -> Option<Vec<(String, f64)>> {
        yaml::get_hash(yaml, "hours/caterers").map(|h| {
            h.iter()
                .map(|(c, h)| {
                    (// argh, those could be int or float, grrr
                     c.as_str().unwrap_or("").to_owned(),
                     h.as_f64()
                        .or_else(|| // sorry for this
                             h.as_i64().map(|f|f as f64 ))
                        .unwrap_or(0f64))
                })
                .collect::<Vec<(String, f64)>>()
        })
    }
}

/// Generates `Bill`s for offer and invoice.
pub mod billing {
    use bill::{BillItem, Bill};

    use util::yaml;
    use util::yaml::Yaml;
    use ::project::error::{ProductResult,ProductError};
    use ::project::product::Product;

    fn item_from_desc_and_value<'y>(desc: &'y Yaml, values: &'y Yaml) -> ProductResult<(BillItem<Product<'y>>,BillItem<Product<'y>>)> {
        let product = try!(Product::from_desc_and_value(desc, values));

        let offered = try!(yaml::get_f64(values, "amount")
                           .ok_or(ProductError::MissingAmount(product.name.to_owned())));
        let sold = yaml::get_f64(values, "sold");
        let sold = if let Some(returned) = yaml::get_f64(values, "returned") {
            // if "returned", there must be no "sold"
            if sold.is_some() {
                return Err(ProductError::AmbiguousAmounts(product.name.to_owned()));
            }
            if returned > offered {
                return Err(ProductError::TooMuchReturned(product.name.to_owned()));
            }
            offered - returned
        } else if let Some(sold) = sold {
            sold
        } else {
            offered
        };

        Ok(( BillItem{ amount: offered, product: product }, BillItem{ amount: sold, product: product }))
    }

    /// Produces two `Bill`s, one for the offer and one for the invoice
    pub fn bills(yaml: &Yaml) -> ProductResult<(Bill<Product>, Bill<Product>)>{
        let mut offer: Bill<Product> = Bill::new();
        let mut invoice: Bill<Product> = Bill::new();

        let service = || Product {
            name: "Service",
            unit: None,
            tax: ::ordered_float::OrderedFloat(0f64),
            price: super::hours::salary(&yaml).unwrap()
        };

        if let Some(total) = super::hours::total(&yaml) {
            offer.add_item(total, service());
            invoice.add_item(total, service());
        }

        let raw_products = try!(yaml::get_hash(yaml, "products").ok_or(ProductError::UnknownFormat));

        for (desc,values) in raw_products {
            let (offer_item, invoice_item) = try!(item_from_desc_and_value(desc, values));
            offer.add(offer_item);
            invoice.add(invoice_item);
        }

        Ok((offer,invoice))
    }
}
