#![allow(dead_code)]

use yaml_rust::Yaml;

//fn unpack_path<'a>(yaml:&'a Yaml, path:&Vec<&str>) ->Result<&'a Yaml, String>
//{
//    for key in path
//}

fn unpack<'a>(yaml:&'a Yaml, key:&str) ->Result<&'a Yaml, String> {
    if let Yaml::BadValue = yaml[key]{
        Err(format!("Nothing found {:?}", key))
    }
    else {
        Ok(&yaml[key])
    }
}

pub mod date {
    use chrono::*;
    use regex::Regex;
    use yaml_rust::Yaml;

    pub fn payed(yaml:&Yaml) -> Result<Date<UTC>, String>
    {
        let payed = try!(super::unpack(yaml, "invoice"));
        let date  = try!(super::unpack(payed, "payed_date"));
        let dmy   = try!(date.as_str().ok_or("cannot parse as string"));
        parse_dmy(dmy)
    }

    pub fn invoice(yaml:&Yaml) -> Result<Date<UTC>, String>
    {
        let invoice = try!(super::unpack(yaml, "invoice"));
        let date    = try!(super::unpack(invoice, "date"));
        let dmy     = try!(date.as_str().ok_or("cannot parse as string"));
        parse_dmy(dmy)
    }

    pub fn offer(yaml:&Yaml) -> Result<Date<UTC>, String>
    {
        let offer = try!(super::unpack(yaml, "offer"));
        let date  = try!(super::unpack(offer, "date"));
        let dmy   = try!(date.as_str().ok_or("cannot parse as string"));
        parse_dmy(dmy)
    }

    pub fn created(yaml:&Yaml) -> Result<Date<UTC>, String>
    {
        let created = try!(super::unpack(yaml, "created"));
        let dmy     = try!(created.as_str().ok_or("cannot parse as string"));
        parse_dmy(dmy)
    }

    fn parse_dmy(dmy:&str) -> Result<Date<UTC>, String>
    {
        let re = Regex::new(r"(?x)^(\d{2}).?(\d{2}).?(\d{4})$").unwrap();

        if let Some(caps) = re.captures(dmy)
        {
            return match UTC.ymd_opt(
                caps.at(3).unwrap().parse().unwrap(), // year
                caps.at(2).unwrap().parse().unwrap(), // month
                caps.at(1).unwrap().parse().unwrap(), // day
                )
            {
                LocalResult::Single(date) => Ok(date),
                _ => Err(format!("ambiguous date: {:?}", dmy))
            };
        }
        else {
            return Err(format!("wrong format {:?}, expecting DD.MM.YYYY", dmy));
        }
    }
}
