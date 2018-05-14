
use storage::{self, Storable};
use util;

use super::Project;
use super::spec::*;

/// Fields that are accessible but are not directly found in the file format.
/// This is used to get fields that are computed through an ordinary `field("responsible")`
custom_derive! {
#[allow(missing_docs)]
    #[derive(Debug,
             IterVariants(ComputedFields), IterVariantNames(ComputedFieldNames),
             EnumFromStr
             )]
    /// `Project::field()` allows accessing fields within the raw `yaml` data structure.
    /// Computed fields are fields that are not present in the document but computed.
    ///
    /// `ComputedFields` is an automatically generated type that allows iterating of the variants of
    /// this Enum.
    pub enum ComputedField {
        /// Usually `storage`, or in legacy part of `signature`
        Responsible,
        /// Consecutive Offer number
        OfferNumber,
        /// Pretty version of `invoice/number`: "`R042`"
        InvoiceNumber,
        /// Pretty version of `invoice/number` including year: "`R2016-042`"
        InvoiceNumberLong,
        /// Name
        Name,
        /// Amount of money owed by the customer
        Final,
        /// Age of the Project in days
        Age,
        /// Time in weeks it took to write the invoice
        OurBad,
        /// Time in weeks it took the custumer to pay invoice
        TheirBad,
        /// Year of the event
        Year,
        /// List of emplyees
        Employees,
        /// Full Name of the customer
        ClientFullName,
        /// Sum of the wages payed to the employees
        Wages,
        /// *experimental* indicates whether the project file adheres to the spec
        Deserializes,
        /// Sorting index
        SortIndex,
        /// Date of the main event
        Date,
        /// Version of the project file format
        Format,
        /// Directory the project is currently stored in
        Dir,
        /// Invalid Option
        Invalid
    }
}

impl<'a> From<&'a str> for ComputedField {
    fn from(s: &'a str) -> ComputedField {
        s.parse::<ComputedField>().unwrap_or(ComputedField::Invalid)
    }
}

impl ComputedField {
    pub fn get(&self, project: &Project) -> Option<String> {
        let storage = storage::get_storage_path();

        match *self {
            ComputedField::Responsible => project.responsible().map(|s| s.to_owned()),
            ComputedField::OfferNumber => project.offer().number(),
            ComputedField::InvoiceNumber => project.invoice().number_str(),
            ComputedField::InvoiceNumberLong => project.invoice().number_long_str(),
            ComputedField::Name => {
                Some(project.name()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| project.file_name()))
            } // TODO remove name() from `Storable`, storables only need a slug()
            ComputedField::Final => {
                project.sum_sold()
                       .map(|c| util::currency_to_string(&c))
                       .ok()
            }
            ComputedField::Age => project.age().map(|a| lformat!("{} days", a)),

            ComputedField::OurBad => {
                project.our_bad()
                       .map(|a| lformat!("{} weeks", a.num_weeks().abs()))
            }
            ComputedField::TheirBad => {
                project.their_bad()
                       .map(|a| lformat!("{} weeks", a.num_weeks().abs()))
            }

            ComputedField::Year => project.year().map(|i| i.to_string()),
            ComputedField::Date => {
                project.modified_date()
                       .map(|d| d.format("%Y.%m.%d").to_string())
            }
            ComputedField::SortIndex => project.index(),

            ComputedField::Employees => project.hours().employees_string(),
            ComputedField::ClientFullName => project.client().full_name(),
            ComputedField::Deserializes => Some(format!("{:?}", project.parse_yaml().is_ok())),
            ComputedField::Wages => {
                project.hours()
                       .gross_wages()
                       .map(|c| util::currency_to_string(&c))
            }
            ComputedField::Format => project.format().map(|f| f.to_string()),
            ComputedField::Dir => {
                project.dir()
                       .parent()
                       .and_then(|d| d.strip_prefix(&storage).ok())
                       .map(|d| d.display().to_string())
            }
            ComputedField::Invalid => None,

            // _ => None
        }
    }
}
