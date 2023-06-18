use crate::{
    project::{spec::*, Project},
    storage::Storable,
};
use std::path::Path;

fn parse_project(yaml: &str) -> Project {
    Project::from_file_content(yaml).unwrap()
}

#[test]
#[rustfmt::skip]
#[ignore]
fn compare_basics(){
    println!("{:?}", ::std::env::current_dir());
    let new_project = Project::open_file(Path::new("./tests/test_projects/current.yml")).unwrap();
    let old_project = Project::open_file(Path::new("./tests/test_projects/old.yml")).unwrap();
    //let config = &::CONFIG;

    assert_eq!(old_project.name(),
                new_project.name());

    assert_eq!(old_project.offer().number(),
                new_project.offer().number());

    //assert_eq!(old_project.offer().date(), // old format had no offer date
    //           new_project.offer().date());

    assert_eq!(old_project.invoice().number_str(),
                new_project.invoice().number_str());

    assert_eq!(old_project.invoice().date(),
                new_project.invoice().date());

    assert_eq!(old_project.payed_date(),
                new_project.payed_date());

    assert_eq!(old_project.client().title(),
                new_project.client().title());

    assert_eq!(old_project.client().last_name(),
                new_project.client().last_name());

    assert_eq!(old_project.client().addressing(),
                new_project.client().addressing());

    assert_eq!(old_project.client().address(),
                new_project.client().address());
}

pub mod client {
    use super::*;

    #[test]
    fn validate_stage1() {
        let doc = r#"
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

        assert_eq!(
            parse_project(doc).client().validate().missing_fields,
            Vec::<String>::new()
        );
    }

    #[test]
    fn missing_address() {
        let doc = r#"
      client:
        title:      Herr # Frau, Professor, Professorin
        first_name: Graf
        last_name:  Zahl
        email: this.man@example.com
      "#;
        assert_eq!(
            parse_project(doc).client().missing_fields(),
            vec!["client/address".to_string()]
        );
    }

    #[test]
    fn missing_title() {
        let doc = r#"
      client:
        first_name: Graf
        last_name:  Zahl
        email: this.man@example.com
        address: |
          Graf Zahl
          Nummernhöllenstraße 666
          01234 Countilvania
      "#;
        assert_eq!(
            parse_project(doc).client().missing_fields(),
            vec!["client/title".to_string(), "client_addressing".to_string(),]
        );
    }

    #[test]
    fn missing_last_name() {
        let doc = r#"
      client:
        title:      Herr # Frau, Professor, Professorin
        first_name: Graf
        email: this.man@example.com
        address: |
          Graf Zahl
          Nummernhöllenstraße 666
          01234 Countilvania
      "#;
        assert_eq!(
            parse_project(doc).client().missing_fields(),
            vec!["client/last_name", "client_addressing"]
        );
    }
}

pub mod offer {
    use super::*;

    #[test]
    fn validate_stage2() {
        let doc = r#"
      offer:
        date: 07.11.2014
        appendix: 1
      manager: somebody
      "#;

        let errors = parse_project(doc).offer().missing_fields();
        assert_eq!(errors, Vec::<String>::new());
    }
}

pub mod invoice {
    use super::*;

    #[test]
    fn validate_stage3() {
        let doc = r#"
      invoice:
        number: 41
        date: 06.12.2014
        payed_date: 08.12.2014
      "#;

        let errors = parse_project(doc).invoice().missing_fields();
        assert_eq!(errors, Vec::<String>::new());
    }

    #[test]
    fn validate_stage3_missing_date() {
        let doc = r#"
        invoice:
          number: 41
          payed_date: 08.12.2014
        "#;

        let errors = parse_project(doc).invoice().missing_fields();
        assert_eq!(errors, vec!["invoice.date"]);
    }
}

/*

mod product {
  use super::*;

  static PRODUCT_TEST_DOC_VALID: &'static str = r#"
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





  //  #[test]
  //  fn validate_products() {
  //      let doc = yaml::parse(PRODUCT_TEST_DOC_VALID).unwrap();
  //
  //      println!("{:#?}", doc);
  //      let products = spec::products::invoice_items(&doc).unwrap();
  //      println!("Products {:#?}", products);
  //      assert_eq!(products[0].product.name, "Kaffee");
  //      assert_eq!(products[0].amount_offered, 5f64);
  //      assert_eq!(products[0].amount_sold, 5f64);
  //      assert_eq!(products[0].cost_sold_before_tax(),
  //                Currency(Some('€'), 12_50));
  //      assert_eq!(products[0].cost_sold_after_tax(),
  //                Currency(Some('€'), 14_88));
  //
  //      assert_eq!(products[1].product.name, "Tee");
  //      assert_eq!(products[1].amount_offered, 6f64);
  //      assert_eq!(products[1].amount_sold, 2f64);
  //
  //      assert_eq!(products[2].product.name, "Wasser");
  //      assert_eq!(products[2].amount_offered, 6f64);
  //      assert_eq!(products[2].amount_sold, 2f64);
  //  }








  static PRODUCT_CATALOG: &'static str = r#"
  cataloge:
    product: &apfelsaft    { name: Apfelsaft,       price: 1.64,  unit: 1l  }
    product: &orangensaft  { name: Orangensaft,     price: 1.86,  unit: 1l  }
    product: &broetchen    { name: halbe Brötchen,  price: 1.16,  unit: stk }
    product: &gluehwein    { name: Glühwein,        price: 1.60,  unit: 1l  }
    product: &kaffee       { name: Kaffee,          price: 2.5,   unit: 1l  }
    product: &tee          { name: Tee,             price: 2.5,   unit: 1l  }
    product: &kekse        { name: Kekse,           price: 3.40,  unit: 400g}
    product: &kinderpunsch { name: Punsch,          price: 1.60,  unit: 1l  }
    product: &salzgebaeck  { name: Salzgebäck,      price: 3.50,  unit: 400g}
    product: &wasser_gross { name: Mineralwasser,   price: 0.61,  unit: 1l  }
  "#;

  static PRODUCT_TEST_SUM_UP1: &'static str = r#"
  products:
    *kaffee: { amount: 20 }
    *wasser_gross: { amount: 100 }
    *apfelsaft: { amount: 15 }
    *orangensaft: { amount: 15 }
    *kekse: { amount: 5 }
  "#;


  static PRODUCT_TEST_SUM_UP2: &'static str = r#"
  products:
    *kinderpunsch: { amount: 10 } # 10 * 1.60
    *gluehwein: { amount: 15 }    # 15 * 1.60
    *kekse: { amount: 3 }         #  3 * 3.40
    *salzgebaeck: { amount: 3 }   #  3 * 3.50
  "#;


  static PRODUCT_TEST_SUM_UP3: &'static str = r#"
  products:
    *wasser_gross: { amount:  8 } # 0.61 *  8 =  4.88
    *apfelsaft:    { amount:  4 } # 1.64 *  4 =  6.56
    *orangensaft:  { amount:  4 } # 1.86 *  4 =  7.44
    *broetchen:    { amount: 40 } # 1.16 * 40 = 46.40
                                            # 65.28
  "#;

  static PRODUCT_TEST_SUM_UP4: &'static str = r#"

  products:
    *kaffee: { amount: 5 }        # 5 * 2.50 = 12.50
    *tee: { amount: 1 }           # 1 * 2.50 =  2.50
    *wasser_gross: { amount: 4 }  # 4 * 0.61 =  2.44
    *orangensaft: { amount: 2 }   # 2 * 1.86 =  3.72
    *apfelsaft: { amount: 2 }     # 2 * 1.64 =  3.28
    *kekse: { amount: 1 }         # 1 * 3.40 =  3.40
                                  #            27.84 aber 27.09
  "#;

  //  fn compare_sum(doc_string: &str, want: i64) {
  //      let doc_string_plus_catalogue = [PRODUCT_CATALOG, doc_string].join("\n");
  //
  //      let doc = yaml::parse(&doc_string_plus_catalogue).unwrap();
  //      let products = spec::products::invoice_items(&doc).unwrap();
  //      let sum_offered = spec::products::sum_offered(&products);
  //      let sum_sold = spec::products::sum_sold(&products);
  //
  //      // assert_eq!(sum_sold.1, 334_79);
  //      assert_eq!(sum_sold.1, want);
  //  }

  //  #[test]
  //  fn check_prices() {
  //      let doc = yaml::parse(&[PRODUCT_CATALOG, PRODUCT_TEST_SUM_UP1].join("\n")).unwrap();
  //      let products = spec::products::invoice_items(&doc).unwrap();
  //      assert_eq!(products[0].product.name, "Apfelsaft");
  //      assert_eq!(products[0].product.price.1, 164);
  //      assert_eq!(products[1].product.name, "Kaffee");
  //      assert_eq!(products[2].product.name, "Kekse");
  //
  //      let odd_prices = r#"products: { *broetchen:    { amount: 40 } } "#;
  //
  //      let doc = yaml::parse(&[PRODUCT_CATALOG, odd_prices].join("\n")).unwrap();
  //      let products = spec::products::invoice_items(&doc).unwrap();
  //      assert_eq!(products[0].product.name, "halbe Brötchen");
  //      assert_eq!(products[0].product.price.1, 116);
  //  }
  //
  //   #[test]
  //   fn sum_up_products1() {
  //       compare_sum(PRODUCT_TEST_SUM_UP1, 180_50);
  //   }
  //   #[test]
  //   fn sum_up_products2() {
  //       compare_sum(PRODUCT_TEST_SUM_UP2, 60_70);
  //   }
  //   #[test]
  //   fn sum_up_products3() {
  //       compare_sum(PRODUCT_TEST_SUM_UP3, 65_28);
  //   }
    // #[test] fn sum_up_products4(){ compare_sum(PRODUCT_TEST_SUM_UP4, 27_09); }

  static PRODUCT_TEST_DOC_INVALID1: &'static str = r#"
  --- # sold and returend
  cataloge:
    product: &tea       { name: Tee,    price: 2.5, unit: 1l  }
  products:
    *tea: { amount: 6, sold: 2, returned: 4 }
  ...
  "#;

  static PRODUCT_TEST_DOC_INVALID2: &'static str = r#"
  --- # returning too much
  cataloge:
    product: &tea { name: Tee, price: 2.5, unit: 1l }
  products:
    *tea: { amount: 6, returned: 7 }
  ...
  "#;

  static PRODUCT_TEST_DOC_INVALID3: &'static str = r#"
  --- # returning too much
  cataloge:
    product: &tea { name: Tee, price: 2.5, unit: 1l }
  products:
    *tea: { returned: 7 }
  ...
  "#;

//  #[test]
//  fn validate_invalid_products() {
//      println!("canary");
//      let invalid1 = yaml::parse(PRODUCT_TEST_DOC_INVALID1).unwrap();
//      let invalid2 = yaml::parse(PRODUCT_TEST_DOC_INVALID2).unwrap();
//      let invalid3 = yaml::parse(PRODUCT_TEST_DOC_INVALID3).unwrap();
//      assert_eq!(spec::products::invoice_items(&invalid1).unwrap_err(),
//                ProductError::AmbiguousAmounts("Tee".to_owned()));
//      assert_eq!(spec::products::invoice_items(&invalid2).unwrap_err(),
//                ProductError::TooMuchReturned("Tee".to_owned()));
//      assert_eq!(spec::products::invoice_items(&invalid3).unwrap_err(),
//                ProductError::MissingAmount("Tee".to_owned()));
//  }


//   #[test]
//   fn rounding() {
//       static PRODUCT_TEST_DOC_INVALID3: &'static str = r#"
//   products:
//     gutschein5: { amount: 8, sold: 8, price: 4.20 }
//     gutschein7: { amount: 1, sold: 1, price: 5.90 }
//   "#;
//
//       let doc = yaml::parse(PRODUCT_TEST_DOC_INVALID3).unwrap();
//       let products = spec::products::invoice_items(&doc).unwrap();
//       let sum_offered = spec::products::sum_offered(&products);
//       let sum_sold = spec::products::sum_sold(&products);
//       assert_eq!(sum_sold * 1.19, Currency(Some('€'), 4701))
//   }

}

*/
