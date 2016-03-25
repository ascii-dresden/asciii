use chrono::*;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::{TableFormat,FormatBuilder};
use term::{Attr, color};

use project::Project;
use manager::LuigiProject;
use repo::Repository;

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

fn project_to_style(project:&Project) -> &str{
    if project.valid_stage2().is_ok(){
        return "d";
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
pub fn simple_rows(projects:&[Project]) -> Vec<Row>{
    projects
        .iter()
        .map(|project| {
            let row_style = project_to_style(&project);
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
pub fn status_rows(projects:&[Project], repo:&Repository) -> Vec<Row>{
    projects.iter().enumerate()
        .map(|(i, project)| {
            let status = repo.get_status(&project.dir());
            let row_style = project_to_style(&project);
            let cells = vec![

                Cell::new( &status.to_string() )
                    .with_style( Attr::ForegroundColor(status.to_color()) ),

                cell!(r->i+1),

                cell!(
                    if project.canceled() {
                        format!("canceled: {name}", name=project.name())
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
                result_to_cell(&project.valid_stage1()),
                result_to_cell(&project.valid_stage2()),
                result_to_cell(&project.valid_stage3()),

                //cell!(project.sum_invoice().map(|i|i.to_string()).unwrap_or(String::from("none"))),
                //cell!(project.wages().map(|i|i.to_string()).unwrap_or(String::from("none"))),
                cell!(project.sum_sold_and_wages().map(|i|i.to_string()).unwrap_or(String::from("none"))),
            ];
            Row::new(cells)
        })
        .collect()
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
