//! Takes care of instantiating the Product.
//! All of the calculating is done by `extern crate bill`.

use bill::{Currency, BillProduct, Tax};

use util::yaml;
use util::yaml::Yaml;

use super::spec::to_currency;
pub mod error{
    #![allow(trivial_casts)]
    error_chain!{
        types { }
        links { }
        foreign_links { }
        errors {
            InvalidPrice (product:String){
                description("A product has either no or an invalid price.")
                display("Invalid price in {}", product)
            }

            UnknownFormat {}
            AmbiguousAmounts(t:String){
                description("more returned than provided")
            }
            MissingAmount(t:String){
                description("invalid price")
            }
            TooMuchReturned(t:String){
                description("invalid format")
            }
        }
    }
}

pub use self::error::*;

//#[derive(Debug)] // manually implemented
/// Stores properties of a product.
///
/// Products are mapped to `Bill`s by `BillItems`,
/// these are implemented by `bill`.
#[derive(Copy,Clone,Debug)]
pub struct Product<'a> {
    pub name: &'a str,
    pub unit: Option<&'a str>,
    pub tax: Tax,
    pub price: Currency
}


impl<'a> Product<'a>{
    pub fn from_old_format<'y>(name: &'y str, values: &'y Yaml) -> Result<Product<'y>> {
        let default_tax = ::CONFIG.get_f64("defaults/tax")
            .expect("Faulty config: field defaults/tax does not contain a value");
        Ok(Product {
            name: name,
            unit: yaml::get_str(values, "unit"),
            price: try!(yaml::get_f64(values, "price")
                .map(to_currency)
                .ok_or_else(||Error::from(ErrorKind::InvalidPrice(name.to_string())))
                ),
            tax: yaml::get_f64(values, "tax").unwrap_or(default_tax).into(),
        })
    }

    pub fn from_new_format(desc: &Yaml) -> Result<Product> {
        //TODO read default tax from document
        let default_tax = ::CONFIG.get_f64("defaults/tax")
            .expect("Faulty config: field defaults/tax does not contain a value");
        let name = yaml::get_str(desc, "name").unwrap_or("unnamed");
        Ok(Product {
            name: name,
            unit: yaml::get_str(desc, "unit"),
            price: try!(yaml::get_f64(desc, "price")
                .ok_or_else(||Error::from(ErrorKind::InvalidPrice(name.to_string())))
                .map(to_currency)),
            tax: yaml::get_f64(desc, "tax").unwrap_or(default_tax).into(),
        })
    }

    pub fn from_desc_and_value<'y>(desc: &'y Yaml, values: &'y Yaml) -> Result<Product<'y>> {
        match *desc {
            yaml::Yaml::String(ref name) => Self::from_old_format(name, values),
            yaml::Yaml::Hash(_) => Self::from_new_format(desc),
            _ => Err(ErrorKind::UnknownFormat.into()),
        }
    }
}

impl<'a> BillProduct for Product<'a>{
    fn price(&self) -> Currency {self.price}
    fn name(&self) -> String {self.name.to_owned()}
    fn tax(&self) -> Tax {self.tax}
}

