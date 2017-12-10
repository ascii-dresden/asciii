//! Implements the ascii invoicer project file specification.
//!
//! This does all of the heavy lifting.
//! The implementation is separated into sub-modules which take care of separate objectives.
//! Most of the functions in these modules take the `yaml` data directly as reference.
//! Each module contains a `validate()` function which ought to be kept up to date.

use std::fmt;

use bill::{Bill, Currency, Tax};
use chrono::{Date, Utc, NaiveTime};
use icalendar::Calendar;
use semver::Version;
use yaml_rust::Yaml;

use storage::Storable;
use super::error::{SpecResult, ErrorList};
use super::product::Product;
use super::product::error::Result as ProductResult;

pub fn print_specresult(label: &str, result: &SpecResult) {
match *result {
        Ok(_) => println!("{}: ✓", label),
        Err(ref errs) => println!("{}: ✗\n{}", label, errs)
    }
}


/// Every other trait in this module ought to be `Validatable`
pub trait Validatable {
    /// Checks for certain errors
    fn validate(&self) -> SpecResult;

    /// Returns true if valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Returns list of found errors
    fn errors(&self) -> Option<ErrorList>{
        self.validate().err()
    }
}

/// Stage 0: the Project itself
///
/// Provide the basics every Project should have.
pub trait IsProject {

    /// Project Name
    fn name(&self) -> Option<&str>;

    /// When was the event
    fn event_date(&self) -> Option<Date<Utc>>;

    /// Version of project format
    fn format(&self) -> Option<Version>;

    /// Did the event actually occur
    fn canceled(&self) -> bool;

    /// Who organized the event
    fn responsible(&self) -> Option<&str>;

    /// Long description of the project
    fn long_desc(&self) -> String;
}

/// Extended functionality for projects
pub trait IsProjectExt {
    /// Number of days since creation of the project
    fn age(&self) -> Option<i64>;
}

impl<T> IsProjectExt for T where T: Storable {
    fn age(&self) -> Option<i64> {
        self.modified_date()
            .map(|date|
                 (Utc::today().signed_duration_since(date))
                              .num_days()
                )
    }
}

/// Stage 1: requirements for an offer
pub trait Offerable {
    /// Raised if the offer number is reused
    fn appendix(&self) -> Option<i64>;

    /// When was the offer created
    fn date(&self) -> Option<Date<Utc>>;

    /// ID of an the offer
    fn number(&self) -> Option<String>;
}

/// Everything about the client
///
/// This is a [client](../struct.Project.html#method.client)
pub trait IsClient {
    ///Returns the content of `/client/email`
    fn email(&self) -> Option<&str>;

    ///Returns the content of `/client/address`
    fn address(&self) -> Option<&str>;

    ///Returns the content of `/client/title`
    fn title(&self) -> Option<&str>;

    ///Returns the first word of `client/title`
    fn salute(&self) -> Option<&str>;

    ///Returns the content of `/client/first_name`
    fn first_name(&self) -> Option<&str>;

    ///Returns the content of `/client/last_name`
    fn last_name(&self) -> Option<&str>;

    /// Combines `first_name` and `last_name`.
    fn full_name(&self) -> Option<String>;

    /// Produces a standard salutation field.
    fn addressing(&self) -> Option<String>;
}

/// Stage 2: requirements for an invoice
pub trait Invoicable {
    /// plain access to `invoice/number`
    fn number(&self) -> Option<i64>;

    /// When was the invoice created
    fn date(&self) -> Option<Date<Utc>>;

    /// invoice number as a string
    fn number_str(&self) -> Option<String>;

    /// invoice number as a long string
    fn number_long_str(&self) -> Option<String>;

    /// An official identifier
    fn official(&self) -> Option<String>;
}

/// Represents an Employee
pub struct Employee {
    /// Name of the Employee
    pub name: String,

    /// Amount of Currency the employees receives per hour
    pub salary: Currency,

    /// Number of hours the employee worked on this project
    pub time: f64,

    /// Salary times hours
    pub wage: Currency,
}

/// Something that has employees
pub trait HasEmployees {
    /// When were the wages payed
    fn wages_date(&self) -> Option<Date<Utc>>;

    /// Salary
    fn salary(&self) -> Option<Currency>;

    /// Tax
    fn tax(&self) -> Option<Tax>;

    /// Sum of wages after tax
    fn net_wages(&self) -> Option<Currency> ;

    /// Sum of wages before tax
    fn gross_wages(&self) -> Option<Currency> ;


    /// Full number of service hours
    ///
    /// `TODO` test this against old format
    fn total_time(&self) -> Option<f64>;

    /// Returns a product from Service
    fn to_product(&self) -> Option<Product> {
        if let Some(salary) = self.salary() {
            Some(Product {
                name: "Service",
                unit: Some("h"),
                tax: self.tax().unwrap_or_else(|| Tax::new(0.0)),
                price: salary
            })
        } else {
            None
        }
    }

    /// Nicely formatted list of employees with their respective service hours
    fn employees_string(&self) -> Option<String>;

    /// List of employees and their respective service hours
    fn employees(&self) -> Option<Vec<Employee>>;

    /// Check if the employees have been payed
    fn employees_payed(&self) -> bool;

    /// Sum of wages for the project
    fn wages(&self) -> Option<Currency>;
}


/// Stage 3: when an `IsProject` is redeem and can be archived
pub trait Redeemable: IsProject {

    /// When was the project payed
    fn payed_date(&self) -> Option<Date<Utc>>;

    /// If was the project payed
    fn is_payed(&self) -> bool;

    /// Returns a bill for the offer and one for the invoice.
    fn bills(&self) -> ProductResult<(Bill<Product>, Bill<Product>)>;

    /// When what is the MWsT of the project.
    fn tax(&self) -> Option<Tax>;

    /// Sum of sold products
    fn sum_sold(&self) -> ProductResult<Currency> {
        let (_,invoice) = self.bills()?;
        Ok(invoice.net_total())
    }

}

/// Holds the time of the beginning and end of an event
#[derive(Debug)]
pub struct EventTime {
    /// Start of the event
    pub start: NaiveTime,

    /// End of the event
    pub end:   NaiveTime
}

/// Describes either the coarse begin and end date of the event
/// or holds a list of distinct `EventTime`s
#[derive(Debug)]
pub struct Event {
    /// Begin of the event
    pub begin: Date<Utc>,

    /// End of the event
    pub end: Option<Date<Utc>>,

    /// Set of of times
    pub times: Vec<EventTime>
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(end) = self.end { writeln!(f, "start: {}\nend:  {}", self.begin, end) }
        else { writeln!(f, "start: {}", self.begin) }?;
        for time in &self.times {
            if time.start == time.end { writeln!(f, " * {}", time.start) }
            else { writeln!(f, " * {} - {}", time.start, time.end) } ?
        }
        Ok(())
    }
}

/// Something that has events
pub trait HasEvents {
    /// Produces an iCal calendar from this project.
    fn to_ical(&self) -> Calendar;

    /// Produces a list of `DateRange`s for the event.
    fn events(&self) -> Option<Vec<Event>>;

    /// Returns a list of `Event`s
    fn times(&self,yaml: &Yaml) -> Option<Vec<EventTime>>;

    /// Returns the location of the event
    fn location(&self) -> Option<&str>;

}
