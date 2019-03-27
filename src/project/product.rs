//! Takes care of instantiating the Product.
//! All of the calculating is done by `extern crate bill`.

#![allow(missing_docs)]

use bill::{Currency, BillProduct, Tax};
use failure::Fail;

use crate::util::yaml;
use crate::util::to_currency;


//#[derive(Debug)] // manually implemented
/// Stores properties of a product.
///
/// Products are mapped to `Bill`s by `BillItems`,
/// these are implemented by `bill`.
#[derive(Copy,Clone,Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Product<'a> {
    pub name: &'a str,
    pub unit: Option<&'a str>,
    pub tax: Tax,
    pub price: Currency
}

pub type ProductResult<T> = ::std::result::Result<T, ProductError>;

#[derive(Fail, Debug)]
pub enum ProductError {
    #[fail( display="Invalid price in {}", _0)]
    InvalidPrice(String),

    #[fail(display = "unknown format")]
    UnknownFormat,
    #[fail(display = "more returned than provided")]
    AmbiguousAmounts(String),

    #[fail(display = "missing amount of {:?}", _0)]
    MissingAmount(String),
    
    #[fail(display = "too much returned of {:?}", _0)]
    TooMuchReturned(String),

    #[fail(display = "Cannot Parse Service")]
    InvalidServerSection 
}


impl<'a> Product<'a> {

    fn from_old_format<'y>( name: &'y str, values: &'y yaml::Yaml, local_tax: Option<Tax>) -> Result<Product<'y>, ProductError> {
        let default_tax = crate::CONFIG.get_f64("defaults/tax").map(Tax::new)
            .expect("Faulty config: field defaults/tax does not contain a value");

        let product_tax = yaml::get_f64(values, "tax").map(Tax::new);
        let tax = product_tax.or(local_tax).unwrap_or(default_tax);

        let unit = yaml::get_str(values, "unit");
        let price = yaml::get_f64(values, "price")
            .map(to_currency)
            .ok_or_else(||ProductError::InvalidPrice(name.to_string()))?;

        Ok(Product { name, unit, price, tax })
    }

    fn from_new_format<'y>(desc: &'y yaml::Yaml, values: &'y yaml::Yaml, local_tax: Option<Tax>) -> Result<Product<'y>, ProductError> {

        let default_tax = crate::CONFIG.get_f64("defaults/tax").map(Tax::new)
            .expect("Faulty config: field defaults/tax does not contain a value");

        let desc_tax = yaml::get_f64(desc, "tax").map(Tax::new);
        let values_tax = yaml::get_f64(values, "tax").map(Tax::new);
        let tax = values_tax.or(desc_tax).or(local_tax).unwrap_or(default_tax);

        let name = yaml::get_str(desc, "name").unwrap_or("unnamed");
        let price = yaml::get_f64(desc, "price")
                .ok_or_else(||ProductError::InvalidPrice(name.to_string()))
                .map(to_currency)?;
        let unit = yaml::get_str(desc, "unit");

        Ok(Product { name, unit, price, tax })
    }

    pub fn from_desc_and_value<'y>(desc: &'y yaml::Yaml, values: &'y yaml::Yaml, local_tax: Option<Tax>) -> Result<Product<'y>, ProductError> {
        match *desc {
            yaml::Yaml::String(ref name) => Self::from_old_format(name, values, local_tax),
            yaml::Yaml::Hash(_) => Self::from_new_format(desc, values, local_tax),
            _ => Err(ProductError::UnknownFormat.into()),
        }
    }
}

impl<'a> BillProduct for Product<'a>{
    fn price(&self) -> Currency {self.price}
    fn name(&self) -> String {self.name.to_owned()}
    fn tax(&self) -> Tax {self.tax}
}

