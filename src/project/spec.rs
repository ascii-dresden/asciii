//! Implements the ascii invoicer project file specification.
//!
//! This does all of the heavy lifting.
//! The implementation is separated into sub-modules which take care of separate objectives.
//! Most of the functions in these modules take the `yaml` data directly as reference.
//! Each module contains a `validate()` function which ought to be kept up to date.

use bill::Currency;
use yaml_rust::Yaml;
use yaml_rust::yaml::Hash as YamlHash;

use chrono::{Date, UTC, TimeZone, Datelike};
use semver::Version;

use super::error::{SpecResult, ErrorList};

use std::str::FromStr;


pub fn print_specresult(label: &str, result: SpecResult) {
    match result {
        Ok(_) => println!("{}: ✓", label),
        Err(ref errs) => println!("{}: ✗\n{}", label, errs)
    }
}


// TODO there may be cases where an f64 can't be converted into Currency
pub fn to_currency(f: f64) -> Currency {
    Currency(::CONFIG.get_char("currency"), (f * 1000.0) as i64) / 10
}


/// Interprets `"24-25.12.2016"` as date.
///
/// Takes care of the old, deprecated, stupid, `dd-dd.mm.yyyy` format, what was I thinking?
/// This is not used in the current format.
fn parse_dmy_date_range(date_str:&str) -> Option<Date<UTC>>{
    let date = date_str.split('.')
        .map(|s|s.split('-').nth(0).unwrap_or("0"))
        .map(|f|f.parse().unwrap_or(0))
        .collect::<Vec<i32>>();
    if date[0] > 0 {
        return Some(UTC.ymd(date[2], date[1] as u32, date[0] as u32))
    }
    None
}


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
    fn get_dmy(&self, path:&str) -> Option<Date<UTC>> {
        self.get(path).and_then(|y|y.as_str()).and_then(|d|self.parse_dmy_date(d))
    }

    /// Interprets `"25.12.2016"` as date.
    fn parse_dmy_date(&self, date_str:&str) -> Option<Date<UTC>>{
        let date = date_str.split('.')
            .map(|f|f.parse().unwrap_or(0))
            .collect::<Vec<i32>>();
        if date.len() >=2 && date[0] > 0 && date[2] > 1900 {
            // XXX this neglects the old "01-05.12.2015" format
            UTC.ymd_opt(date[2], date[1] as u32, date[0] as u32).single()
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
                      .or( y.as_str()
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
        self.get(path).and_then(|y| y.as_f64().or( y.as_i64().map(|y|y as f64)))
    }
}

/// Every other trait in this module ought to be `Validatable`
pub trait Validatable {
    fn validate(&self) -> SpecResult;

    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    fn errors<'a>(&'a self) -> Option<ErrorList>{
        self.validate().err()
    }
}

/// Stage 0: the Project itself
///
/// Provide the basics every Project should have.
pub trait IsProject: ProvidesData {
    // TODO reevaluate if these fields really belong here
    fn name(&self) -> Option<&str> {
        self.get_str("event.name")
            // old spec
            .or_else(|| self.get_str("event"))
    }

    fn event_date(&self) -> Option<Date<UTC>>{
        self.get_dmy( "event.dates.0.begin")
        .or_else(||self.get_dmy("created"))
        .or_else(||self.get_dmy("date"))
        // probably the dd-dd.mm.yyyy format
        .or_else(||self.get_str("date")
                 .and_then(|s| parse_dmy_date_range(s))
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

    fn long_desc(&self) -> String;
}


/// Stage 1: requirements for an offer
pub trait Offerable: ProvidesData {
    fn appendix(&self) -> Option<i64> {
        self.get_int("offer.appendix")
    }

    /// When was the offer created
    fn date(&self) -> Option<Date<UTC>> {
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

/// Everything about the client
///
/// This is a [client](../struct.Project.html#method.client)
pub trait IsClient: ProvidesData {
    ///Returns the content of `/client/email`
    fn email(&self) -> Option<&str> {
        self.get_str("client/email")
    }

    ///Returns the content of `/client/address`
    fn address(&self) -> Option<&str> {
        self.get_str("client/address")
        .or_else(|| self.get_str("address"))
    }

    ///Returns the content of `/client/title`
    fn title(&self) -> Option<&str> {
        self.get_str("client/title")
        // old spec
        .or_else(|| self.get_str("client").and_then(|c|c.lines().nth(0)))
    }

    ///Returns the first word of `client/title`
    fn salute(&self) -> Option<&str> {
        self.title().and_then(|s|s.split_whitespace().nth(0))
    }

    ///Returns the content of `/client/first_name`
    fn first_name(&self) -> Option<&str> {
        self.get_str("client.first_name")
        // old spec
        // .or_else(|| yaml::get_str(&yaml, "client").and_then(|c|c.lines().nth(0)))
    }

    ///Returns the content of `/client/last_name`
    fn last_name(&self) -> Option<&str> {
        self.get_str("client.last_name")
        // old spec
        .or_else(|| self.get_str("client").and_then(|c|c.lines().nth(1)))
    }

    /// Combines `first_name` and `last_name`.
    fn full_name(&self) -> Option<String> {
        let first = self.first_name();
        let last = self.last_name();
        first.and(last)
             .and(Some(format!("{} {}",
                               first.unwrap_or(""),
                               last.unwrap_or(""))))
    }

    /// Produces a standard salutation field.
    fn addressing(&self) -> Option<String> {
        if let Some(salute) = self.salute().and_then(|salute| salute.split_whitespace().nth(0))
        // only the first word
        {
            let last_name = self.last_name();


            let lang = ::CONFIG.get_str("defaults/lang")
                .expect("Faulty config: defaults/lang does not contain a value");

            let gend_path = "gender_matches/".to_owned() + &salute.to_lowercase();
            let gend = ::CONFIG.get_str(&gend_path)
                .expect(&format!("Faulty config: {} does not contain a value", gend_path));

            let addr_path = "lang_addressing/".to_owned() + &lang.to_lowercase() + "/" + gend;
            let addr = ::CONFIG.get_str(&addr_path)
                .expect(&format!("Faulty config: {} does not contain a value", addr_path));

            last_name.and(Some(format!("{} {} {}", addr, salute, last_name.unwrap_or(""))))
        } else {
            None
        }
    }

    ///// Validates the output of each of this modules functions.
    //fn validate(&self) -> super::SpecResult {
    //    let mut errors = super::field_exists(yaml,
    //                                         &[
    //                                         //"client/email", // TODO make this a requirement
    //                                         "client/address",
    //                                         "client/title",
    //                                         "client/last_name",
    //                                         "client/first_name"
    //                                         ]);


    //    if addressing(yaml).is_none() {
    //        errors.push("client_addressing");
    //    }
    //    if !errors.is_empty() {
    //        return Err(errors);
    //    }

    //    Ok(())
    //}
}

/// Stage 2: requirements for an invoice
pub trait Invoicable: ProvidesData {
    /// plain access to `invoice/number`
    fn number(&self) -> Option<i64> {
        self.get_int("invoice.number")
        // old spec
        .or_else(|| self.get_int("rnumber"))
    }

    /// When was the invoice created
    fn date(&self) -> Option<Date<UTC>> {
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

    /// An official identifier
    fn official(&self) -> Option<String> {
        self.get_str("invoice.official").map(ToOwned::to_owned)
    }
}


use super::product::error::{Result, Error, ErrorKind};
use super::product::Product;
use bill::{BillItem, Bill};

pub trait HasEmployees: ProvidesData {
    /// When were the wages payed
    fn wages_date(&self) -> Option<Date<UTC>> {
        self.get_dmy("hours.wages_date")
        // old spec
        .or_else(|| self.get_dmy("wages_date"))
    }

    /// Salary
    fn salary(&self) -> Option<Currency> {
        self.get_f64("hours.salary").map(to_currency)
    }

    /// Full number of service hours
    /// TODO test this against old format
    fn total(&self) -> Option<f64> {
        self.employees().map(|vec| {
            vec.iter()
                .map(|&(_, h)| h)
                .fold(0f64, |acc, h| acc + h)
        })
        //.or_else(|| )
    }

    //fn total_salary(&self) -> SpecResult<(f64,Currency)> {
    //    let salary = self.salary();
    //    let total = self.total();
    //    match (salary, total) {
    //        (Some(0), Some(t)) if t > 0  => Err(),
    //        (None, Some(_)) => Err(),
    //        (Some(s), Some(t)) => Ok((t,s))
    //    }
    //}

    /// Nicely formated list of employees with their respective service hours
    fn employees_string(&self) -> Option<String> {
        self.employees().map(|v| {
            v.iter()
                .filter(|&&(_, ref time)| *time as u32 > 0)
                .map(|&(ref name, ref time)| format!("{}: ({})", name, time)) // TODO Fix #57 here
                .collect::<Vec<String>>()
                .join(", ")
        })
    }

    /// List of employees and ther respective service hours
    fn employees(&self) -> Option<Vec<(String, f64)>> {
        self.get_hash("hours.caterers")
            .or(self.get_hash("hours.employees"))
            .and_then(|h| {
                h.iter()
                    .map(|(c, h)| {
                        (// argh, those could be int or float, grrr
                            c.as_str().unwrap_or("").to_owned(),
                            h.as_f64()
                            .or_else(|| // sorry for this
                                     h.as_i64().map(|f|f as f64 ))
                            .unwrap_or(0f64))
                    })
                .map(|(employee,time)| if time > 0f64 {
                    Some((employee, time))
                } else {
                    None
                } )
                .collect::<
                Option<
                Vec<(String, f64)>
                >
                >()
            })
    }
}

/// Stage 3: when an `IsProject` is redeem and can be archived
pub trait Redeemable: IsProject {
    /// When was the project payed
    fn payed_date(&self) -> Option<Date<UTC>> {
        self.get_dmy("invoice.payed_date")
        // old spec
        .or_else(|| self.get_dmy("payed_date"))
    }

    fn bills(&self) -> Result<(Bill<Product>, Bill<Product>)> ;

    /// implementation detail
    /// TODO please move into concrete implementation
    fn item_from_desc_and_value<'y>(&self, desc: &'y Yaml, values: &'y Yaml) -> Result<(BillItem<Product<'y>>,BillItem<Product<'y>>)> {
        let get_f64 = |yaml, path|
            self.get_direct(yaml,path)
                .and_then(|y| y.as_f64()
                               .or( y.as_i64()
                                     .map(|y|y as f64)
                                  )
                         );

        let product = try!(Product::from_desc_and_value(desc, values));

        let offered = try!(get_f64(values, "amount")
                           .ok_or(Error::from(ErrorKind::MissingAmount(product.name.to_owned()))));
        let sold = get_f64(values, "sold");
        let sold = if let Some(returned) = get_f64(values, "returned") {
            // if "returned", there must be no "sold"
            if sold.is_some() {
                return Err(ErrorKind::AmbiguousAmounts(product.name.to_owned()).into());
            }
            if returned > offered {
                return Err(ErrorKind::TooMuchReturned(product.name.to_owned()).into());
            }
            offered - returned
        } else if let Some(sold) = sold {
            sold
        } else {
            offered
        };

        Ok(( BillItem{ amount: offered, product: product }, BillItem{ amount: sold, product: product }))
    }
}

impl Validatable for HasEmployees {
    fn validate(&self) -> SpecResult {
        let mut errors = ErrorList::new();
        if self.wages_date().is_none(){ errors.push("wages_date");}

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
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
            return Err(errors);
        }

        Ok(())
    }
}

pub mod events {
    use super::IsProject;
    use yaml_rust::Yaml;
    use chrono::{Date, UTC, NaiveTime};
    use icalendar::Event as CalEvent;
    use icalendar::{Component, Calendar};

    #[derive(Debug)]
    pub struct EventTime {
        pub start: NaiveTime,
        pub end:   NaiveTime
    }

    #[derive(Debug)]
    pub struct Event{
        pub begin: Date<UTC>,
        pub end: Option<Date<UTC>>,
        pub times: Vec<EventTime>
    }

    use std::fmt;
    impl fmt::Display for Event {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            try!(
                if let Some(end) = self.end { writeln!(f, "start: {}\nend:  {}", self.begin, end) }
                else { writeln!(f, "start: {}", self.begin) }
                );
            for time in &self.times {
                try!(
                    if time.start == time.end { writeln!(f, " * {}", time.start) }
                    else { writeln!(f, " * {} - {}", time.start, time.end) }
                    )
            }
            Ok(())
        }
    }

    fn naive_time_from_str(string:&str) -> Option<NaiveTime> {
        let t:Vec<u32> = string
            .splitn(2, |p| p == '.' || p == ':')
            .map(|s|s.parse().unwrap_or(0))
            .collect();

        if let (Some(h),m) = (t.get(0),t.get(1).unwrap_or(&0)){
            if *h < 24 && *m < 60 {
                return Some(NaiveTime::from_hms(*h,*m,0))
            }
        }

        None
    }

    #[test]
    fn test_naive_time_from_str() {
        assert_eq!(Some(NaiveTime::from_hms(9,15,0)), naive_time_from_str("9.15"));
        assert_eq!(Some(NaiveTime::from_hms(9,0,0)),  naive_time_from_str("9."));
        assert_eq!(Some(NaiveTime::from_hms(9,0,0)),  naive_time_from_str("9"));
        assert_eq!(Some(NaiveTime::from_hms(23,0,0)), naive_time_from_str("23.0"));
        assert_eq!(Some(NaiveTime::from_hms(23,59,0)), naive_time_from_str("23.59"));
        assert_eq!(None, naive_time_from_str("24.0"));
        assert_eq!(None, naive_time_from_str("25.0"));
        assert_eq!(None, naive_time_from_str("0.60"));

        assert_eq!(Some(NaiveTime::from_hms(9,15,0)), naive_time_from_str("9:15"));
        assert_eq!(Some(NaiveTime::from_hms(9,0,0)),  naive_time_from_str("9:"));
        assert_eq!(Some(NaiveTime::from_hms(9,0,0)),  naive_time_from_str("9"));
        assert_eq!(Some(NaiveTime::from_hms(23,0,0)), naive_time_from_str("23:0"));
    }

    pub trait HasEvents: IsProject {

        /// Produces an iCal calendar from this project.
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

        /// Produces a list of `DateRange`s for the event.
        fn events(&self) -> Option<Vec<Event>> {
            let dates = try_some!(self.get("event.dates/").and_then(|a| a.as_vec()));
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

                    Some( Event{
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
                        .and_then(naive_time_from_str);

                    let end   = self.get_direct(h, "end")
                        .and_then(|y|y.as_str())
                        .and_then(naive_time_from_str)
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
}

