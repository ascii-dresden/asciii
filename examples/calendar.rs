extern crate asciii;

use asciii::actions::calendar_and_tasks as calendar;
use asciii::storage::StorageDir;


fn main() {

    let dir = StorageDir::All;

    println!("{}", calendar(dir).unwrap())
}
