//! All the printing code lives here.

use std::error::Error;

use chrono::*;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format::{LineSeparator,LinePosition,FormatBuilder};
use term::{Attr, color};
use super::BillType;

use project::Project;
use storage::Storable;
use util::currency_to_string;

//TODO construct table rows way more dynamically

/// Configuration for this list output.
#[derive(Debug)]
pub struct ListConfig<'a>{
    pub mode:         ListMode,
    pub show_errors:  bool,
    pub git_status:   bool,
    pub sort_by:      &'a str,
    pub filter_by:    Option<Vec<&'a str>>,
    pub use_colors:   bool,
    pub details:      Option<Vec<&'a str>>,
}

#[derive(Debug, PartialEq)]
pub enum ListMode{ Simple, Verbose, Nothing, Paths, Csv }

impl<'a> Default for ListConfig<'a>{
    fn default() -> ListConfig<'a>{
        ListConfig{
            mode:         if ::CONFIG.get_bool("list/verbose"){ ListMode::Verbose } else{ ListMode::Simple },
            git_status:   ::CONFIG.get_bool("list/gitstatus"),
            show_errors:  false,
            sort_by:      ::CONFIG.get_str("list/sort").expect("Faulty config: list/sort does not contain a value"),
            filter_by:    None,
            use_colors:   ::CONFIG.get_bool("list/colors"),
            details:      None,
        }
    }
}

fn payed_to_cell(project:&Project) -> Cell{
    let sym = ::CONFIG.get_str("currency").expect("Faulty config: currency does not contain a value");
    match (project.payed_by_client(), project.payed_caterers()) {
        (false,false) => Cell::new("✗").with_style(Attr::ForegroundColor(color::RED)),
        (_,   false) => Cell::new(sym).with_style(Attr::ForegroundColor(color::YELLOW)),
        (_,   true) => Cell::new(sym).with_style(Attr::ForegroundColor(color::GREEN)),
    }
}

fn result_to_cell(res:&Result<(), Vec<&str>>, bold:bool) -> Cell{
    match (res, bold){
        (&Ok(_),           false) => Cell::new("✓").with_style(Attr::ForegroundColor(color::GREEN)), // ✗
        (&Ok(_),           true)  => Cell::new("✓").with_style(Attr::ForegroundColor(color::GREEN))
                                                   .with_style(Attr::Bold), // ✗
        (&Err(ref _errors),_)     => Cell::new("✗").with_style(Attr::ForegroundColor(color::RED))// + &errors.join(", ") )
        //&Err(ref errors) => Cell::new( &format!("✗ {}",  &errors.join(", ") )) .with_style(Attr::ForegroundColor(color::RED))
    }
}

/// create a Style string from the properties of a project
fn project_to_style(project:&Project) -> &str{
    // can be send as invoice
    if project.is_ready_for_invoice().is_ok(){
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
             -7 ... -1     => "Fr",
            -14 ... -8     => "Fy",
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
            let row_style = if list_config.use_colors {project_to_style(project)}else{""};
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
            let row_style = if list_config.use_colors {project_to_style(project)}else{""};
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
pub fn verbose_rows(projects:&[Project], list_config:&ListConfig) -> Vec<Row>{
    projects.iter().enumerate()
        .map(|(i, project)| {
            //trace!("configuring row: {:?}", project.name());
            let row_style = if list_config.use_colors {project_to_style(project)}else{""};
            let mut cells = Vec::new();

            // TODO how can we illustrate that a project has been removed? what about a red x
            // for every project that was just moved to the archive?
            // Or just git-add them when archiving automatically, that is what ascii2 would
            // have done
            let status = project.get_git_status();
            let (color, style) = status.to_style();

            cells.push( Cell::new( &status.to_string() )
                        .with_style( Attr::ForegroundColor(color) )
                        .with_style( style.unwrap_or_else(||Attr::Standout(false)) )
                      );


            let validation1 = project.is_ready_for_offer();
            let validation2 = project.is_ready_for_invoice();
            let validation3 = project.is_ready_for_archive();

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
                result_to_cell(&validation1, project.offer_file_exists()),
                result_to_cell(&validation2, project.invoice_file_exists()),
                payed_to_cell(&project),
                result_to_cell(&validation3, false),

                //cell!(output_file_exists(project, Project::offer_file_name)),
                //cell!(output_file_exists(project, Project::invoice_file_name)),

                cell!(r->project.sum_sold().map(|i|currency_to_string(&i)).unwrap_or(String::from("none"))),
                //cell!(project.wages().map(|i|i.to_string()).unwrap_or(String::from("none"))),
                //cell!(project.sum_sold_and_wages().map(|i|i.to_string()).unwrap_or(String::from("none"))),
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
pub fn dynamic_rows(projects:&[Project], list_config:&ListConfig) -> Vec<Row>{
    projects
        .iter()
        .map(|project| {
            let row_style = if list_config.use_colors {project_to_style(project)}else{""};

            let mut cells = Vec::new();

            if let Some(ref details) = list_config.details{
                cells.extend_from_slice(
                    &details.iter().map(|d|
                                        cell!( project.get(&d).unwrap_or_else(String::new)).style_spec(row_style),
                                        ).collect::<Vec<Cell>>()
                    );
                if list_config.show_errors{
                    let validation = (project.is_ready_for_offer(), project.is_ready_for_invoice(), project.is_ready_for_archive());

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
/// the interesting code is in `dynamic_rows()`, `verbose_rows()`, `path_rows()` or `simple_rows()`.
/// This Documentations is redundant, infact, it is already longer than the function itself.
pub fn print_projects(rows:Vec<Row>){
    trace!("starting table print");
    let mut table = Table::init(rows);
    table.set_format(FormatBuilder::new().column_separator(' ').padding(0,0).build());
    table.printstd();
    trace!("done printing table.");
}

/// Prints Projects as CSV
pub fn print_csv_year(year:i32){
    match ::actions::csv(year) {
        Ok(csv) => println!("{}", csv),
        Err(err) => println!("{}", err.description())
    }
}

/// Prints Projects as CSV
pub fn print_csv(projects:&[Project]){
    match ::actions::projects_to_csv(projects) {
        Ok(csv) => println!("{}", csv),
        Err(err) => println!("{}", err.description())
    }
}

//fn table_for_arrangement(table:&mut Table){
//    table.set_format(FormatBuilder::new() .padding(0, 0) .build());
//}

fn table_with_borders(table:&mut Table){
    table.set_format( FormatBuilder::new()
                      .borders('│').padding(1, 1)
                      .separators( &[LinePosition::Title], LineSeparator::new('─', '─', '├', '┤'))
                      .separators( &[LinePosition::Top],    LineSeparator::new('─', '─', '┌', '┐'))
                      .separators( &[LinePosition::Bottom], LineSeparator::new('─', '─', '└', '┘'))
                      .build()
                    );
}

pub fn show_details(project:&Project, bill_type:&BillType) {
    trace!("print::show_details()");
    println!("{}: {}", bill_type.to_string(), project.name());

    let (offer, invoice) = match project.bills() {
        Ok(tuple) => tuple,
        Err(e) => {
            error!("{}", e);
            return
        }
    };

    let bill = match *bill_type {
        BillType::Offer => offer,
        BillType::Invoice => invoice
    };

    let mut table = Table::new();
    trace!("                   - created table");
    //table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table_with_borders(&mut table);
    //table.set_titles( row![cell!(""), bill_type, cell!(project.name())]);
    //table.add_row( row![cell!(""), cell!("name"), cell!("amount"), cell!("price"), cell!("cost")]);
    trace!("                   - added a row");
    for (index,item) in bill.as_items().iter().enumerate(){
        table.add_row(
            row![ cell!((index+1).to_string()),
                  item.product.name,
                  r->item.amount.to_string(),
                  r->currency_to_string(&item.product.price),
                  r->currency_to_string(&(item.gross()))
            ]);
    }

    table.add_row( row![cell!(""), cell!("======="), cell!(r->"======"), cell!(r->"======"), cell!(r->"======")]);
    for (&tax, itemlist) in bill.iter() {
        table.add_row( row!["",
                            "",
                            "",
                            "",
                            cell!(r->itemlist.gross_sum().postfix())
        ]);
        if itemlist.tax_sum().value() > 0 {
            table.add_row( row!["",
                                "",
                                "",
                                cell!(r->format!("+{}%",*tax*100f64)),
                                cell!(r->format!("{}", itemlist.tax_sum().postfix()))
                                //cell!(r->itemlist.net_sum().postfix())
            ]);
        }
    }
    table.add_row( row!["", "Total", "", "", bill.net_total().postfix()]);

    table.printstd();

    // show times
    if let Some(events) = project.events() {
        for event in events {
            println!("{}", event);
        }
    }

    println!("{}", project.caterers());

}
