#![allow(missing_docs)]

use failure::Fail;

use super::yaml_provider::FieldValue;

#[derive(Fail, Debug)]
pub enum ProjectError {

    #[fail(display="This feature is not enabled in this build")]
    FeatureDeactivated,

    #[fail(display="Cannot determine target file name")]
    CantDetermineTargetFile,
}


/// Result of validating part of a project.
///
/// We have to differentiate between incomplete data (missing values) and wrong data (invalid values).
/// Wrong data should always be a hard error - there is no reason to have invalid values in project files.
///
/// Missing data is not an error, it simply means that some information is not available yet.
/// An example is the field for the payment date: this field is missing until the invoice has been paid,
/// since the date is unknown until that point.
#[derive(Eq, PartialEq, Debug, Default)]
pub struct ValidationResult {
    /// hard error messages (invalid data)
    pub validation_errors: Vec<String>,

    /// soft error messages (incomplete data)
    pub missing_fields: Vec<String>
}

impl ValidationResult {
    pub fn new() -> Self {
        ValidationResult {
            validation_errors: Vec::new(),
            missing_fields: Vec::new(),
        }
    }

    pub fn is_ok(&self) -> bool {
        return self.validation_errors.is_empty() && self.missing_fields.is_empty()
    }

    pub fn validate_field<T>(&mut self, name: &str, val: FieldValue<T>) {
        if let FieldValue::Invalid(msg) = val {
            self.validation_errors.push(lformat!("field {} has invalid value: {}", name, msg).to_string());
        }
    }

    pub fn require_option<T>(&mut self, name: &str, val: Option<T>) {
        if val.is_none() {
            self.missing_fields.push(name.to_string())
        }
    }

    pub fn require_field<T>(&mut self, name: &str, val: FieldValue<T>) {
        if !val.is_ok() {
            self.missing_fields.push(name.to_string())
        }

        if let FieldValue::Invalid(msg) = val {
            self.validation_errors.push(lformat!("field {} has invalid value: {}", name, msg).to_string());
        }
    }

    pub fn and(mut self, next: ValidationResult) -> ValidationResult {
        self.missing_fields.extend(next.missing_fields);
        self.validation_errors.extend(next.validation_errors);
        self
    }

}

