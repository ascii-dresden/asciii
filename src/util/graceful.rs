// TODO: This has "crate potential"
use std::process;
use std::error::Error;

pub trait GracefulExit<T>{
    fn graceful(self, message:&str) -> T;
}

impl<T> GracefulExit<T> for Option<T> {
    fn graceful(self, message:&str) -> T{
        match self{
            Some(val) => val,
            None => {
                println!("Error: {}", message);
                process::exit(1);
            }
        }
    }
}

impl<T, E> GracefulExit<T> for Result<T, E> where E:Error{
    fn graceful(self, message:&str) -> T{
        match self{
            Ok(val) => val,
            Err(e)=> {
                println!("Error: {}", message);
                println!("{}", e.description());
                println!("{:?}", e.cause());
                process::exit(1);
            }
        }
    }
}
