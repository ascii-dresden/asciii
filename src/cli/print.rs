//! All the printing code lives here.

use chrono::*;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::{TableFormat,FormatBuilder};
use term::{Attr, color};

use super::ListConfig;

use project::Project;
use storage::Storable;
use repo::Repository;

//TODO construct table rows way more dynamically

#[allow(dead_code)]
pub fn print_project(_project:&Project){
    unimplemented!();
}

#[inline]
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
                    0      => "Fyb",
             -7 ... 0      => "Fr",
            -14 ... -7     => "Fy",
            _ if age < -14 => "Fg",
            _              => "d"
        };
    }
    "Fr"
}

/// produces the rows used in `print_projects()`
pub fn path_rows(projects:&[Project], list_config:&ListConfig) -> Vec<Row>{
    projects
        .iter()
        .map(|project| {
            let row_style = if list_config.use_colors {project_to_style(&project)}else{""};
            Row::new(vec![
                     cell!(project.invoice_num().unwrap_or("".into())),
                     cell!(project.name()).style_spec(row_style),
                     cell!(project.file().display()),

                     //cell!(project.date().map(|d|d.format("%d.%m.%Y").to_string()).unwrap_or("no_date".into())),
                     //cell!(project.file().display()),
            ])
        })
    .collect()
}

/// Triggered by `list --simple`, usually you set this in your config under `list/verbose: false`.
pub fn simple_rows(projects:&[Project], list_config:&ListConfig) -> Vec<Row>{
    projects
        .iter()
        .map(|project| {
            let row_style = if list_config.use_colors {project_to_style(&project)}else{""};
            Row::new(vec![
                     cell!(
                         if project.canceled() {
                             format!("X {name}", name=project.name())
                         } else{
                             project.name()
                         })
                     .style_spec(row_style),

                     //cell!(project.manager()),
                     cell!(project.invoice_num().unwrap_or("".into())),

                     cell!(project.date().map(|d|d.format("%d.%m.%Y").to_string()).unwrap_or("no_date".into())),
                     //cell!(project.file().display()),
            ])
        })
    .collect()
}

/// Triggered by `list --verbose`, usually you set this in your config under `list/verbose`.
///
/// produces the rows used in `print_projects()`
#[inline]
pub fn verbose_rows(projects:&[Project], list_config:&ListConfig, repo:Option<Repository>) -> Vec<Row>{
    projects.iter().enumerate()
        .map(|(i, project)| {
            //trace!("configuring row: {:?}", project.name());
            let row_style = if list_config.use_colors {project_to_style(&project)}else{""};
            let mut cells = Vec::new();

            if let Some(ref repo) = repo{
                // TODO how can we illustrate that a project has been removed? what about a red x
                // for every project that was just moved to the archive?
                // Or just git-add them when archiving automatically, that is what ascii2 would
                // have done
                let status = repo.get_status(&project.dir());
                let (color, style) = status.to_style();

                cells.push( Cell::new( &status.to_string() )
                            .with_style( Attr::ForegroundColor(color) )
                            .with_style( style.unwrap_or(Attr::Standout(false)) )
                            );
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
                                 cell!( project.get(&d).unwrap_or_else(String::new)),
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

/// Triggered by `list --nothing`
///
/// This prints nothing unless you tell it to with `--details`
pub fn dynamic_rows(projects:&[Project], list_config:&ListConfig, _repo:Option<Repository>) -> Vec<Row>{
    projects
        .iter()
        .map(|project| {
            let row_style = if list_config.use_colors {project_to_style(&project)}else{""};

            let mut cells = Vec::new();

            if let Some(ref details) = list_config.details{
                cells.extend_from_slice(
                    &details.iter().map(|d|
                                        cell!( project.get(&d).unwrap_or_else(String::new)).style_spec(row_style),
                                        ).collect::<Vec<Cell>>()
                    );
                if list_config.show_errors{
                    let validation = (project.valid_stage1(),
                    project.valid_stage2(),
                    project.valid_stage3());

                    cells.extend_from_slice( &[
                                             // Errors
                                             cell!(validation.0.err().map(|errs| errs.join(", ")).unwrap_or("".to_owned())),
                                             cell!(validation.1.err().map(|errs| errs.join(", ")).unwrap_or("".to_owned())),
                                             cell!(validation.2.err().map(|errs| errs.join(", ")).unwrap_or("".to_owned())),
                    ]);
                }
            }
            Row::new(cells)
        })
    .collect()
}

/// Prints Projects Rows
///
/// This doesn't do much, except taking a Vec of Rows and printing it,
/// the interesting code is in dynamic_rows, verbose_rows, path_rows or simple_rows.
/// This Documentations is redundant, infact, it is already longer than the function itself.
pub fn print_projects(rows:Vec<Row>){
    trace!("starting table print");
    let mut table = Table::init(rows);
    trace!("setting table format");
    table.set_format(FormatBuilder::new().column_separator(' ').padding(0,0).build());
    trace!("calling table print");
    table.printstd();
    trace!("done printing table.");
}

/// Prints Projects as CSV
///
/// This doesn't do much, except taking a Vec of Rows and printing it,
/// the interesting code is in dynamic_rows, verbose_rows, path_rows or simple_rows.
/// This Documentations is redundant, infact, it is already longer than the function itself.
pub fn print_csv(projects:&[Project]){
    // TODO print head
    let splitter = ";";
        println!("{}", [
                 "Rnum",
                 "Bezeichnung",
                 "Datum",
                 "Rechnungs",
                 "Betreuer",
                 "Verantwortlich",
                 "Bezahlt am",
                 "Betrag",
                 "Canceled"].join(splitter));
    for project in projects{
        println!("{}", [
            project.get("InvoiceNumber").unwrap_or_else(String::new),
            project.get("Name").unwrap_or_else(String::new),
            project.get("event/dates/0/begin").unwrap_or_else(String::new),
            project.get("invoice/date").unwrap_or_else(String::new),
            project.get("Caterers").unwrap_or_else(String::new),
            project.get("Responsible").unwrap_or_else(String::new),
            project.get("invoice/payed_date").unwrap_or_else(String::new),
            project.get("Final").unwrap_or_else(String::new),
            project.canceled_string().to_owned()
        ].join(splitter));
    }
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
    let mut table = table!{
        ["sold ", project.sum_sold().unwrap() ],
        ["tax", project.tax_sold().unwrap() ],
        ["sum+tax", project.sum_sold_and_taxes().unwrap()],
        ["sum+wages", project.sum_sold_and_wages().unwrap()]
    };
    table.set_format(TableFormat::new());
    table.printstd();

    //println!("{}", project.sum_offered().unwrap());
    //println!("{}", project.tax_offered().unwrap());

}
