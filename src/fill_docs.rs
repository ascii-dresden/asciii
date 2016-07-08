//! Fills export templates to create tex documents.
//!
//! Haven't decided on a templating engine yet, my own will probably not do.

#![allow(unused_variables, dead_code)]

use std::fmt;
use std::path::Path;
use std::error::Error;
use rustc_serialize::json::ToJson;
use handlebars::{RenderError,Handlebars,no_escape};

custom_derive! {
    #[derive(Debug,
             IterVariants(VirtualFields), IterVariantNames(VirtualFieldNames),
             EnumFromStr
             )]
pub enum Template{
    Document,
    Simple,
    Invalid
}
}
impl<'a> From<&'a str> for Template{
    fn from(s:&'a str) -> Template{
        s.parse::<Template>().unwrap_or(Template::Invalid)
    }
}

#[derive(Debug)]
pub enum FillError{
    RenderError(RenderError),
    InvalidTemplate
}
impl Error for FillError{
    fn description(&self) -> &str{
        match *self{
            FillError::RenderError(ref inner) => inner.description(),
            FillError::InvalidTemplate => "Invalid Template"
        }
    }

    fn cause(&self) -> Option<&Error>{
        match *self{
            FillError::RenderError(ref inner) => Some(inner),
            FillError::InvalidTemplate => None
        }
    }
}
impl fmt::Display for FillError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self.cause() {
            None => write!(f, "{}", self.description(),),
            Some(cause) => write!(f, "{}", cause)
        }
    }
}

//
// All you need to make try!() fun again
impl From<RenderError>  for FillError{
    fn from(he: RenderError) -> FillError{ FillError::RenderError(he) }
}

/// Takes a `T:ToJson` and a template path and does it's thing.
///
/// Returns path to created file, potenially in a `tempdir`.
//pub fn fill_template<E:ToJson>(document:E, template_file:&Path) -> PathBuf{
pub fn fill_template<E:ToJson>(document:&E, template:Template) -> Result<String, FillError>{

    let test_template = String::from(r#"\Name{{{name}}}"#);
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);

    handlebars.register_template_string("test", test_template).unwrap();
    handlebars.register_template_file("document", Path::new("./templates/document.tex.hbs")).unwrap();
    handlebars.register_template_file("simple", Path::new("./templates/simple.hbs")).unwrap();

    let rendered = match template{
        Template::Document => {
            handlebars.register_escape_fn(|data|data .replace("\n",r#"\newline"#));
            handlebars.render("document", document)
                .map(|r|r .replace("<","{") .replace(">","}"))
        },
        Template::Simple => handlebars.render("simple", document),
        Template::Invalid => return Err(FillError::InvalidTemplate)
    }.map_err(FillError::RenderError);

    println!("{:?}\n", document.to_json());
    rendered
}

/// Print config that lands in output files, for now
pub fn print_config(){
//Name                includes/name
//Strasse             includes/strasse
//Universitaet        includes/universitaet
//Fakultaet           includes/fakultaet
//Zusatz              includes/zusatz
//RetourAdresse       includes/retouradresse
//Ort                 includes/ort
//Land                includes/land
//Telefon             includes/telefon
//Telefax             includes/telefax
//Telex               includes/telex
//HTTP                includes/http
//EMail               includes/email
//Bank                includes/bank
//BLZ                 includes/blz
//IBAN                includes/iban
//BIC                 includes/bic
//Konto               includes/konto
//Steuernummer        includes/steuernummer
//Unterschrift        manager
//Adresse             client/address //].gsub("\n","\\newline ") %>}
//Betreff             messages][document_type][0]
//                    invoice/official]}" unless @data[:invoice/official].nil?  %>} %% [Angebot|Rechnung]
//Datum               ype/date
//AngebotManuel       offer/number
//Veranstaltung       event/name
//RechnungsNummer     invoice/longnumber //if type == :invoice %% bei Angeboten leer lassen
//Anrede              client/addressing
//Gruss               messages/signature //<%= @data[:signature] %>}{1cm}

}
