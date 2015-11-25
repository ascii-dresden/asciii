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
    for project in projects{
        table.add_row(row![project.index(), project.name(), project.manager(), project.date()]);
    }
    table.printstd();
}
