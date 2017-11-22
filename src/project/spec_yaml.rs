use std::str::FromStr;

use bill::{Bill, Currency, Tax};
use icalendar::Event as CalEvent;
use icalendar::{Component, Calendar};
use yaml_rust::Yaml;
use yaml_rust::yaml::Hash as YamlHash;

use super::*;
use super::spec::*;
use super::product::error::Result as ProductResult;
use util::yaml;
use util::to_currency;

use util;

/// Enables access to structured data via a simple path
///
/// A path can be something like `users/clients/23/name`
/// but also  `users.clients.23.name`
pub trait ProvidesData {
    /// You only need to implement this.
    //fn data(&self) -> impl PathAccessible {
    fn data(&self) -> &Yaml;

    /// Wrapper around `get_path()`.
    ///
    /// Splits path string
    /// and replaces `Yaml::Null` and `Yaml::BadValue`.
    fn get<'a>(&'a self, path:&str) -> Option<&'a Yaml> {
        self.get_direct(self.data(), path)
    }

    /// Wrapper around `get_path()`.
    ///
    /// Splits path string
    /// and replaces `Yaml::Null` and `Yaml::BadValue`.
    fn get_direct<'a>(&'a self, data:&'a Yaml, path:&str) -> Option<&'a Yaml> {
        // TODO this can be without copying
        let path = path.split(|p| p == '/' || p == '.')
                      .filter(|k|!k.is_empty())
                      .collect::<Vec<&str>>();
        match self.get_path(data, &path) {
            Some(&Yaml::BadValue) |
            Some(&Yaml::Null) => None,
            content => content
        }
    }

    /// Returns content at `path` in the yaml document.
    /// TODO make this generic over the type of data to support more than just `Yaml`.
    fn get_path<'a>(&'a self, data:&'a Yaml, path:&[&str]) -> Option<&'a Yaml>{
        if let Some((&path, remainder)) = path.split_first() {
            match *data {
                // go further into the rabit hole
                Yaml::Hash(ref hash) => {
                    if remainder.is_empty(){
                        hash.get(&Yaml::String(path.to_owned()))
                    } else {
                        hash.get(&Yaml::String(path.to_owned()))
                            .and_then(|c| self.get_path(c, remainder))
                    }
                },
                // interpret component as index
                Yaml::Array(ref vec) => {
                    if let Ok(index) = path.parse::<usize>() {
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

    /// Gets a Date in `dd.mm.YYYY` format.
    fn get_dmy(&self, path:&str) -> Option<Date<Utc>> {
        self.get(path).and_then(|y|y.as_str()).and_then(|d|self.parse_dmy_date(d))
    }

    /// Interprets `"25.12.2016"` as date.
    fn parse_dmy_date(&self, date_str:&str) -> Option<Date<Utc>>{
        let date = date_str.split('.')
            .map(|f|f.parse().unwrap_or(0))
            .collect::<Vec<i32>>();
        if date.len() >=2 && date[0] > 0 && date[2] > 1900 {
            // XXX this neglects the old "01-05.12.2015" format
            Utc.ymd_opt(date[2], date[1] as u32, date[0] as u32).single()
        } else {
            None
        }
    }

    /// Gets a `Bool` value.
    ///
    /// **Careful** this is a bit sweeter then ordinary `YAML1.2`,
    /// this will interpret `"yes"` and `"no"` as booleans, similar to `YAML1.1`.
    /// Actually it will interpret any string but `"yes"` als `false`.
    fn get_bool(&self, path:&str) -> Option<bool> {
        self.get(path)
            .and_then(|y| y
                      .as_bool()
                      // allowing it to be a str: "yes" or "no"
                      .or_else(|| y.as_str()
                           .map( |yes_or_no|
                                 match yes_or_no.to_lowercase().as_ref() {
                                     "yes" => true,
                                     //"no" => false,
                                     _ => false
                                 })
                         ))
    }

    fn field_exists<'a>(&'a self, paths: &[&'a str]) -> ErrorList {
        let mut errors = ErrorList::new();
        for err in paths.into_iter()
            .map(|i|*i)
                .filter(|path| self.get(path).is_none()) {
                    errors.push(err);
                }
        errors

    }

    /// Gets `Some(Yaml::Hash)` or `None`.
    //pub fn get_hash<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a BTreeMap<Yaml,Yaml>> {
    fn get_hash<'a>(&'a self, path:&str) -> Option<&'a YamlHash> {
        self.get(path).and_then(|y|y.as_hash())
    }

    /// Gets a `Float` value.
    ///
    /// Also takes a `Yaml::I64` and reinterprets it.
    fn get_f64(&self, path:&str) -> Option<f64> {
        self.get(path).and_then(|y| y.as_f64().or_else(|| y.as_i64().map(|y|y as f64)))
    }
}

impl ProvidesData for Project {
    fn data(&self) -> &Yaml{ self.yaml() }
}

impl IsProject for Project {
    fn name(&self) -> Option<&str> {
        self.get_str("event.name")
            // old spec
            .or_else(|| self.get_str("event"))
    }

    fn event_date(&self) -> Option<Date<Utc>>{
        self.get_dmy( "event.dates.0.begin")
        .or_else(||self.get_dmy("created"))
        .or_else(||self.get_dmy("date"))
        // probably the dd-dd.mm.yyyy format
        .or_else(||self.get_str("date")
                 .and_then(|s| yaml::parse_dmy_date_range(s))
                 )
    }

    //#[deprecated(note="Ambiguous: what format? use \"Version\"")]
    fn format(&self) -> Option<Version> {
        self.get_str("meta.format")
            .or_else(||self.get_str("format"))
            .and_then(|s| Version::from_str(s).ok())
    }

    fn canceled(&self) -> bool{
        self.get_bool("canceled").unwrap_or(false)
    }

    fn responsible(&self) -> Option<&str> {
        self.get_str("manager")
        // old spec
        .or_else(|| self.get_str("signature").and_then(|c|c.lines().last()))
    }

    fn long_desc(&self) -> String {
        use std::fmt::Write;
        let mut out_string = String::new();

        if let Some(responsible) = self.responsible() {
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

                    if let Some(location) = self.location() { cal_event.location(location); }

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
                        if let Some(location) = self.location() { cal_event.location(location); }

                        if let Some(end)   = event.begin.and_time(time.end) {
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
        let dates = try_some!(ProvidesData::get(self, "event.dates/").and_then(|a| a.as_vec()));
        dates.into_iter()
            .map(|h| {

                let begin = try_some!(
                    self.get_direct(h, "begin")
                    .and_then(|y|y.as_str())
                    .and_then(|d|self.parse_dmy_date(d)));

                let end =
                    self.get_direct(h, "end")
                    .and_then(|y|y.as_str())
                    .and_then(|d|self.parse_dmy_date(d));

                Some( spec::Event{
                    begin: begin,
                    end: end,
                    times: self.times(h).unwrap_or_else(Vec::new)
                })
            })

        .collect()
    }

    fn times(&self,yaml: &Yaml) -> Option<Vec<EventTime>> {
        let times = try_some!(self.get_direct(yaml, "times").and_then(|l|l.as_vec()));
        times.into_iter()
            .map(|h| {

                let begin = self.get_direct(h, "begin")
                    .and_then(|y|y.as_str())
                    .or(Some("00.00"))
                    .and_then(util::naive_time_from_str);

                let end   = self.get_direct(h, "end")
                    .and_then(|y|y.as_str())
                    .and_then(util::naive_time_from_str)
                    .or(begin); // TODO assume a duration of one hour instead

                if let (Some(begin),Some(end)) = (begin,end) {
                    Some( EventTime{
                        start: begin,
                        end: end
                    })
                } else { None }
            })
        .collect()
    }

    fn location(&self) -> Option<&str> {
        self.get_str("event.location")
    }
}

/// Returns a product from Service
fn service_to_product<'a, T: HasEmployees>(s: &T) -> Option<Product<'a>> {
    if let Some(salary) = s.salary() {
        Some(Product {
            name: "Service",
            unit: Some("h"),
            tax: s.tax().unwrap_or_else(|| Tax::new(0.0)),
            price: salary
        })
    } else {
        None
    }
}

impl Redeemable for Project {
    fn payed_date(&self) -> Option<Date<Utc>> {
        self.get_dmy("invoice.payed_date")
        // old spec
        .or_else(|| self.get_dmy("payed_date"))
    }

    fn is_payed(&self) -> bool {
        self.payed_date().is_some()
    }

    fn tax(&self) -> Option<Tax> {
        self.get_f64("tax").map(Tax::new)
    }

    fn bills(&self) -> ProductResult<(Bill<Product>, Bill<Product>)> {
        let mut offer: Bill<Product> = Bill::new();
        let mut invoice: Bill<Product> = Bill::new();

        let service = service_to_product(&self.hours())
            .expect("cannot create product from employees, salary or tax missing");

        if let Some(total) = self.hours().total_time() {
            if total.is_normal() {
                offer.add_item(total, service);
                invoice.add_item(total, service);
            }
        }

        let raw_products =
            self.get_hash("products")
                .ok_or_else(||product::error::Error::from(product::error::ErrorKind::UnknownFormat))?;

        // let document_tax =  // TODO activate this once the tax no longer 19%

        for (desc,values) in raw_products {
            let (offer_item, invoice_item) = self.item_from_desc_and_value(desc, values)?;
            if offer_item.amount.is_normal()   { offer.add(offer_item); }
            if invoice_item.amount.is_normal() { invoice.add(invoice_item); }
        }

        Ok((offer,invoice))
    }

}

impl Validatable for Project {
    fn validate(&self) -> SpecResult {
        let mut errors = ErrorList::new();
        if self.name().is_none(){errors.push("name")}
        if self.event_date().is_none(){errors.push("date")}
        if self.responsible().is_none(){errors.push("manager")}
        if self.format().is_none(){errors.push("format")}
        //if hours::salary().is_none(){errors.push("salary")}

        if errors.is_empty(){ Ok(()) }
        else { Err(errors) }
    }
}


impl Validatable for Redeemable {
    fn validate(&self) -> SpecResult {
        let mut errors = ErrorList::new();
        if self.payed_date().is_none() { errors.push("payed_date"); }

        if let Some(format) = self.format() {
            if format < Version::parse("2.0.0").unwrap() {
                return Ok(());
            }
        }

        if !errors.is_empty() {
            bail!(errors);
        }

        Ok(())
    }
}

impl<'a> ProvidesData for Client<'a> {
    fn data(&self) -> &Yaml{
        self.inner.data()
    }
}

impl<'a> IsClient for Client<'a> {
    fn email(&self) -> Option<&str> {
        self.get_str("client/email")
    }

    fn address(&self) -> Option<&str> {
        self.get_str("client/address")
        .or_else(|| self.get_str("address"))
    }

    fn title(&self) -> Option<&str> {
        self.get_str("client/title")
        // old spec
        .or_else(|| self.get_str("client").and_then(|c|c.lines().nth(0)))
    }

    fn salute(&self) -> Option<&str> {
        self.title().and_then(|s|s.split_whitespace().nth(0))
    }

    fn first_name(&self) -> Option<&str> {
        self.get_str("client.first_name")
        // old spec
        // .or_else(|| yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(0)))
    }

    fn last_name(&self) -> Option<&str> {
        self.get_str("client.last_name")
        // old spec
        .or_else(|| self.get_str("client").and_then(|c|c.lines().nth(1)))
    }

    fn full_name(&self) -> Option<String> {
        let first = self.first_name();
        let last = self.last_name();
        first.and(last)
             .and(Some(format!("{} {}",
                               first.unwrap_or(""),
                               last.unwrap_or(""))))
    }

    fn addressing(&self) -> Option<String> {
        if let Some(salute) = self.salute().and_then(|salute| salute.split_whitespace().nth(0))
        // only the first word
        {
            let last_name = self.last_name();


            let lang = ::CONFIG.get_str("defaults/lang");

            let gend_path = "gender_matches/".to_owned() + &salute.to_lowercase();
            let gend = ::CONFIG.get_str(&gend_path);

            let addr_path = "lang_addressing/".to_owned() + &lang.to_lowercase() + "/" + gend;
            let addr = ::CONFIG.get_str(&addr_path);

            last_name.and(Some(format!("{} {} {}", addr, salute, last_name.unwrap_or(""))))
        } else {
            None
        }
    }
}

impl<'a> Validatable for Client<'a> {
    fn validate(&self) -> SpecResult {
        let mut errors = self.field_exists( &[
                                             //"client/email", // TODO make this a requirement
                                             "client/address",
                                             "client/title",
                                             "client/last_name",
                                             "client/first_name"
                                             ]);


        if self.addressing().is_none() {
            errors.push("client_addressing");
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

impl<'a> ProvidesData for Offer<'a> {
    fn data(&self) -> &Yaml{
        self.inner.data()
    }
}

impl<'a> Offerable for Offer<'a> {
    fn appendix(&self) -> Option<i64> {
        self.get_int("offer.appendix")
    }

    fn date(&self) -> Option<Date<Utc>> {
        self.get_dmy("offer.date")
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

impl<'a> Validatable for Offer<'a> {
    fn validate(&self) -> SpecResult {
        //if IsProject::canceled(self) {
        //    return Err(vec!["canceled"]);
        //}

        let mut errors = self.field_exists(&["offer.date", "offer.appendix", "manager"]);
        if Offerable::date(self).is_none() {
            errors.push("offer_date_format");
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())

    }
}

impl<'a> ProvidesData for Invoice<'a> {
    fn data(&self) -> &Yaml{
        self.inner.data()
    }
}

impl<'a> Invoicable for Invoice<'a> {
    fn number(&self) -> Option<i64> {
        self.get_int("invoice.number")
        // old spec
        .or_else(|| self.get_int("rnumber"))
    }

    fn date(&self) -> Option<Date<Utc>> {
        self.get_dmy("invoice.date")
        // old spec
        .or_else(|| self.get_dmy("invoice_date"))
    }

    fn number_str(&self) -> Option<String> {
        self.number().map(|n| format!("R{:03}", n))
    }

    fn number_long_str(&self) -> Option<String> {
        let year = try_some!(self.date()).year();
        // TODO Length or format should be a setting
        self.number().map(|n| format!("R{}-{:03}", year, n))
    }

    fn official(&self) -> Option<String> {
        self.get_str("invoice.official").map(ToOwned::to_owned)
    }
}

impl<'a> Validatable for Invoice<'a> {
    fn validate(&self) -> SpecResult {
        let mut errors = self.field_exists(&["invoice.number"]);

        // if super::offer::validate(yaml).is_err() {errors.push("offer")}
        if self.date().is_none() {
            errors.push("invoice_date");
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

impl<'a> ProvidesData for Hours<'a> {
    fn data(&self) -> &Yaml{
        self.inner.data()
    }
}

impl<'a> HasEmployees for Hours<'a> {
    fn wages_date(&self) -> Option<Date<Utc>> {
        self.get_dmy("hours.wages_date")
        // old spec
        .or_else(|| self.get_dmy("wages_date"))
    }

    fn salary(&self) -> Option<Currency> {
        self.get_f64("hours.salary").map(to_currency)
    }

    fn tax(&self) -> Option<Tax> {
        self.get_f64("hours.tax").map(Tax::new)
    }

    fn net_wages(&self) -> Option<Currency> {
        let triple = (self.total_time(), self.salary(), self.tax());
        match triple {
            ( Some(total_time), Some(salary), Some(tax))
                => Some(total_time * salary * (tax.value() + 1f64)),
            // covering the legacy case where Services always had Tax=0%
            ( Some(total_time), Some(salary), None)
                => Some(total_time * salary),
            _ => None
        }
    }

    fn gross_wages(&self) -> Option<Currency> {
        let tuple = (self.total_time(), self.salary());
        if let ( Some(total_time), Some(salary)) = tuple {
            Some(total_time * salary)
        } else {
            None
        }
    }

    fn total_time(&self) -> Option<f64> {
        self.employees()
            .map(|e|e.iter()
                     .fold(0f64, |acc, e| acc + e.time))
    }

    fn employees_string(&self) -> Option<String> {
        self.employees()
            .map(|e|e.iter()
            .filter(|e| e.time as u32 > 0)
            .map(|e| format!("{}: ({}h {})", e.name, e.time, (e.salary * e.time).postfix()))
            .collect::<Vec<String>>()
            .join(", "))
    }

    fn employees(&self) -> Option<Vec<Employee>> {
        let employees =
        self.get_hash("hours.caterers")
            .or_else(||self.get_hash("hours.employees"));

        if let Some(employees) = employees {
            employees.iter()
                     .map(|(c, h)| (
                             c.as_str().unwrap_or("").into(),
                             make_float(h)
                             )
                         )
                     .filter(|&(_, h)| h > 0f64 )
                     .map(|(name, time)| {
                          let wage = try_some!(self.salary()) * time;
                          let salary = try_some!(self.salary());
                          Some( Employee { name, time, wage, salary })
                      })
                     .collect::< Option<Vec<Employee>> >()
        } else {
            None
        }
    }

    fn employees_payed(&self) -> bool {
        self.employees().is_none()
        ||
        self.wages_date().is_some()
    }

    fn wages(&self) -> Option<Currency> {
        if let (Some(total), Some(salary)) = (self.total_time(), self.salary()) {
            Some(total * salary)
        } else{None}
    }
}

// helper for HasEmployees::employees()
fn make_float(h: &Yaml) -> f64 {
    h.as_f64().or_else(|| h.as_i64().map(|f|f as f64 ))
        .unwrap_or(0f64)
}



impl<'a> Validatable for Hours<'a> {
    fn validate(&self) -> SpecResult {
        let mut errors = ErrorList::new();
        if !self.employees_payed() { errors.push("employees_payed"); }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

