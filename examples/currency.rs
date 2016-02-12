extern crate currency;
use currency::Currency;

fn main(){
    println!("{}", Currency(Some('€'), 0));
}
