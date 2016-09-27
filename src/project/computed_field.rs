use chrono::Datelike;

use util::currency_to_string;
use storage::Storable;

use super::spec::*;
use super::Project;

/// Fields that are accessible but are not directly found in the file format.
/// This is used to get fields that are computed through an ordinary `get("responsible")`
custom_derive! {
    #[derive(Debug,
             IterVariants(ComputedFields), IterVariantNames(ComputedFieldNames),
             EnumFromStr
             )]
    /// `Project::get()` allows accessing fields within the raw `yaml` data structure.
    /// Computed fields are fields that are not present in the document but computed.
    ///
    /// `ComputedFields` is an automatically generated type that allows iterating of the variants of
    /// this Enum.
    pub enum ComputedField{
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

impl<'a> From<&'a str> for ComputedField {
    fn from(s: &'a str) -> ComputedField {
        s.parse::<ComputedField>().unwrap_or(ComputedField::Invalid)
    }
}

impl ComputedField {
    pub fn get(&self, project: &Project) -> Option<String> {
        match *self {
            ComputedField::Responsible => project::manager(project.yaml()).map(|s| s.to_owned()),
            ComputedField::OfferNumber => offer::number(project.yaml()),
            ComputedField::InvoiceNumber => invoice::number_str(project.yaml()),
            ComputedField::InvoiceNumberLong => invoice::number_long_str(project.yaml()),
            ComputedField::Name => project::name(project.yaml()).map(|s| s.to_owned()),
            ComputedField::Final => project.sum_sold().map(|c| currency_to_string(&c)).ok(),
            ComputedField::Age => project.age().map(|a| format!("{} days", a)),
            ComputedField::Year => project.date().map(|d| d.year().to_string()),

            ComputedField::Caterers => hours::caterers_string(project.yaml()),
            ComputedField::ClientFullName => client::full_name(project.yaml()),
            ComputedField::Invalid => None,

            // _ => None
        }
    }
}

