use asciii::project::{spec::HasEmployees, Project};

use pretty_assertions::assert_eq;

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
