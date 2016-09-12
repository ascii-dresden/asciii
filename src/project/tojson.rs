use rustc_serialize::json::{ToJson, Json};
use chrono::*;
use bill::{Bill, BillItem};
use ordered_float::OrderedFloat;

use super::Project;
use super::product::Product;
use super::spec;
use ::storage::Storable;

impl ToJson for Project{
    fn to_json(&self) -> Json{
        use ::project::spec::*;

        let s = |s:&str| String::from(s);

        let opt_str = |opt:Option<&str>| opt.map(|e|e.to_owned()).to_json() ;
        let y = &self.yaml;
        let dmy = |date:Option<Date<UTC>>| date.map(|d|d.format("%d.%m.%Y").to_string()).to_json();

        let item_to_json = |item:&BillItem<Product>, tax:OrderedFloat<f64>| btreemap!{
            s("name") => item.product.name.to_json(),
            s("price") => item.product.price.to_string().to_json(),
            s("unit") => item.product.unit.unwrap_or_else(||"").to_json(),
            s("amount") => item.amount.to_json(),
            s("cost") => item.sum().to_string().to_json(),
            s("tax") => tax.into_inner().to_json()
        }.to_json();

        let bill_to_json = |bill:&Bill<Product>| bill.as_items_with_tax().into_iter()
                                                                     .map(|(tax, item)| item_to_json(item,tax) )
                                                                     .collect::<Vec<Json>>()
                                                                     .to_json();

        let taxes_by_tax_to_json = |bill:&Bill<Product>| bill.taxes_by_tax().iter()
                                                                            .map(|(tax,taxes)| btreemap!{
                                                                                s("tax") => (tax.into_inner()*100.0).to_json(),
                                                                                s("taxes") => taxes.to_json(),
                                                                            }.to_json())
                                                                            .collect::<Vec<Json>>()
                                                                            .to_json();

        let (offer, invoice) = self.bills().unwrap();

        let map = btreemap!{
            //String::from("adressing") => ,

            s("bills") =>  btreemap!{
                s("offer") => bill_to_json(&offer),
                s("invoice") => bill_to_json(&invoice),
            }.to_json(),

            s("client") => btreemap!{
                s("email")      => opt_str(client::email(y)),
                s("last_name")  => opt_str(client::last_name(y)),
                s("first_name") => opt_str(client::first_name(y)),
                s("full_name")  => client::full_name(y).to_json(),
                s("title")      => opt_str(client::title(y)),
                s("address")    => opt_str(client::address(y)),
                s("addressing") => client::addressing(y).to_json(),
            }.to_json(),


            s("event") => btreemap!{
                s("name")    => self.name().to_json(),
                s("date")    => dmy(project::date(y)),
                s("manager") => self.manager().to_json(),
            }.to_json(),


            s("offer") => btreemap!{
                s("number") => offer::number(y).to_json(),
                s("date")   => dmy(spec::date::offer(y)),
                s("sums")   => taxes_by_tax_to_json(&offer),
                s("total")  => offer.total().to_json(),
                s("total_before_tax")  => offer.total_before_tax().to_json(),
            }.to_json(),

            s("invoice") => btreemap!{
                s("date")   => dmy(spec::date::invoice(y)),
                s("number")      => invoice::number_str(y).to_json(),
                s("number_long") => invoice::number_long_str(y).to_json(),
                s("official") => invoice::official(y).to_json(),
                s("sums")   => taxes_by_tax_to_json(&invoice),
                s("total")  => invoice.total().to_json(),
                s("total_before_tax")  => invoice.total_before_tax().to_json(),
            }.to_json(),

            s("hours") => btreemap!{
                s("time")   => hours::total(y),
            }.to_json(),

        };
        Json::Object(map)
    }
}

impl<'a> ToJson for Product<'a> {
    fn to_json(&self) -> Json {
        let s = |s: &str| String::from(s);
        Json::Object(btreemap!{
            s("name")     => self.name.to_json(),
            s("unit")     => self.unit.map(|s|s.to_owned()).to_json(),
            s("tax")      => self.tax.to_string().to_json(),
            s("price")    => self.price.to_string().to_json(),
            s("currency") => self.price.0.map(|s|s.to_string()).to_json(),
        })
    }
}

