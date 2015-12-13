use chrono::UTC;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::{TableFormat,Align};
use term::{Attr, color};

use project::Project;
use manager::LuigiProject;
use repo::{Repo,GitStatus};

pub fn print_project(_project:&Project){
    unimplemented!();
}

pub fn simple_rows(projects:&[Project]) -> Vec<Row>{
    projects
        .iter()
        .map(|project|
             Row::new(vec![
                      cell!(project.name()),
                      cell!(project.manager()),
                      cell!(project.invoice_num()),
                      cell!(project.date().unwrap_or(UTC::today())
                            .format("%d.%m.%Y").to_string()),

                      cell!(project.index().unwrap_or("no_index".into())),
                      cell!(project.date().map(|d|d.to_string()).unwrap_or("no_date".into())),
                      //cell!(project.file().display()),
             ])
            )
        .collect()
}

fn print_result(res:&Result<(), Vec<&str>>) -> String{
    match res{
        &Ok(_) => String::from("✓"), // ✗
        &Err(ref errors) => String::from("✗")// + &errors.join(", ")
    }
}

pub fn status_rows(projects:&[Project], repo:&Repo) -> Vec<Row>{
    projects
        .iter()
        .map(|project| {
            //TODO
            Row::new(vec![
                     cell!(project.name()),
                     cell!(project.manager()),
                     cell!(project.invoice_num()),
                     cell!(project.date().unwrap_or(UTC::today()).format("%d.%m.%Y").to_string()),
                     cell!(print_result(&project.valid_stage1())),
                     cell!(print_result(&project.valid_stage2())),
                     cell!(print_result(&project.valid_stage3())),

                     cell!(repo.status.get(&project.file()).unwrap_or(&GitStatus::Unknown)),
                     cell!(repo.status.get(&project.dir()).unwrap_or(&GitStatus::Unknown)),
            ])
        })
        .collect()
}

// TODO add this code to prettytable-rs
pub fn print_projects(mut rows:Vec<Row>){
    let mut table = Table::new();
    table.set_format(TableFormat::new(None, None, None));
    for (i, mut row) in rows.drain(..).enumerate(){
        row.insert_cell(0,cell!(r:i+1)); // make this optional
        table.add_row(row);
    }
    table.printstd();
}
