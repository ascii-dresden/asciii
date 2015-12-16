pub enum ProductUnit {
    Piece, Liter, Hour, Kilogramm, Gramm, None
}

#[derive(Debug)]
pub struct Product<'a> {
    pub name: &'a str,
    pub unit: Option<&'a str>,
    pub tax: f64,
    pub price: f64 //TODO make this a currency
}

impl<'a> Product<'a> {
    fn cost_before_tax()
       // -> f64
    {}

    fn cost_after_tax()
       // -> f64
    {}
}

#[derive(Debug)]
pub struct InvoiceItem<'a> {
    pub amount_offered: f64,
    pub amount_sold: f64,
    pub item: Product<'a>
}
