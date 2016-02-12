extern crate chrono;
use chrono::*;

fn main (){

    let today = Local::today();
    let other = UTC.ymd(2014, 11, 28);


    println!("{:?}", (today-other).num_days());
}
