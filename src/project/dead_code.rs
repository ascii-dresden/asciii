// product/project stuff
enum ProductUnit {
    Piece, Liter, Hour, Kilogramm, Gramm
}

struct Product {
    pub name: String,
    pub unit: ProductUnit,
    pub tax: f64,
    pub price: f64 //TODO make this a currency
}

impl Product {
    fn cost_before_tax()
       // -> f64
    {}

    fn cost_after_tax()
       // -> f64
    {}
}

struct InvoiceItem {
    pub amount_offered: usize,
    pub amount_sold: usize,
    pub item: Product
}

struct Customer {
    pub first_name: String,
    pub last_name: String,
    pub email: String, // TODO replace with e.g. `crate:emailaddress`
}

struct Event {
    pub start:DateTime<UTC>,
    pub end:DateTime<UTC>,
}

