#![allow(trivial_casts)]

use std::io;
use std::fmt;
use util::yaml;

use super::product;

error_chain!{
    types {
        Error, ErrorKind, ChainErr, Result;
    }

    links {
        product::Error, product::ErrorKind, Product;
    }

    foreign_links {
        io::Error, Io;
        fmt::Error, Fmt;
        yaml::YamlError, Yaml;
    }

    errors {
        CantDetermineTargetFile{
            description("Cannot determine target file name")
        }
    }
}

