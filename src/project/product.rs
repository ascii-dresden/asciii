use std::fmt;
use currency::Currency;

use util::yaml;
use util::yaml::Yaml;

use super::{ProductResult, ProductError};
use super::spec::to_currency;

#[deprecated]
const DEFAULT_TAX:f64 = 0.19;// TODO read from config

pub enum ProductUnit {
    Piece, Liter, Hour, Kilogramm, Gramm, None
}

//#[derive(Debug)] // manually implemented
pub struct Product<'a> {
    pub name: &'a str,
    pub unit: Option<&'a str>,
    pub tax: f64,
    pub price: Currency
}

#[derive(Debug)]
pub struct InvoiceItem<'a> {
    pub amount_offered: f64,
    pub amount_sold: f64,
    pub item: Product<'a>
}

impl<'a> Product<'a>{

    pub fn from_old_format<'y>(name:&'y str, values: &'y Yaml) -> ProductResult<Product<'y>> {
        Ok(Product{
            name:  name,
            unit:  yaml::get_str(values, "unit"),
            price: try!(yaml::get_f64(values, "price")
                .ok_or(ProductError::InvalidPrice)
                .and_then(to_currency)),
            tax:   yaml::get_f64(values, "tax").unwrap_or(DEFAULT_TAX),
        })
    }

    pub fn from_new_format<'y>(desc: &'y Yaml) -> ProductResult<Product<'y>> {
        Ok(Product{
            name:  yaml::get_str(desc, "name").unwrap_or("unnamed"),
            unit:  yaml::get_str(desc, "unit"),
            price: try!(yaml::get_f64(desc, "price")
                .ok_or(ProductError::InvalidPrice)
                .and_then(to_currency)),
            tax:   yaml::get_f64(desc, "tax").unwrap_or(DEFAULT_TAX),
        })
    }

    pub fn from_desc_and_value<'y>(desc: &'y Yaml, values: &'y Yaml) -> ProductResult<Product<'y>> {
        match *desc {
            yaml::Yaml::String(ref name) => Self::from_old_format(name,values),
            yaml::Yaml::Hash(_)          => Self::from_new_format(desc),
            _                            => Err(ProductError::UnknownFormat)
        }
    }

}

impl<'a> InvoiceItem<'a> {
    pub fn cost_before_tax(&self) -> Currency {
        self.amount_sold * &self.item.price
    }

    pub fn cost_after_tax(&self) -> Currency {
        self.amount_sold * &self.item.price * (1.0 + self.item.tax)
    }

    pub fn from_desc_and_value<'y>(desc: &'y Yaml, values: &'y Yaml) -> ProductResult<InvoiceItem<'y>> {
        let product = try!(Product::from_desc_and_value(desc,values));

        let offered = try!(yaml::get_f64(values, "amount").ok_or(ProductError::MissingAmount(product.name.to_owned())));
        let sold = yaml::get_f64(values, "sold");
        let sold =
            if let Some(returned) = yaml::get_f64(values, "returned"){
                // if "returned", there must be no "sold"
                if sold.is_some() {return Err(ProductError::AmbiguousAmounts(product.name.to_owned()));}
                if returned > offered {return Err(ProductError::TooMuchReturned(product.name.to_owned()));}
                offered - returned
            } else if let Some(sold) = sold {
                sold
            } else {
                offered
            };

        Ok(InvoiceItem {
            amount_offered: offered,
            amount_sold: sold,
            item: product
        })
    }
}





impl<'a> fmt::Debug for Product<'a>{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.price)
    }
}
impl<'a> fmt::Display for Product<'a>{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}), tax: {} unit: {:?}", self.name, self.price, self.tax, self.unit)
    }
}

