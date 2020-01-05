use asciii::{
    document_export::fill_template,
    project::{
        export::{Complete, ExportTarget},
        BillType, Project,
    },
};
use pretty_assertions::assert_eq;

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
