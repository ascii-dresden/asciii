extern crate asciii;

use std::error::Error;

use asciii::actions;
use asciii::storage::StorageDir;

fn main() {

    let dir = StorageDir::All;

    match actions::calendar(dir) {
        Ok(cal) => println!("{}", cal),
        Err(er) => println!("{}", er.description())
    }

}
