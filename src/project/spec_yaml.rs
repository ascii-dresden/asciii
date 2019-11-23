use std::str::FromStr;

use bill::{Bill, Currency, Tax};
use icalendar::Event as CalEvent;
use icalendar::{Component, Calendar};
use failure::bail;
use yaml_rust::Yaml;

use super::*;
use super::spec::*;
use super::error::ValidationResult;
use super::product::ProductError;
use super::yaml_provider::error::FieldResultExt;
use crate::util::{self, to_currency};
use crate::util::yaml::parse_dmy_date;

impl YamlProvider for Project {
    fn data(&self) -> &Yaml {
        self.yaml()
    }
}

impl IsProject for Project {
    fn name(&self) -> FieldResult<&str> {
        self.get_str("event.name")
            // old spec
            .if_missing_try(|| self.get_str("event"))
    }

    fn event_date(&self) -> FieldResult<Date<Utc>> {
        self.get_dmy("event.dates.0.begin")
            .if_missing_try(|| self.get_dmy("created"))
            .if_missing_try(|| self.get_dmy_legacy_range("date"))
    }

    //#[deprecated(note="Ambiguous: what format? use \"Version\"")]
    fn format(&self) -> FieldResult<Version> {
        self.get_str("meta.format")
            // old spec
            .if_missing_try(|| self.get_str("format"))
            .and_then(|s| Version::from_str(s).map_err(|e| FieldError::from(e)))
    }

    fn canceled(&self) -> bool {
        self.get_bool("canceled").unwrap_or(false)
    }

    fn responsible(&self) -> FieldResult<&str> {
        self.get_str("manager")
            // old spec
            .if_missing_try(|| self.get_str("signature").and_then(|c| c.lines().last().ok_or(FieldError::invalid("invalid signature"))))
    }

    fn long_desc(&self) -> String {
        use std::fmt::Write;
        let mut out_string = String::new();

        if let Some(responsible) = self.responsible().ok() {
            out_string += &lformat!("Responsible: {}", responsible);
        }

        if let Some(employees) = self.hours().employees_string() {
            writeln!(out_string, "\n{}", employees).unwrap();
        }

        out_string
    }
}

impl HasEvents for Project {
    fn to_ical(&self) -> Calendar {
        let mut calendar = Calendar::new();
        if let Some(events) = self.events() {
            for event in events {
                if event.times.is_empty() {

                    let mut cal_event = CalEvent::new();
                    cal_event.description(&self.long_desc());

                    if let Some(location) = self.location().ok() {
                        cal_event.location(location);
                    }

                    if let Some(end) = event.end {
                        cal_event.start_date(event.begin);
                        cal_event.end_date(end);
                    } else {
                        cal_event.all_day(event.begin);
                    }

                    cal_event.summary(&self.name().unwrap_or("unnamed"));
                    calendar.push(cal_event);

                } else {
                    for time in &event.times {

                        let mut cal_event = CalEvent::new();
                        cal_event.description(&self.long_desc());
                        if let Some(location) = self.location().ok() {
                            cal_event.location(location);
                        }

                        if let Some(end) = event.begin.and_time(time.end) {
                            cal_event.ends(end);
                        }

                        if let Some(start) = event.begin.and_time(time.start) {
                            cal_event.starts(start);
                        }

                        //cal_event.start_date(event.begin);

                        cal_event.summary(&self.name().unwrap_or("unnamed"));
                        calendar.push(cal_event);
                    }
                }
            }
        }

        calendar
    }

    #[allow(unused_qualifications)]
    fn events(&self) -> Option<Vec<spec::Event>> {
        let dates = YamlProvider::get(self, "event.dates/")
            .and_then(Yaml::as_vec)?;
        dates.iter()
             .map(|h| {

            let begin = self.get_direct(h, "begin")
                            .and_then(Yaml::as_str)
                            .and_then(|d| parse_dmy_date(d))?;

            let end = self.get_direct(h, "end")
                          .and_then(Yaml::as_str)
                          .and_then(|d| parse_dmy_date(d));

            Some(spec::Event {
                     begin,
                     end,
                     times: self.times(h).unwrap_or_else(Vec::new),
                 })
        })
             .collect()
    }

    fn times(&self, yaml: &Yaml) -> Option<Vec<EventTime>> {
        let times = self.get_direct(yaml, "times").and_then(Yaml::as_vec)?;
        times.iter()
             .map(|h| {

            let start = self.get_direct(h, "begin")
                            .and_then(Yaml::as_str)
                            .or(Some("00.00"))
                            .and_then(util::naive_time_from_str);

            let end = self.get_direct(h, "end")
                          .and_then(Yaml::as_str)
                          .and_then(util::naive_time_from_str)
                          .or(start); // TODO: assume a duration of one hour instead

            if let (Some(start), Some(end)) = (start, end) {
                Some(EventTime {
                         start,
                         end,
                     })
            } else {
                None
            }
        })
             .collect()
    }

    fn location(&self) -> FieldResult<&str> {
        self.get_str("event.location")
    }
}

/// Returns a product from Service
fn service_to_product<'a, T: HasEmployees>(s: &T) -> Result<Product<'a>, Error> {
    if let Some(salary) = s.salary().ok() {
        Ok(Product {
                 name: "Service",
                 unit: Some("h"),
                 tax: s.tax().ok().unwrap_or_else(|| Tax::new(0.0)),
                 price: salary,
             })
    } else {
        bail!(ProductError::InvalidServerSection)
    }
}

impl Redeemable for Project {
    fn payed_date(&self) -> FieldResult<Date<Utc>> {
        self.get_dmy("invoice.payed_date")
        // old spec
        .if_missing_try(|| self.get_dmy("payed_date"))
    }

    fn is_payed(&self) -> bool {
        self.payed_date().ok().is_some()
    }

    fn tax(&self) -> FieldResult<Tax> {
        self.get_f64("tax").map(Tax::new)
    }

    fn bills(&self) -> Result<(Bill<Product<'_>>, Bill<Product<'_>>), Error> {
        let mut offer: Bill<Product<'_>> = Bill::new();
        let mut invoice: Bill<Product<'_>> = Bill::new();

        let service = service_to_product(&self.hours())?;
       //  .("cannot create product from employees, salary or tax missing");

        if let Some(total) = self.hours().total_time() {
            if total.is_normal() {
                offer.add_item(total, service);
                invoice.add_item(total, service);
            }
        }

        let raw_products =
            self.get_hash("products")
                .ok().ok_or_else(|| ProductError::UnknownFormat)?;

        // let document_tax =  // TODO: activate this once the tax no longer 19%

        for (desc, values) in raw_products {
            let (offer_item, invoice_item) = self.item_from_desc_and_value(desc, values)?;
            if offer_item.amount.is_normal() {
                offer.add(offer_item);
            }
            if invoice_item.amount.is_normal() {
                invoice.add(invoice_item);
            }
        }

        Ok((offer, invoice))
    }
}

impl Validatable for Project {
    fn validate(&self) -> ValidationResult {
        let mut validation = ValidationResult::new();

        validation.require_field("name", self.name());
        validation.require_field("date", self.event_date());
        validation.require_field("manager", self.responsible());
        validation.require_field("format", self.format());

        validation
    }
}


impl Validatable for dyn Redeemable {
    fn validate(&self) -> ValidationResult {
        let mut validation = ValidationResult::new();

        validation.validate_field("format", self.format());
        if let Some(format) = self.format().ok() {
            if format < Version::parse("2.0.0").unwrap() {
                return validation;
            }
        }

        validation.require_field("payed_date", self.payed_date());
        validation
    }
}

impl<'a> YamlProvider for Client<'a> {
    fn data(&self) -> &Yaml {
        self.inner.data()
    }
}

impl<'a> IsClient for Client<'a> {
    fn email(&self) -> FieldResult<&str> {
        self.get_str("client/email")
            .if_missing_try(|| self.get_str("email"))
    }

    fn address(&self) -> FieldResult<&str> {
        self.get_str("client.address")
            // old spec
            .if_missing_try(|| self.get_str("address"))
    }

    fn title(&self) -> FieldResult<&str> {
        self.get_str("client/title")
            // old spec
            .if_missing_try(|| self
                .get_str("client")
                .and_then(|c|c.lines().nth(0).ok_or(FieldError::invalid("invalid client name")))
            )
    }

    fn salute(&self) -> FieldResult<&str> {
        self.title().and_then(|s| s.split_whitespace().nth(0).ok_or(FieldError::invalid("title has no salute")))
    }

    fn first_name(&self) -> FieldResult<&str> {
        self.get_str("client.first_name")
        // old spec
        // .or_else(|_|  yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(0)))
    }

    fn last_name(&self) -> FieldResult<&str> {
        self.get_str("client.last_name")
        // old spec
        .if_missing_try(|| self.get_str("client").and_then(|c|c.lines().nth(1).ok_or(FieldError::invalid("invalid client name"))))
    }

    fn full_name(&self) -> Option<String> {
        let first = self.first_name().ok();
        let last = self.last_name().ok();
        first.and(last)
             .and(Some(format!("{} {}", first.unwrap_or(""), last.unwrap_or(""))))
    }

    fn addressing(&self) -> Option<String> {
        if let Some(salute) = self.salute().ok()
                                  .and_then(|salute| salute.split_whitespace().nth(0))
        // only the first word
        {
            let last_name = self.last_name().ok();


            let lang = crate::CONFIG.get_str("defaults/lang");

            let gender_path = "gender_matches/".to_owned() + &salute.to_lowercase();
            let gender = crate::CONFIG.get_str_or(&gender_path)?;

            let addr_path = "lang_addressing/".to_owned() + &lang.to_lowercase() + "/" + gender;
            let addr = crate::CONFIG.get_str_or(&addr_path)?;

            last_name.and(Some(format!("{} {} {}", addr, salute, last_name.unwrap_or(""))))
        } else {
            None
        }
    }
}

impl<'a> Validatable for Client<'a> {
    fn validate(&self) -> ValidationResult {
        let mut validation = ValidationResult::new();

        validation.require_field("client/address", self.address());
        validation.require_field("client/title", self.title());
        validation.require_field("client/last_name", self.last_name());
        validation.require_field("client/first_name", self.first_name());
        validation.require_option("client_addressing", self.addressing());

        validation
    }
}

impl<'a> YamlProvider for Offer<'a> {
    fn data(&self) -> &Yaml {
        self.inner.data()
    }
}

impl<'a> Offerable for Offer<'a> {
    fn appendix(&self) -> FieldResult<i64> {
        self.get_int("offer.appendix")
    }

    fn date(&self) -> FieldResult<Date<Utc>> {
        self.get_dmy("offer.date")
    }

    fn number(&self) -> FieldResult<String> {
        let num = self.appendix().unwrap_or(1);
        Offerable::date(self)
            //.map(|d| d.format("%Y%m%d").to_string())
            .map(|d| d.format("A%Y%m%d").to_string())
            .map(|s| format!("{}-{}", s, num))

        // old spec
        .if_missing_try(|| self.get_str("manumber").map(ToString::to_string))
    }
}

impl<'a> Validatable for Offer<'a> {
    fn validate(&self) -> ValidationResult {
        let mut validation = ValidationResult::new();

        validation.require_field("offer.date", self.date());
        validation.require_field("manager", self.inner.responsible());
        validation.require_field("appendix", self.appendix());

        validation
    }
}

impl<'a> YamlProvider for Invoice<'a> {
    fn data(&self) -> &Yaml {
        self.inner.data()
    }
}

impl<'a> Invoicable for Invoice<'a> {
    fn number(&self) -> FieldResult<i64> {
        self.get_int("invoice.number")
            .if_missing_try(|| self.get_int("rnumber"))
    }

    fn date(&self) -> FieldResult<Date<Utc>> {
        self.get_dmy("invoice.date")
            .if_missing_try(|| self.get_dmy("invoice_date"))
    }

    fn number_str(&self) -> Option<String> {
        self.number().ok().map(|n| format!("R{:03}", n))
    }

    fn number_long_str(&self) -> Option<String> {
        let year = self.date().ok()?.year();
        // TODO: Length or format should be a setting
        self.number().ok().map(|n| format!("R{}-{:03}", year, n))
    }

    fn official(&self) -> FieldResult<String> {
        self.get_str("invoice.official").map(ToOwned::to_owned)
    }
}

impl<'a> Validatable for Invoice<'a> {
    fn validate(&self) -> ValidationResult {
        let mut validation = ValidationResult::new();

        validation.require_field("invoice.number", self.number());
        validation.require_field("invoice.date", self.date());

        validation
    }
}

impl<'a> YamlProvider for Hours<'a> {
    fn data(&self) -> &Yaml {
        self.inner.data()
    }
}

impl<'a> HasEmployees for Hours<'a> {
    fn wages_date(&self) -> FieldResult<Date<Utc>> {
        self.get_dmy("hours.wages_date")
        // old spec
        .or_else(|_|  self.get_dmy("wages_date"))
    }

    fn salary(&self) -> FieldResult<Currency> {
        self.get_f64("hours.salary").map(to_currency)
    }

    fn tax(&self) -> FieldResult<Tax> {
        self.get_f64("hours.tax").map(Tax::new)
    }

    fn net_wages(&self) -> Option<Currency> {
        let triple = (self.total_time(), self.salary().ok(), self.tax().ok());
        match triple {
            (Some(total_time), Some(salary), Some(tax)) => Some(total_time * salary * (tax.value() + 1f64)),
            // covering the legacy case where Services always had Tax=0%
            (Some(total_time), Some(salary), None) => Some(total_time * salary),
            _ => None,
        }
    }

    fn gross_wages(&self) -> Option<Currency> {
        let tuple = (self.total_time(), self.salary().ok());
        if let (Some(total_time), Some(salary)) = tuple {
            Some(total_time * salary)
        } else {
            None
        }
    }

    fn total_time(&self) -> Option<f64> {
        self.employees().ok()
            .map(|e| {
                     e.iter()
                      .fold(0f64, |acc, e| acc + e.time)
                 })
    }

    fn employees_string(&self) -> Option<String> {
        self.employees().ok()
            .map(|e| {
            e.iter()
             .filter(|e| e.time as u32 > 0)
             .map(|e| {
                      format!("{}: ({}h {})",
                              e.name,
                              e.time,
                              (e.salary * e.time).postfix())
                  })
             .collect::<Vec<String>>()
             .join(", ")
        })
    }

    fn employees(&self) -> FieldResult<Vec<Employee>> {
        let employees = self.get_hash("hours.caterers")
                            .or_else(|_| self.get_hash("hours.employees"));

            employees?.iter()
                     .map(|(c, h)| {(c.as_str().unwrap_or("").into(), make_float(h))
                     })
                     .filter(|&(_, h)| h > 0f64)
                     .map(|(name, time)| {
                let wage = self.salary()? * time;
                let salary = self.salary()?;
                FieldResult::Ok(Employee {
                         name,
                         time,
                         wage,
                         salary,
                     })
            })
            .collect::<FieldResult<Vec<Employee>>>()
    }

    fn employees_payed(&self) -> bool {
        self.employees().is_err() || self.wages_date().is_ok()
    }

    fn wages(&self) -> Option<Currency> {
        if let (Some(total), Some(salary)) = (self.total_time(), self.salary().ok()) {
            Some(total * salary)
        } else {
            None
        }
    }
}

// helper for HasEmployees::employees()
fn make_float(h: &Yaml) -> f64 {
    h.as_f64()
     .or_else(|| h.as_i64().map(|f| f as f64))
     .unwrap_or(0f64)
}



impl<'a> Validatable for Hours<'a> {
    fn validate(&self) -> ValidationResult {
        let mut validation = ValidationResult::new();

        validation.validate_field("hours.caterers", self.employees());

        // return directly if no employees need to be paid
        if self.employees().unwrap_or(Vec::new()).is_empty() {
            return validation
        }

        // check that payment validates
        validation.validate_field("hours.tax", self.tax());
        validation.validate_field("hours.salary", self.salary());
        validation.require_field("hours.wages_date", self.wages_date());

        validation
    }
}
