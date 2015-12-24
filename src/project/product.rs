use std::fmt;
use currency::Currency;

pub enum ProductUnit {
    Piece, Liter, Hour, Kilogramm, Gramm, None
}

//#[derive(Debug)]
pub struct Product<'a> {
    pub name: &'a str,
    pub unit: Option<&'a str>,
    pub tax: f64,
    pub price: Currency
}

impl<'a> InvoiceItem<'a> {
    pub fn cost_before_tax(&self) -> Currency {
        self.amount_sold * self.item.price
    }

    pub fn cost_after_tax(&self) -> Currency {
        self.amount_sold * self.item.price * (1.0 + self.item.tax)
    }
}

#[derive(Debug)]
pub struct InvoiceItem<'a> {
    pub amount_offered: f64,
    pub amount_sold: f64,
    pub item: Product<'a>
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
