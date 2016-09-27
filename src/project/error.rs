#![allow(trivial_casts)]

use std::io;
use std::fmt;

pub mod product{
    error_chain!{
        types { }
        links { }
        foreign_links { }
        errors {
            InvalidPrice {}
            UnknownFormat {}
            AmbiguousAmounts(t:String){
                description("more returned than provided")
            }
            MissingAmount(t:String){
                description("invalid price")
            }
            TooMuchReturned(t:String){
                description("invalid format")
            }
        }
    }
}

error_chain!{
    types {
        Error, ErrorKind, ChainErr, Result;
    }

    links {
        self::product::Error, self::product::ErrorKind, Product;
    }

    foreign_links {
        io::Error, Io;
        fmt::Error, Fmt;
    }

    errors {
        CantDetermineTargetFile{
            description("Cannot determine target file name")
        }
    }
}

