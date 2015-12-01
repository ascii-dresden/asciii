use chrono::UTC;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::{TableFormat,Align};
use term::{Attr, color};

use project::Project;
use manager::LuigiProject;

pub fn print_project(_project:&Project){
    unimplemented!();
}

pub fn print_projects(projects:&[Project]){
    let mut table = Table::new();
    table.set_format(TableFormat::new("", None, None));
    for (i, project) in projects.iter().enumerate(){
        table.add_row(Row::new(vec![
                      cell!(r:i+1), // .with_style(Attr::ForegroundColor(color::GREEN)),
                      cell!(project.name()),
                      cell!(project.manager()),
                      cell!(project.invoice_num()),
                      cell!(project.date().unwrap_or(UTC::today())
                            .format("%d.%m.%Y").to_string()
                            )
        ])
                      );
    }
    table.printstd();
}
