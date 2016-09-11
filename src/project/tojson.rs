use rustc_serialize::json::{ToJson, Json};
use chrono::*;

use std::collections::BTreeMap;

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

        let map = btreemap!{
            //String::from("adressing") => ,
            s("client") => btreemap!{
                s("email")      => opt_str(client::email(y)),
                s("last_name")  => opt_str(client::last_name(y)),
                s("first_name") => opt_str(client::first_name(y)),
                s("full_name")  => client::full_name(y).to_json(),
                s("title")      => opt_str(client::title(y)),
                s("address")    => opt_str(client::address(y)),
                s("addressing") => client::addressing(y).to_json(),
            }.to_json(),

            ("bills") =>  btreemap!{
                s("offer") => Json::Null,
                s("invoice") => Json::Null
            }.to_json(),


            s("offer") => btreemap!{
                s("number") => offer::number(y).to_json(),
                s("date")   => dmy(spec::date::offer(y)),
                s("sum")   => Json::I64(-1), // TODO per tax please
            }.to_json(),

            s("event") => btreemap!{
                s("name")    => self.name().to_json(),
                s("date")    => dmy(project::date(y)),
                s("manager") => self.manager().to_json(),
            }.to_json(),

            s("invoice") => btreemap!{
                s("date")   => dmy(spec::date::invoice(y)),
                s("number")      => invoice::number_str(y).to_json(),
                s("number_long") => invoice::number_long_str(y).to_json(),
                s("official") => invoice::official(y).to_json(),
                s("sum")   => Json::I64(-1),
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

