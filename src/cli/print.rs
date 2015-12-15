use chrono::*;
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

fn result_to_cell(res:&Result<(), Vec<&str>>) -> Cell{
    match res{
        &Ok(_)           => Cell::new("✓").with_style(Attr::ForegroundColor(color::GREEN)), // ✗
        &Err(ref errors) => Cell::new("✗").with_style(Attr::ForegroundColor(color::RED))// + &errors.join(", ")
    }
}

fn project_to_style<'a>(project:&'a Project) -> &'a str{
    if project.valid_stage2().is_ok(){
        return "d";
    }

    if let Some(date) = project.date(){
        let age = (Local::today() - date).num_days();
        return match age{
            _ if age > 28  => "Fm",
              1 ... 28     => "Fc",
                    0      => "Fy",
             -7 ... 0      => "Fr",
            -14 ... -7     => "Fy",
            _ if age < -14 => "Fg",
            _              => "d"
        };
    }
    "Fr"
}

pub fn status_rows(projects:&[Project], repo:&Repo) -> Vec<Row>{
    projects
        .iter()
        .enumerate()
        .map(|(i, project)| {
            let status = repo.get_status(&project.dir());
            let row_style = project_to_style(&project);
            let cells = vec![

                Cell::new( &status.to_string() )
                    .with_style( Attr::ForegroundColor(status.to_color()) ),

                cell!(r:i),

                cell!(project.name())
                    .style_spec(row_style),
                // Hendrik Sollich
                cell!(project.manager())
                    .style_spec(row_style),
                // R042
                cell!(project.invoice_num())
                    .style_spec(row_style),
                // Date
                cell!(project.date().unwrap_or(UTC::today()).format("%d.%m.%Y").to_string())
                    .style_spec(row_style),

                // status "✓  ✓  ✗"
                result_to_cell(&project.valid_stage1()),
                result_to_cell(&project.valid_stage2()),
                result_to_cell(&project.valid_stage3()),

            ];
            Row::new(cells)
        })
        .collect()
}

// TODO add this code to prettytable-rs
pub fn print_projects(rows:Vec<Row>){
    let mut table = Table::init(rows);
    table.set_format(TableFormat::new(None, None, None));
    table.printstd();
}
