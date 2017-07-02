extern crate asciii;

use std::error::Error;
use asciii::actions::calendar;
use asciii::storage::StorageDir;


fn main() {

    let dir = StorageDir::All;

    println!("{}", calendar(dir).unwrap())
}
