
use util::yaml;
use currency::Currency;

use super::ProductError;
use super::spec;





static CLIENT_TEST_DOC:&'static str =
r#"
client:
  title:      Herr # Frau, Professor, Professorin
  first_name: Graf
  last_name:  Zahl

  email: this.man@example.com
  address: |
    Graf Zahl
    Nummernhöllenstraße 666
    01234 Countilvania
"#;

#[test]
fn validate_stage1(){
    let doc = yaml::parse(CLIENT_TEST_DOC).unwrap();
    assert!(spec::client::validate(&doc).is_ok());
}






static OFFER_TEST_DOC:&'static str =
r#"
offer:
  date: 07.11.2014
  appendix: 1
manager: somebody
"#;

#[test]
fn validate_stage2(){
    let doc = yaml::parse(OFFER_TEST_DOC).unwrap();
    let errors = spec::offer::validate(&doc);
    println!("{:#?}", errors);
    assert!(errors.is_ok());
}


static INVOICE_TEST_DOC:&'static str =
r#"
invoice:
  number: 41
  date: 06.12.2014
  payed_date: 08.12.2014
"#;

#[test]
fn validate_stage3(){
    let doc = yaml::parse(INVOICE_TEST_DOC).unwrap();
    let errors = spec::invoice::validate(&doc);
    println!("{:#?}", errors);
    assert!(errors.is_ok());
}





static PRODUCT_TEST_DOC_VALID:&'static str =
r#"
---
cataloge:
  product: &coffee    { name: Kaffee, price: 2.5, unit: 1l  }
  product: &tea       { name: Tee,    price: 2.5, unit: 1l  }
  product: &water     { name: Wasser, price: 2.5, unit: 1l  }

products:
  *coffee: { amount: 5 }
  *tea: { amount: 6, sold: 2 }
  *water:
    amount: 6
    returned: 4
...
"#;





#[test]
//#[ignore]
fn validate_products(){
    let doc = yaml::parse(PRODUCT_TEST_DOC_VALID).unwrap();

    println!("{:#?}",doc);
    let products = spec::products::invoice_items(&doc).unwrap();
    println!("Products {:#?}",products);
    assert_eq!(products[0].item.name, "Kaffee");
    assert_eq!(products[0].amount_offered, 5f64);
    assert_eq!(products[0].amount_sold, 5f64);
    assert_eq!(products[0].cost_before_tax(), Currency(Some('€'), 12_50));
    assert_eq!(products[0].cost_after_tax(), Currency(Some('€'), 14_88));

    assert_eq!(products[1].item.name, "Tee");
    assert_eq!(products[1].amount_offered, 6f64);
    assert_eq!(products[1].amount_sold, 2f64);

    assert_eq!(products[2].item.name, "Wasser");
    assert_eq!(products[2].amount_offered, 6f64);
    assert_eq!(products[2].amount_sold, 2f64);
}






static PRODUCT_TEST_DOC_INVALID1:&'static str =
r#"
--- # sold and returend
cataloge:
  product: &tea       { name: Tee,    price: 2.5, unit: 1l  }
products:
  *tea: { amount: 6, sold: 2, returned: 4 }
...
"#;

static PRODUCT_TEST_DOC_INVALID2:&'static str =
r#"
--- # returning too much
cataloge:
  product: &tea { name: Tee, price: 2.5, unit: 1l }
products:
  *tea: { amount: 6, returned: 7 }
...
"#;

static PRODUCT_TEST_DOC_INVALID3:&'static str =
r#"
--- # returning too much
cataloge:
  product: &tea { name: Tee, price: 2.5, unit: 1l }
products:
  *tea: { returned: 7 }
...
"#;

#[test]
fn validate_invalid_products(){
    println!("canary");
    let invalid1= yaml::parse(PRODUCT_TEST_DOC_INVALID1).unwrap();
    let invalid2= yaml::parse(PRODUCT_TEST_DOC_INVALID2).unwrap();
    let invalid3= yaml::parse(PRODUCT_TEST_DOC_INVALID3).unwrap();
    assert_eq!( spec::products::invoice_items(&invalid1).unwrap_err(), ProductError::AmbiguousAmounts("Tee".to_owned()));
    assert_eq!( spec::products::invoice_items(&invalid2).unwrap_err(), ProductError::TooMuchReturned("Tee".to_owned()));
    assert_eq!( spec::products::invoice_items(&invalid3).unwrap_err(), ProductError::MissingAmount("Tee".to_owned()));
}

