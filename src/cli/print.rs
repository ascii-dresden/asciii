#![allow(dead_code)]

use chrono::*;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::{TableFormat,FormatBuilder};
use term::{Attr, color};

use super::ListConfig;

use project::Project;
use manager::LuigiProject;
use repo::Repository;
use util::yaml;

//TODO construct table rows way more dynamically

pub fn print_project(_project:&Project){
    unimplemented!();
}
fn result_to_cell(res:&Result<(), Vec<&str>>) -> Cell{
    match *res{
        Ok(_)           => Cell::new("✓").with_style(Attr::ForegroundColor(color::GREEN)), // ✗
        Err(ref _errors) => Cell::new("✗").with_style(Attr::ForegroundColor(color::RED))// + &errors.join(", ") )
        //&Err(ref errors) => Cell::new( &format!("✗ {}",  &errors.join(", ") )) .with_style(Attr::ForegroundColor(color::RED))
    }
}

/// create a Style string from the properties of a project
fn project_to_style(project:&Project) -> &str{
    // can be send as invoice
    if project.valid_stage2().is_ok(){
        return "d"
    }

    if let Some(date) = project.date(){
        let age = (Local::today() - date).num_days();
        if project.canceled(){
            return ""
        }
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

/// produces the rows used in `print_projects()`
pub fn simple_rows(projects:&[Project], list_config:&ListConfig) -> Vec<Row>{
    projects
        .iter()
        .map(|project| {
            let row_style = if list_config.use_colors {project_to_style(&project)}else{""};
            Row::new(vec![
                     cell!(
                         if project.canceled() { format!("X {name}", name=project.name()) }
                         else{ project.name() }
                         ).style_spec(row_style),

                         //cell!(project.manager()),
                         cell!(project.invoice_num().unwrap_or("".into())),

                         cell!(project.date().map(|d|d.format("%d.%m.%Y").to_string()).unwrap_or("no_date".into())),
                         //cell!(project.file().display()),
            ])
        })
    .collect()
}

/// produces the rows used in `print_projects()`
pub fn rows(projects:&[Project], list_config:&ListConfig) -> Vec<Row>{
    projects.iter().enumerate()
        .map(|(i, project)| {
            let row_style = if list_config.use_colors {project_to_style(&project)}else{""};
            let cells = vec![

                cell!(r->i+1),

                cell!(
                    if project.canceled() {
                        format!("CANCELED: {name}", name=project.name())
                    } else{ project.name() }
                    ).style_spec(row_style),

                // R042
                cell!(project.invoice_num().unwrap_or("".into()))
                    .style_spec(row_style),

                // Date
                cell!(project.date().unwrap_or(UTC::today()).format("%d.%m.%Y").to_string())
                    .style_spec(row_style),
           ];

            Row::new(cells)
        }).collect()
}

/// produces the rows used in `print_projects()`
pub fn verbose_rows(projects:&[Project], list_config:&ListConfig, repo:Option<Repository>) -> Vec<Row>{
    projects.iter().enumerate()
        .map(|(i, project)| {
            let row_style = if list_config.use_colors {project_to_style(&project)}else{""};
            let mut cells = Vec::new();

            if let Some(ref repo) = repo{
                let status = repo.get_status(&project.dir());
                cells.push( Cell::new( &status.to_string() ).with_style( Attr::ForegroundColor(status.to_color()) ));
            };

            let validation1 = project.valid_stage1();
            let validation2 = project.valid_stage2();
            let validation3 = project.valid_stage3();

            cells.extend_from_slice( &[
                cell!(r->i+1),

                cell!(
                    if project.canceled() {
                        format!("CANCELED: {name}", name=project.name())
                    } else{ project.name() }
                    ).style_spec(row_style),

                // Hendrik Sollich
                cell!(project.manager())
                    .style_spec(row_style),

                // sort index
                //cell!(project.index().unwrap_or(String::from(""))),

                // R042
                cell!(project.invoice_num().unwrap_or("".into()))
                    .style_spec(row_style),

                // Date
                cell!(project.date().unwrap_or(UTC::today()).format("%d.%m.%Y").to_string())
                    .style_spec(row_style),

                // status "✓  ✓  ✗"
                result_to_cell(&validation1),
                result_to_cell(&validation2),
                result_to_cell(&validation3),

                //cell!(project.sum_invoice().map(|i|i.to_string()).unwrap_or(String::from("none"))),
                //cell!(project.wages().map(|i|i.to_string()).unwrap_or(String::from("none"))),
                cell!(project.sum_sold_and_wages().map(|i|i.to_string()).unwrap_or(String::from("none"))),

            ]);

            if let Some(ref details) = list_config.details{
                cells.extend_from_slice(
                    &details.iter().map(|d|
                                 cell!( project.get(&d).unwrap_or_else(||String::from(""))),
                                 ).collect::<Vec<Cell>>()
                    );
            }


            //if list_config.details{
            //    // TODO
            //    //
            //}

            if list_config.show_errors{
                cells.extend_from_slice( &[
                                         // Errors
                                         cell!(validation1.err().map(|errs| errs.join(", ")).unwrap_or("".to_owned())),
                                         cell!(validation2.err().map(|errs| errs.join(", ")).unwrap_or("".to_owned())),
                                         cell!(validation3.err().map(|errs| errs.join(", ")).unwrap_or("".to_owned())),
                ]);
            }

            Row::new(cells)
        }).collect()
}

#[allow(dead_code,unused_variables)]
pub fn dynamic_rows(projects:&[Project], fields:&[&str], repo:Option<Repository>) -> Vec<Row>{
    projects.iter().enumerate()
        .map(|(i, project)| {
            let mut cells = vec![
                cell!(r->i+1)
            ];
            for field in fields{
                let data = yaml::get_as_string(project.yaml(), field)
                    .unwrap_or_else(||String::from(""));
                cells.push(cell!(data))
            }
            Row::new(cells)
        }).collect()
}

/// Prints Projects
pub fn print_projects(rows:Vec<Row>){
    let mut table = Table::init(rows);
    table.set_format(FormatBuilder::new().column_separator(' ').padding(0,0).build());
    table.printstd();
}

pub fn show_items(project:&Project){

    println!("{}", project.name());

    let mut table = Table::new();
    table.set_format(TableFormat::new());
    for item in &project.invoice_items().unwrap(){
        table.add_row( Row::new(vec![
                                Cell::new(item.item.name),
                                Cell::new(&item.amount_sold.to_string()),
                                Cell::new(&item.item.price.to_string()),
                                Cell::new(&(item.item.price * item.amount_sold).to_string()),
        ]));
    }
    table.printstd();
    println!("{}", project.sum_sold().unwrap());
    println!("{}", project.tax_sold().unwrap());
    println!("{}", project.sum_sold_and_taxes().unwrap());
    println!("{}", project.sum_sold_and_wages().unwrap());

    //println!("{}", project.sum_offered().unwrap());
    //println!("{}", project.tax_offered().unwrap());

}
