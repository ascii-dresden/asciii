extern crate asciii;

use std::error::Error;
use asciii::calendar;
use asciii::storage::StorageDir;


fn main() {

    let dir = StorageDir::All;

    println!("{}", calendar(dir).unwrap())
}
