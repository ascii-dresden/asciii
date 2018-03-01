use asciii;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand, Shell};
use super::subcommands;
use std::str::FromStr;

use ::cli::error::*;

#[allow(unknown_lints, cyclomatic_complexity)]
pub fn with_cli<F> (app_handler:F) where F: Fn(App) {
    app_handler(
        App::new("asciii")
            .author(crate_authors!())
            .version(asciii::VERSION.as_ref())
            .about(lformat!("The ascii invoicer III").as_ref())
            .settings(&[AppSettings::SubcommandRequiredElseHelp,AppSettings::ColoredHelp,AppSettings::DeriveDisplayOrder])
            .after_help(asciii::DOCUMENTATION_URL)


            .subcommand(SubCommand::with_name("bootstrap")
                        .aliases(&["boot", "clone"])
                        .about(lformat!("set's up a new instance").as_ref())
                        .arg(Arg::with_name("repo")
                             .help(lformat!("Remote repository").as_ref())
                             .required(true))
                        .arg(Arg::with_name("to")
                             .help(lformat!("where to clone to").as_ref())
                             .long("to")
                             .takes_value(true)
                             .required(false))
                        .arg(Arg::with_name("editor")
                             .help(lformat!("Override the configured editor").as_ref())
                             .long("editor")
                             .takes_value(true)
                             .short("e"))

                       )


            .subcommand(SubCommand::with_name("new")
                        .about(lformat!("Create a new project").as_ref())

                        .arg(Arg::with_name("name")
                             .help(lformat!("Project name").as_ref())
                             .required(true))

                        .arg(Arg::with_name("date")
                             .help(lformat!("Manually set the date of the project").as_ref())
                             .validator(validators::is_dmy)
                             .short("d")
                             .long("date")
                             .takes_value(true))

                        .arg(Arg::with_name("description")
                             .help(lformat!("Override the description of the project").as_ref())
                             .long("desc")
                             .takes_value(true))

                        .arg(Arg::with_name("template")
                             .help(lformat!("Use a specific template").as_ref())
                             .long("template")
                             .takes_value(true)
                             .short("t"))

                        .arg(Arg::with_name("editor")
                             .help(lformat!("Override the configured editor").as_ref())
                             .long("editor")
                             .takes_value(true)
                             .short("e"))

                        .arg(Arg::with_name("manager")
                             .help(lformat!("Override the manager of the project").as_ref())
                             .long("manager")
                             .short("m")
                             .takes_value(true))

                        .arg(Arg::with_name("time")
                             .help(lformat!("Manually set the start time of the project").as_ref())
                             .long("time")
                             .takes_value(true))

                        .arg(Arg::with_name("time_end")
                             .help(lformat!("Manually set the end time of the project").as_ref())
                             .long("time_end")
                             .takes_value(true))

                        .arg(Arg::with_name("length")
                             .help(lformat!("Overrides the duration of the event").as_ref())
                             .long("length")
                             .takes_value(true))

                        .arg(Arg::with_name("don't edit")
                             .help(lformat!("Do not edit the file after creation").as_ref())
                             .long("dont"))

                        )

            .subcommand(SubCommand::with_name("list")
                        .aliases(&["ls", "dir", "la", "l", "lsit"])
                        .about(lformat!("List Projects").as_ref())

                        .arg(Arg::with_name("archive")
                             .help(lformat!("list archived projects of a specific year, defaults to the current year").as_ref())
                             .short("a")
                             .long("archive")
                             .min_values(0)
                             .takes_value(true)
                             .value_name("year")
                            )

                        .arg(Arg::with_name("year")
                             .help(lformat!("List projects from that year, archived or not").as_ref())
                             .short("y")
                             .long("year")
                             .min_values(0)
                             .takes_value(true)
                            )

                        .arg(Arg::with_name("details")
                             .help(lformat!("Add extra fields to print for each project listed").as_ref())
                             .short("d")
                             .long("details")
                             .takes_value(true)
                             .multiple(true)
                            )
                        .arg(Arg::with_name("filter")
                             .help(lformat!("Filter selection by field content").as_ref())
                             .short("f")
                             .long("filter")
                             .takes_value(true)
                             .multiple(true)
                            )
                        .arg(Arg::with_name("errors")
                             .help(lformat!("Show Errors for each project").as_ref())
                             .long("errors")
                             .short("e")
                            )
                        .arg(Arg::with_name("colors")
                             .help(lformat!("Show colors").as_ref())
                             .long("colors")
                             .short("c")
                            )
                        .arg(Arg::with_name("no colors")
                             .help(lformat!("Show colors for each project").as_ref())
                             .long("no-colors")
                             .short("n")
                             .conflicts_with("color")
                            )
                        .arg(Arg::with_name("simple")
                             .help(lformat!("Show non-verbose list").as_ref())
                             .long("simple")
                            )
                        .arg(Arg::with_name("csv")
                             .help(lformat!("Print in csv form").as_ref())
                             .long("csv")
                             .conflicts_with("simple")
                             .conflicts_with("verbose")
                            )
                        .arg(Arg::with_name("verbose")
                             .help(lformat!("Opposite of simple").as_ref())
                             .long("verbose")
                             .short("v")
                             .conflicts_with("simple")
                             .conflicts_with("csv")
                            )
                        .arg(Arg::with_name("sort")
                             .help(lformat!("Sort by :").as_ref())
                             .long("sort")
                             .short("s")
                             .possible_values(&["date",  "index",  "name",  "manager"])
                             .takes_value(true)
                            )
                        .arg(Arg::with_name("all")
                             .help(lformat!("List all projects, ever").as_ref())
                             .short("A")
                             .long("all"))

                        .arg(Arg::with_name("templates")
                             .help(lformat!("List templates").as_ref())
                             .long("templates")
                             .short("t")
                            )

                        .arg(Arg::with_name("years")
                             .help(lformat!("List years in archive").as_ref())
                             .long("years"))

                        .arg(Arg::with_name("paths")
                             .help(lformat!("List paths to each project file").as_ref())
                             .long("paths")
                             .short("p")
                            )

                        .arg(Arg::with_name("broken")
                             .help(lformat!("List broken projects  without project file").as_ref())
                             .long("broken")
                             .short("b")
                            )

                        .arg(Arg::with_name("computed_fields")
                             .help(lformat!("List all computed data fields that can be used with --details").as_ref())
                             .long("computed")
                             .short("C")
                            )

                        .arg(Arg::with_name("nothing")
                             .help(lformat!("Print nothing, expect the fields supplied via --details").as_ref())
                             .long("nothing")
                             .short("x")
                            )
                        )

            .subcommand(SubCommand::with_name("open")
                        .about(lformat!("Open storage path").as_ref())
                        .arg(Arg::with_name("templates")
                             .help(lformat!("Open path to templates instead").as_ref())
                             .long("templates")
                             .short("t")
                             .conflicts_with("output")
                             .conflicts_with("bin")
                             .conflicts_with("archive")
                             .conflicts_with("search_term")
                            )
                        .arg(Arg::with_name("output")
                             .help(lformat!("Open path to created documents instead").as_ref())
                             .long("output")
                             .short("o")
                             .conflicts_with("templates")
                             .conflicts_with("bin")
                             .conflicts_with("archive")
                             .conflicts_with("search_term")
                            )
                        .arg(Arg::with_name("bin")
                             .help(lformat!("Open path to current binary instead").as_ref())
                             .long("bin")
                             .short("b")
                             .conflicts_with("templates")
                             .conflicts_with("output")
                             .conflicts_with("archive")
                             .conflicts_with("search_term")
                            )
                        )
                        //# TODO add --invoice and --offer

            .subcommand(SubCommand::with_name("edit")
                        .aliases(&["ed"])
                        .about(lformat!("Edit a specific project").as_ref())
                        .arg(Arg::with_name("search_term")
                             .help(lformat!("Search term, possibly event name").as_ref())
                             .required(true)
                             .multiple(true)
                            )

                        .arg(Arg::with_name("archive")
                             .help(lformat!("Pick an archived project").as_ref())
                             .short("a")
                             .long("archive")
                             .takes_value(true)
                            )

                        .arg(Arg::with_name("template")
                             .help(lformat!("Edit a template file, use `list --templates` to learn which.").as_ref())
                             .short("t")
                             .long("template")
                            )

                        .arg(Arg::with_name("editor")
                             .help(lformat!("Override the configured editor").as_ref())
                             .short("e")
                             .long("editor")
                             .takes_value(true)
                            )
                        )

            .subcommand(SubCommand::with_name("meta")
                .subcommand(SubCommand::with_name("edit")
                        .about(lformat!("Edit the meta data store").as_ref())

                        .arg(Arg::with_name("template")
                             .help(lformat!("Edit a template file, use `list --templates` to learn which.").as_ref())
                             .short("t")
                             .long("template")
                            )

                        .arg(Arg::with_name("editor")
                             .help(lformat!("Override the configured editor").as_ref())
                             .short("e")
                             .long("editor")
                             .takes_value(true)
                            )
                        )
                .subcommand(SubCommand::with_name("store"))
                .subcommand(SubCommand::with_name("dump"))
                )

            .subcommand(SubCommand::with_name("archive")
                        .about(lformat!("Move a Project into the archive").as_ref())
                        .arg(Arg::with_name("search terms")
                             .help(lformat!("Search terms to match the project").as_ref())
                             //.required(true)
                             .multiple(true)
                             .conflicts_with("all")
                            )

                        .arg(Arg::with_name("force")
                             .help(lformat!("Archives the project, even though it is not completely valid").as_ref())
                             .long("force")
                             .short("F")
                            )

                        .arg(Arg::with_name("all")
                             .help(lformat!("Archives all projects that can be archived").as_ref())
                             .long("all")
                             .short("a")
                            )

                        .arg(Arg::with_name("year")
                             .help(lformat!("Override the year").as_ref())
                             .long("year")
                             .short("y")
                             .takes_value(true)
                            )
                       )

            .subcommand(SubCommand::with_name("unarchive")
                        .about(lformat!("Move a Project out of the archive").as_ref())
                        .arg(Arg::with_name("year")
                             .help(lformat!("Specify the Archiv").as_ref())
                             .required(true)
                            )
                        .arg(Arg::with_name("name")
                             .help(lformat!("The name of the project, duh!").as_ref())
                             .required(true)
                             .multiple(true)
                            )
                       )

            .subcommand(SubCommand::with_name("show")
                        .aliases(&["display"])
                        .about(lformat!("Display a specific project").as_ref())
                        .arg(Arg::with_name("search_term")
                             .help(lformat!("Search term, possibly event name").as_ref())
                             .required(true)
                             .multiple(true)
                            )

                        .arg(Arg::with_name("json")
                             .help(lformat!("Show project as JSON").as_ref())
                             .long("json")
                             .short("j"))

                        .arg(Arg::with_name("ical")
                             .help(lformat!("Show project as iCal").as_ref())
                             .long("ical")
                             .short("C"))

                        .arg(Arg::with_name("yaml")
                             .help(lformat!("Show project as raw yaml").as_ref())
                             .long("yaml"))

                        .arg(Arg::with_name("detail")
                             .help(lformat!("Shows a particular detail").as_ref())
                             .long("detail")
                             .short("d")
                             .takes_value(true)
                             )

                        .arg(Arg::with_name("archive")
                             .help(lformat!("Pick an archived project").as_ref())
                             .long("archive")
                             .min_values(0)
                             .short("a")
                             .takes_value(true)
                            )

                        .arg(Arg::with_name("empty fields")
                             .help(lformat!("Shows fields that can be filled automatically").as_ref())
                             .long("empty_fields")
                             .short("f")
                            )

                        .arg(Arg::with_name("errors")
                             .help(lformat!("Shows the errors in this project").as_ref())
                             .long("errors")
                             .short("e")
                            )

                        .arg(Arg::with_name("template")
                             .help(lformat!("Show fields in templates that are filled").as_ref())
                             .long("template")
                             .short("t")
                            )
                        //#conflicts_with: archive  # this causes a crash

                        .arg(Arg::with_name("files")
                             .help(lformat!("List files that belong to a project").as_ref())
                             .long("files"))

                        .arg(Arg::with_name("invoice")
                             .help(lformat!("Display values in invoice mode").as_ref())
                             .long("invoice")
                             .short("i")
                            )

                        .arg(Arg::with_name("offer")
                             .help(lformat!("Display values in offer mode").as_ref())
                             .long("offer")
                             .short("o")
                            )

                        //.arg(Arg::with_name("hours") //# what used to be --caterers
                        //     .help(lformat!("Display hours").as_ref())
                        //     .long("hours")
                        //    )

                        .arg(Arg::with_name("csv")
                             .help(lformat!("Show as csv").as_ref())
                             .long("csv")
                             .short("c")
                            )

                        //.arg(Arg::with_name("markdown")
                        //     .help(lformat!("Show as markdown").as_ref())
                        //     .long("markdown")
                        //     .short("m")
                        //    )
                    )

            .subcommand(SubCommand::with_name("set")
                        .aliases(&["ed"])
                        .about(lformat!("Set a value in a project file").as_ref())
                        .arg(Arg::with_name("search_term")
                             .help(lformat!("Search term, possibly event name").as_ref())
                             .required(true)
                            )

                        .arg(Arg::with_name("field name")
                             .help(lformat!("Which field to set").as_ref())
                             .required(true)
                            )

                        .arg(Arg::with_name("field value")
                             .help(lformat!("What to put in the field").as_ref())
                            )

                        .arg(Arg::with_name("archive")
                             .help(lformat!("Pick an archived project").as_ref())
                             .short("a")
                             .long("archive")
                             .min_values(0)
                             .takes_value(true)
                            )
                        )

            .subcommand(SubCommand::with_name("path")
                        .about(lformat!("Show storage path").as_ref())
                        .arg(Arg::with_name("templates")
                             .help(lformat!("Shows templates path instead").as_ref())
                             .long("templates")
                             .short("t")
                             .conflicts_with("output")
                             .conflicts_with("bin")
                             .conflicts_with("archive")
                             .conflicts_with("search_term")
                            )
                        .arg(Arg::with_name("output")
                             .help(lformat!("Shows path to created documents instead").as_ref())
                             .long("output")
                             .short("o")
                             .conflicts_with("templates")
                             .conflicts_with("bin")
                             .conflicts_with("archive")
                             .conflicts_with("search_term")
                            )
                        .arg(Arg::with_name("bin")
                             .help(lformat!("Open path to current binary instead").as_ref())
                             .long("bin")
                             .short("b")
                             .conflicts_with("templates")
                             .conflicts_with("output")
                             .conflicts_with("archive")
                             .conflicts_with("search_term")
                            )
                        )

            .subcommand(SubCommand::with_name("workspace")
                        .aliases(&["ws"])
                         .arg(Arg::with_name("archive")
                             .help(lformat!("Open an archive instead").as_ref())
                             .short("a")
                             .long("archive")
                             .takes_value(true)
                            )

                        .arg(Arg::with_name("editor")
                             .help(lformat!("Override the configured editor").as_ref())
                             .short("e")
                             .long("editor")
                             .takes_value(true)
                            )
                        .about(lformat!("Open the working directory in an editor").as_ref())
                        )



            .subcommand(SubCommand::with_name("csv")
                        .about(lformat!("Produces a CSV report for a given year").as_ref())
                        .arg(Arg::with_name("year")
                             .help(lformat!("List projects from that year, archived or not").as_ref())
                             //.short("y")
                             //.long("year")
                             .validator(|y| y.parse::<i32>().map(|_ok|()).map_err(|e|e.to_string()))
                             .takes_value(true)
                            )
                        )

            .subcommand(SubCommand::with_name("calendar")
                        .aliases(&["cal","ical","ics","kalender"])
                        .arg(Arg::with_name("archive")
                             .help(lformat!("List archived projects of a specific year, defaults to the current year").as_ref())
                             .short("a")
                             .long("archive")
                             .min_values(0)
                             .takes_value(true)
                             .value_name("year")
                            )

                        .arg(Arg::with_name("tasks")
                             .help(lformat!("Include open tasks").as_ref())
                             .short("t")
                             .long("tasks")
                            )

                        .arg(Arg::with_name("year")
                             .help(lformat!("List projects from that year, archived or not").as_ref())
                             .short("y")
                             .long("year")
                             .min_values(0)
                             .takes_value(true)
                            )

                        .arg(Arg::with_name("all")
                             .help(lformat!("List all projects, ever").as_ref())
                             .short("A")
                             .long("all"))
                       )

            .subcommand(SubCommand::with_name("dues")
                        .about(lformat!("Experimental: open dues").as_ref())

                        .arg(Arg::with_name("invoices")
                             .help(lformat!("Show unpayed wages").as_ref())
                             .long("invoices")
                             .short("i")
                            )

                        .arg(Arg::with_name("wages")
                             .help(lformat!("Show unpayed wages").as_ref())
                             .long("wages")
                             .short("w")
                             .conflicts_with("invoices")
                            )
                       )

            .subcommand(SubCommand::with_name("make")
                        .about(lformat!("Creates documents from projects").as_ref())
                        .aliases(&["mk"])

                        .arg(Arg::with_name("file")
                             .help(lformat!("Manually pass a file path").as_ref())
                             .long("file")
                             .takes_value(true)
                            )

                        .arg(Arg::with_name("output")
                             .help(lformat!("Manually pass a output folder").as_ref())
                             .long("output")
                             .takes_value(true)
                            )

                        .arg(Arg::with_name("force")
                             .help(lformat!("Do it against better judgement").as_ref())
                             .long("force")
                            )

                        .arg(Arg::with_name("dry-run")
                             .help(lformat!("Do not create final output file").as_ref())
                             .short("d")
                             .long("dry")
                            )

                        .arg(Arg::with_name("print-only")
                             .help(lformat!("Only prints to stdout").as_ref())
                             .long("print")
                            )

                        .arg(Arg::with_name("open")
                             .help(lformat!("Open the pdf file afterwards.").as_ref())
                             .long("open")
                            )

                        .arg(Arg::with_name("search_term")
                             .help(lformat!("Search term, possibly event name").as_ref())
                             .multiple(true)
                             )

                        .arg(Arg::with_name("offer")
                             .help(lformat!("Produce an offer document").as_ref())
                             .long("offer")
                             .conflicts_with("invoice")
                             )

                        .arg(Arg::with_name("invoice")
                             .help(lformat!("Produce an invoice document").as_ref())
                             .long("invoice")
                             )

                        .arg(Arg::with_name("archive")
                             .help(lformat!("Pick an archived project").as_ref())
                             .short("a")
                             .long("archive")
                             .min_values(0)
                             .takes_value(true)
                            )

                        .arg(Arg::with_name("template")
                             .help(lformat!("Use a particular template").as_ref())
                             .short("t")
                             .long("template")
                             .takes_value(true)
                             )
                       )

            .subcommand(SubCommand::with_name("delete")
                        .about(lformat!("Deletes a project").as_ref())
                        .aliases(&["rm"])

                        .arg(Arg::with_name("dry-run")
                             .help(lformat!("Do not create final output file").as_ref())
                             .short("d")
                             .long("dry")
                            )

                        .arg(Arg::with_name("search_term")
                             .help(lformat!("Search term, possibly event name").as_ref())
                             .required(true)
                             .multiple(true))

                        .arg(Arg::with_name("archive")
                             .help(lformat!("list archived projects").as_ref())
                             .short("a")
                             .long("archive")
                             .min_values(0)
                             .takes_value(true)
                            )
                        //.arg(Arg::with_name("template")
                        //     .help(lformat!("A template").as_ref())
                        //     .short("t")
                        //     .long("template")
                        //    )
                       )


            .subcommand(SubCommand::with_name("config")
                        .aliases(&["settings"])
                        .about(lformat!("Show and edit your config").as_ref())
                        .arg(Arg::with_name("edit")
                             .help(lformat!("Edit your config").as_ref())
                             .short("e")
                             .long("edit")
                            )

                        .arg(Arg::with_name("editor")
                             .help(lformat!("Override the configured editor").as_ref())
                             .long("editor")
                             .takes_value(true)
                             )

                        .arg(Arg::with_name("show")
                             .help(lformat!("Show a specific config value").as_ref())
                             .short("s")
                             .long("show")
                             .takes_value(true)
                            )

                        .arg(Arg::with_name("default")
                             .help(lformat!("Show default config").as_ref())
                             .short("d")
                             .long("default")
                            )

                        .arg(Arg::with_name("set root")
                             .help(lformat!("set the root folder in the config").as_ref())
                             .long("set-root")
                             .takes_value(true)
                            )

                        .arg(Arg::with_name("location")
                             .help(lformat!("Show the location of the config file").as_ref())
                             .short("l")
                             .long("location")
                            )

                        // TODO unimplemented!()
                        .arg(Arg::with_name("init")
                             .help(lformat!("Create config file.").as_ref())
                             .short("i")
                             .long("init")
                            )

                        )

            .subcommand(SubCommand::with_name("shell")
                        .aliases(&["sh", "repl"])
                        .about(lformat!("(experimental) starts interactive shell").as_ref())
                       )

            .subcommand(SubCommand::with_name("whoami")
                        .about(lformat!("Show your name from config").as_ref())
                       )

            //# GIT STUFF
            .subcommand(SubCommand::with_name("status")
                        .about(lformat!("Show the working tree status").as_ref())
                        .aliases(&["st"])
                       )

            .subcommand(SubCommand::with_name("pull")
                        .about(lformat!("Pull and merge new commits from remote").as_ref())
                        .aliases(&["update"])
                        .arg(Arg::with_name("rebase")
                             .help(lformat!("git pull with --rebase").as_ref())
                             .long("rebase")
                            )
                       )

            .subcommand(SubCommand::with_name("diff")
                        .about(lformat!("git diff").as_ref())
                        .arg(Arg::with_name("search_term")
                             .help(lformat!("Search term, possibly event name").as_ref())
                             .multiple(true)
                            )
                        .arg(Arg::with_name("archive")
                             .help(lformat!("list archived projects").as_ref())
                             .short("a")
                             .long("archive")
                             .min_values(0)
                             .takes_value(true)
                            )
                        .arg(Arg::with_name("template")
                             .help(lformat!("A template").as_ref())
                             .short("t")
                             .long("template")
                            )
                       )

            .subcommand(SubCommand::with_name("add")
                        .about(lformat!("Add file contents to the git-index").as_ref())
                        .arg(Arg::with_name("search_term")
                             .help(lformat!("Search term, possibly event name").as_ref())
                             .multiple(true)
                            )
                        .arg(Arg::with_name("archive")
                             .help(lformat!("list archived projects").as_ref())
                             .short("a")
                             .long("archive")
                             .min_values(0)
                             .takes_value(true)
                            )
                        .arg(Arg::with_name("template")
                             .help(lformat!("A template").as_ref())
                             .short("t")
                             .long("template")
                            )
                        .arg(Arg::with_name("all")
                             .help(lformat!("Add all projects").as_ref())
                             .short("A")
                             .long("all"))

                       )

            .subcommand(SubCommand::with_name("commit")
                        .aliases(&["cm"])
                        .about(lformat!("Save changes locally").as_ref())
                       )

            .subcommand(SubCommand::with_name("push")
                        .about(lformat!("Upload locally saved changes to the remote").as_ref())
                       )

            .subcommand(SubCommand::with_name("cleanup")
                        .about(lformat!("cleans changes and untracked files in project folder").as_ref())
                        .arg(Arg::with_name("search_term")
                             .help(lformat!("Search term, possibly event name").as_ref())
                             .multiple(true)
                             .required(true)
                            )
                        .arg(Arg::with_name("archive")
                             .help(lformat!("list archived projects").as_ref())
                             .short("a")
                             .long("archive")
                             .min_values(0)
                             .takes_value(true)
                            )
                        .arg(Arg::with_name("template")
                             .help(lformat!("A template").as_ref())
                             .short("t")
                             .long("template")
                            )
                       )

            .subcommand(SubCommand::with_name("stash").about(lformat!("equals git stash").as_ref()))
            .subcommand(SubCommand::with_name("pop").about(lformat!("equals git pop").as_ref()))

            .subcommand(SubCommand::with_name("log")
                        .aliases(&["lg", "hist", "history"])
                        .about(lformat!("Show commit logs").as_ref())
                        .arg(Arg::with_name("search_term")
                             .help(lformat!("Search term, possibly event name").as_ref())
                             .multiple(true)
                            )
                        .arg(Arg::with_name("archive")
                             .help(lformat!("list archived projects").as_ref())
                             .short("a")
                             .long("archive")
                             .min_values(0)
                             .takes_value(true)
                            )
                        .arg(Arg::with_name("template")
                             .help(lformat!("A template").as_ref())
                             .short("t")
                             .long("template")
                            )
                       )

            .subcommand(SubCommand::with_name("remote")
                        .about(lformat!("Show information about the remote").as_ref())
                       )

            .subcommand(SubCommand::with_name("complete")
                        //.aliases(&["lg", "hist", "history"])
                        .about(lformat!("Generates completion for bash, zsh, etc").as_ref())
                        .arg(Arg::with_name("shell")
                             .help(lformat!("what shell to generate completion for (bash, zsh, fish,PowerShell)").as_ref())
                             .min_values(0)
                             .takes_value(true)
                             .value_name("shell")
                            )

                       )

            .subcommand(SubCommand::with_name("version")
                .about(lformat!("Prints version of this tool").as_ref())
            )

            .subcommand(SubCommand::with_name("doc")
                .about(lformat!("Opens the online documentation, please read it").as_ref())
            )

        );
}

/// Starting point for handling commandline matches
pub fn match_matches(matches: &ArgMatches) {
    let res = match matches.subcommand() {
     ("bootstrap", Some(sub_m)) => subcommands::bootstrap(sub_m),
     ("list",      Some(sub_m)) => subcommands::list(sub_m),
     ("csv",       Some(sub_m)) => subcommands::csv(sub_m),
     ("new",       Some(sub_m)) => subcommands::new(sub_m),
     ("edit",      Some(sub_m)) => subcommands::edit(sub_m),
     ("meta",      Some(sub_m)) => subcommands::meta(sub_m),
     ("workspace", Some(sub_m)) => subcommands::workspace(sub_m),
     ("set",       Some(sub_m)) => subcommands::set(sub_m),
     ("show",      Some(sub_m)) => subcommands::show(sub_m),
     ("calendar",  Some(sub_m)) => subcommands::calendar(sub_m),
     ("archive",   Some(sub_m)) => subcommands::archive(sub_m),
     ("unarchive", Some(sub_m)) => subcommands::unarchive(sub_m),
     ("config",    Some(sub_m)) => subcommands::config(sub_m),
     ("whoami",    _          ) => subcommands::config_show("user/name"),

     ("path",      Some(sub_m)) => subcommands::show_path(sub_m),
     ("open",      Some(sub_m)) => subcommands::open_path(sub_m),

     ("make",      Some(sub_m)) => subcommands::make(sub_m),
     ("delete",    Some(sub_m)) => subcommands::delete(sub_m),
     ("spec",      Some(sub_m)) => subcommands::spec(sub_m),

     ("doc",       _          ) => subcommands::doc(),
     ("version",   _          ) => subcommands::version(),

     ("dues",      Some(sub_m)) => subcommands::dues(sub_m),
     ("shell",     Some(sub_m)) => subcommands::shell(sub_m),

     ("remote",    _          ) => subcommands::git_remote(),
     ("pull",      Some(sub_m)) => subcommands::git_pull(sub_m),
     ("diff",      Some(sub_m)) => subcommands::git_diff(sub_m),
     ("cleanup",   Some(sub_m)) => subcommands::git_cleanup(sub_m),
     ("status",    _          ) => subcommands::git_status(),
     ("add",       Some(sub_m)) => subcommands::git_add(sub_m),
     ("commit",    _          ) => subcommands::git_commit(),
     ("push",      _          ) => subcommands::git_push(),
     ("stash",     _          ) => subcommands::git_stash(),
     ("pop",       _          ) => subcommands::git_stash_pop(),
     ("log",       Some(sub_m)) => subcommands::git_log(sub_m),
     ("complete",  Some(sub_m)) => generate_completions(sub_m),
     _                          => Ok(())
    };
    if let Err(e) = res {
        println!("{}", e)
    }
}

pub fn generate_completions(matches: &ArgMatches) -> Result<()>{
    if let Some(shell) = matches.value_of("shell").and_then(|s|Shell::from_str(s).ok()) {
        with_cli(|mut app| app.gen_completions("asciii", shell, ".") );
    } else {
        error!("{}", lformat!("please specify either bash, zsh, fish or powershell"));
    }
    Ok(())
}

pub mod validators {
    use asciii::util::yaml::parse_dmy_date;

    pub fn is_dmy(val: String) -> Result<(), String> {
        match parse_dmy_date(&val) {
            Some(_) => Ok(()),
            None => Err(lformat!("Date Format must be DD.MM.YYYY")),
        }
    }
}
