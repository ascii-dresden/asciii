use chrono::UTC;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::TableFormat;

use project::Project;
use manager::LuigiProject;

pub fn sort_by_date(_projects:&[Project]){
    unimplemented!();
}

pub fn print_project(_project:&Project){
    unimplemented!();
}

pub fn print_projects(projects:&[Project]){
    let mut table = Table::new();
    table.set_format(TableFormat::new("", None, None));
    for (i, project) in projects.iter().enumerate(){
        table.add_row(row![
                      i+1,
                      project.name(),
                      project.manager(),
                      project.invoice_num(),
                      project.date().unwrap_or(UTC::today())
                             .format("%d.%m.%Y").to_string()]
                      );
    }
    table.printstd();
}
