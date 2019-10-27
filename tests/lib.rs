use asciii;

use asciii::project::Project;

#[cfg(all(feature = "serialization", test))]
mod regression {
    mod document_export {
        use asciii::{
            document_export::fill_template,
            project::{
                export::{Complete, ExportTarget},
                BillType, Project,
            },
        };

        fn export(path: &str, bill_type: BillType) -> String {
            let p = Project::open(path).unwrap();
            let exported: Complete = p.export();
            std::env::set_var("ASCIII_PATH", "");
            fill_template(&exported, bill_type, "./templates/export.tex.hbs").unwrap()
        }

        #[test]
        fn current_offer() {
            let exported = export("./tests/test_projects/current.yml", BillType::Offer);
            let expected = include_str!("./test_projects/expected_exports/current_offer");
            assert_eq!(exported, expected)
        }

        #[test]
        fn current_invoice() {
            let exported = export("./tests/test_projects/current.yml", BillType::Invoice);
            let expected = include_str!("./test_projects/expected_exports/current_invoice");
            assert_eq!(exported, expected)
        }

        #[test]
        fn inline_offer() {
            let exported = export("./tests/test_projects/inline.yml", BillType::Offer);
            let expected = include_str!("./test_projects/expected_exports/inline_offer");
            assert_eq!(exported, expected)
        }

        #[test]
        fn inline_invoice() {
            let exported = export("./tests/test_projects/inline.yml", BillType::Invoice);
            let expected = include_str!("./test_projects/expected_exports/inline_invoice");
            assert_eq!(exported, expected)
        }

    }
}

#[cfg(test)]
mod taxed_service {
    use super::*;
    use asciii::project::spec::HasEmployees;

    #[test]
    fn services_are_implicitly_zero() {
        let hours_zero_tax: &str = "hours: { salary: 8.0, caterers: { unknown: 3 }, tax: 0 }";
        let hours_untaxed: &str = "hours: { salary: 8.0, caterers: { unknown: 3 } }";

        let project_untaxed = Project::from_file_content(&hours_untaxed).unwrap();
        let project_zero_tax = Project::from_file_content(&hours_zero_tax).unwrap();

        assert_eq!(
            project_untaxed.hours().net_wages(),
            project_zero_tax.hours().net_wages()
        );
    }

    #[test]
    fn services_calculate_gross_wages() {
        let hours_taxed: &str = r#"
hours: { salary: 8.0, caterers: { unknown: 3 }, tax: 0.19 }
tax: 0.19"#;
        let hours_untaxed: &str = "hours: { salary: 9.52, caterers: { unknown: 3 } }";

        let project_taxed = Project::from_file_content(&hours_taxed).unwrap();
        let project_untaxed = Project::from_file_content(&hours_untaxed).unwrap();
        assert_eq!(
            project_taxed.hours().net_wages(),
            project_untaxed.hours().net_wages()
        );
    }
}
