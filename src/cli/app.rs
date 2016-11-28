use asciii;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use super::subcommands;

pub fn build_cli() -> App<'static, 'static> {
    App::new("asciii")
        .author(crate_authors!())
        .version(asciii::VERSION.as_ref())
        .about("The ascii invoicer III")
        .settings(&[AppSettings::SubcommandRequiredElseHelp,AppSettings::ColoredHelp])
        .after_help(asciii::DOCUMENTATION_URL)

        .subcommand(SubCommand::with_name("doc")
            .about("Opens the online documentation, please read it")
        )

        .subcommand(SubCommand::with_name("version")
            .about("Prints version of this tool")
        )

        .subcommand(SubCommand::with_name("new")
                    .about("Create a new project")

                    .arg(Arg::with_name("name")
                         .help("Project name")
                         .required(true))

                    .arg(Arg::with_name("date")
                         .help("Manually set the date of the project")
                         .validator(validators::is_dmy)
                         .short("d")
                         .long("date")
                         .takes_value(true))

                    .arg(Arg::with_name("description")
                         .help("Override the description of the project")
                         .long("desc")
                         .takes_value(true))

                    .arg(Arg::with_name("template")
                         .help("Use a specific template")
                         .long("template")
                         .short("t"))

                    .arg(Arg::with_name("editor")
                         .help("Override the configured editor")
                         .long("editor")
                         .short("e"))

                    .arg(Arg::with_name("manager")
                         .help("Override the manager of the project")
                         .long("manager")
                         .short("m")
                         .takes_value(true))

                    .arg(Arg::with_name("time")
                         .help("Manually set the start time of the project")
                         .long("time")
                         .takes_value(true))

                    .arg(Arg::with_name("time_end")
                         .help("Manually set the end time of the project")
                         .long("time_end")
                         .takes_value(true))

                    .arg(Arg::with_name("length")
                         .help("Overrides the duration of the event")
                         .long("length")
                         .takes_value(true))

                    .arg(Arg::with_name("don't edit")
                         .help("Do not edit the file after creation")
                         .long("dont"))

                    )

        .subcommand(SubCommand::with_name("list")
                    .aliases(&["ls", "dir", "la", "l", "lsit"])
                    .about("List Projects")

                    .arg(Arg::with_name("archive")
                         .help("list archived projects of a specific year, defaults to the current year")
                         .short("a")
                         .long("archive")
                         .min_values(0)
                         .takes_value(true)
                         .value_name("year")
                        )
                    .arg(Arg::with_name("year")
                         .help("List projects from that year, archived or not")
                         .short("y")
                         .long("year")
                         .min_values(0)
                         .takes_value(true)
                        )

                    .arg(Arg::with_name("details")
                         .help("Add extra fields to print for each project listed")
                         .short("d")
                         .long("details")
                         .takes_value(true)
                         .multiple(true)
                        )
                    .arg(Arg::with_name("filter")
                         .help("List of fields to print for each project listed")
                         .short("f")
                         .long("filter")
                         .takes_value(true)
                         .multiple(true)
                        )
                    .arg(Arg::with_name("errors")
                         .help("Show Errors for each project")
                         .long("errors")
                         .short("e")
                        )
                    .arg(Arg::with_name("colors")
                         .help("Show colors for each project")
                         .long("colors")
                         .short("c")
                        )
                    .arg(Arg::with_name("no colors")
                         .help("Show colors for each project")
                         .long("no-colors")
                         .short("n")
                         .conflicts_with("color")
                        )
                    .arg(Arg::with_name("simple")
                         .help("Use simple list")
                         .long("simple")
                        )
                    .arg(Arg::with_name("csv")
                         .help("Print in csv form")
                         .long("csv")
                         .conflicts_with("simple")
                         .conflicts_with("verbose")
                        )
                    .arg(Arg::with_name("verbose")
                         .help("Opposite of simple")
                         .long("verbose")
                         .short("v")
                         .conflicts_with("simple")
                         .conflicts_with("csv")
                        )
                    .arg(Arg::with_name("sort")
                         .help("sort by")
                         .long("sort")
                         .short("s")
                         .possible_values(&["date",  "index",  "name",  "manager"])
                         .takes_value(true)
                        )
                    .arg(Arg::with_name("all")
                         .help("List all projects, ever")
                         .short("A")
                         .long("all"))

                    .arg(Arg::with_name("templates")
                         .help("List templates")
                         .long("templates")
                         .short("t")
                        )

                    .arg(Arg::with_name("years")
                         .help("List years in archive")
                         .long("years"))

                    .arg(Arg::with_name("paths")
                         .help("List paths to each project file")
                         .long("paths")
                         .short("p")
                        )

                    //.arg(Arg::with_name("broken")
                    //     .help("List broken projects  without project file")
                    //     .long("broken")
                    //     .short("b")
                    //    )

                    .arg(Arg::with_name("computed_fields")
                         .help("List all computed data fields that can be used with --details")
                         .long("computed")
                         .short("C")
                        )

                    .arg(Arg::with_name("nothing")
                         .help("Print nothing, expect the fields supplied via --details")
                         .long("nothing")
                         .short("x")
                        )
                    )

        .subcommand(SubCommand::with_name("csv")
                    .about("Produces a CSV report for a given year")
                    .arg(Arg::with_name("year")
                         .help("List projects from that year, archived or not")
                         //.short("y")
                         //.long("year")
                         .validator(|y| y.parse::<i32>().map(|_ok|()).map_err(|e|e.to_string()))
                         .takes_value(true)
                        )
                    )

        .subcommand(SubCommand::with_name("archive")
                    .about("Move a Project into the archive")
                    .arg(Arg::with_name("search terms")
                         .help("Search terms to match the project")
                         .required(true)
                         .multiple(true)
                        )

                    .arg(Arg::with_name("force")
                         .help("Archives the Project, even though it is not completely valid")
                         .long("force")
                         .short("f")
                        )

                    .arg(Arg::with_name("year")
                         .help("Override the year")
                         .long("year")
                         .short("y")
                         .takes_value(true)
                        )
                   )

        .subcommand(SubCommand::with_name("unarchive")
                    .about("Move a Project out of the archive")
                    .arg(Arg::with_name("year")
                         .help("Specify the Archiv")
                         .required(true)
                        )
                    .arg(Arg::with_name("name")
                         .help("The name of the project, duh!")
                         .required(true)
                         .multiple(true)
                        )
                   )

        .subcommand(SubCommand::with_name("path")
                    .about("Show storage path")
                    .arg(Arg::with_name("templates")
                         .help("Open path to templates instead")
                         .long("templates")
                         .short("t")
                         .conflicts_with("output")
                         .conflicts_with("bin")
                         .conflicts_with("archive")
                         .conflicts_with("search_term")
                        )
                    .arg(Arg::with_name("output")
                         .help("Open path to created documents instead")
                         .long("output")
                         .short("o")
                         .conflicts_with("templates")
                         .conflicts_with("bin")
                         .conflicts_with("archive")
                         .conflicts_with("search_term")
                        )
                    .arg(Arg::with_name("bin")
                         .help("Open path to current binary instead")
                         .long("bin")
                         .short("b")
                         .conflicts_with("templates")
                         .conflicts_with("output")
                         .conflicts_with("archive")
                         .conflicts_with("search_term")
                        )
                    )

        .subcommand(SubCommand::with_name("open")
                    .about("Open storage path")
//                    .arg(Arg::with_name("search_term")
//                         .help("Search term, possibly event name")
//                         .multiple(true)
//                        )
//
//                    .arg(Arg::with_name("archive")
//                         .help("Pick an archived project")
//                         .short("a")
//                         .long("archive")
//                    .min_values(0)
//                         .takes_value(true)
//                        )
//
//                    .arg(Arg::with_name("invoice")
//                         .help("Open values resulting invoice file. Makes it if necessary")
//                         .long("invoice")
//                         .short("i")
//                        )
//
//                    .arg(Arg::with_name("offer")
//                         .help("Open values resulting offer file. Makes it if necessary")
//                         .long("offer")
//                         .short("o")
//                        )
//
//
                    .arg(Arg::with_name("templates")
                         .help("Open path to templates instead")
                         .long("templates")
                         .short("t")
                         .conflicts_with("output")
                         .conflicts_with("bin")
                         .conflicts_with("archive")
                         .conflicts_with("search_term")
                        )
                    .arg(Arg::with_name("output")
                         .help("Open path to created documents instead")
                         .long("output")
                         .short("o")
                         .conflicts_with("templates")
                         .conflicts_with("bin")
                         .conflicts_with("archive")
                         .conflicts_with("search_term")
                        )
                    .arg(Arg::with_name("bin")
                         .help("Open path to current binary instead")
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
                    .about("Edit a specific project")
                    .arg(Arg::with_name("search_term")
                         .help("Search term, possibly event name")
                         .required(true)
                         .multiple(true)
                        )

                    .arg(Arg::with_name("archive")
                         .help("Pick an archived project")
                         .short("a")
                         .long("archive")
                         .min_values(0)
                         .takes_value(true)
                        )

                    .arg(Arg::with_name("template")
                         .help("Edit a template file, use `list --templates` to learn which.")
                         .short("t")
                         .long("template")
                        )

                    .arg(Arg::with_name("editor")
                         .help("Override the configured editor")
                         .short("e")
                         .long("editor")
                         .takes_value(true)
                        )
                    )

        .subcommand(SubCommand::with_name("set")
                    .aliases(&["ed"])
                    .about("Set a value in a project")
                    .arg(Arg::with_name("search_term")
                         .help("Search term, possibly event name")
                         .required(true)
                        )

                    .arg(Arg::with_name("field name")
                         .help("Which field to set")
                         .required(true)
                        )

                    .arg(Arg::with_name("field value")
                         .help("What to put in the field")
                        )

                    .arg(Arg::with_name("archive")
                         .help("Pick an archived project")
                         .short("a")
                         .long("archive")
                         .min_values(0)
                         .takes_value(true)
                        )


                   )

        .subcommand(SubCommand::with_name("show")
                    .aliases(&["display"])
                    .about("Display a specific project")
                    .arg(Arg::with_name("search_term")
                         .help("Search term, possibly event name")
                         .required(true)
                         .multiple(true)
                        )

                    .arg(Arg::with_name("json")
                         .help("Show project as JSON")
                         .long("json")
                         .short("j"))

                    .arg(Arg::with_name("ical")
                         .help("Show project as iCal")
                         .long("ical")
                         .short("C"))

                    .arg(Arg::with_name("dump")
                         .help("Dump project yaml")
                         .long("dump"))

                    .arg(Arg::with_name("detail")
                         .help("Shows a particular detail")
                         .long("detail")
                         .short("d")
                         .takes_value(true)
                         )

                    .arg(Arg::with_name("archive")
                         .help("Pick an archived project")
                         .long("archive")
                         .min_values(0)
                         .short("a")
                         .takes_value(true)
                        )

                    .arg(Arg::with_name("empty fields")
                         .help("shows fields that can still be filled")
                         .long("empty_fields")
                         .short("f")
                        )

                    .arg(Arg::with_name("errors")
                         .help("Shows the errors in this project")
                         .long("errors")
                         .short("e")
                        )

                    .arg(Arg::with_name("template")
                         .help("Show show fields in templates that are filled")
                         .long("template")
                         .short("t")
                        )
                    //#conflicts_with: archive  # this causes a crash

                    .arg(Arg::with_name("files")
                         .help("List files that belong to a project")
                         .long("files"))

                    .arg(Arg::with_name("invoice")
                         .help("Display values in invoice mode")
                         .long("invoice")
                         .short("i")
                        )

                    .arg(Arg::with_name("offer")
                         .help("Display values in offer mode")
                         .long("offer")
                         .short("o")
                        )

                    .arg(Arg::with_name("hours") //# what used to be --caterers
                         .help("Display hours")
                         .long("hours")
                        )

                    .arg(Arg::with_name("csv")
                         .help("Show as csv")
                         .long("csv")
                         .short("c")
                        )

                    .arg(Arg::with_name("markdown")
                         .help("Show as markdown")
                         .long("markdown")
                         .short("m")
                        )
                )

        .subcommand(SubCommand::with_name("calendar")
                    .arg(Arg::with_name("archive")
                         .help("list archived projects of a specific year, defaults to the current year")
                         .short("a")
                         .long("archive")
                         .min_values(0)
                         .takes_value(true)
                         .value_name("year")
                        )
                    .arg(Arg::with_name("year")
                         .help("List projects from that year, archived or not")
                         .short("y")
                         .long("year")
                         .min_values(0)
                         .takes_value(true)
                        )

                    .arg(Arg::with_name("all")
                         .help("List all projects, ever")
                         .short("A")
                         .long("all"))
                   )

        .subcommand(SubCommand::with_name("spec")
                    .about("runs full spec on all projects")
                   )

        .subcommand(SubCommand::with_name("make")
                    .about("Creates documents from projects")
                    .aliases(&["mk"])

                    .arg(Arg::with_name("force")
                         .help("Do it against better judgement")
                         .short("f")
                         .long("force")
                        )

                    .arg(Arg::with_name("dry-run")
                         .help("Do not create final output file")
                         .short("d")
                         .long("dry")
                        )

                    .arg(Arg::with_name("search_term")
                         .help("Search term, possibly event name")
                         .required(true)
                         .multiple(true))

                    .arg(Arg::with_name("offer")
                         .help("Produce an offer document")
                         .short("o")
                         .long("offer")
                         .conflicts_with("invoice")
                         )

                    .arg(Arg::with_name("invoice")
                         .help("Produce an invoice document")
                         .short("i")
                         .long("invoice")
                         )

                    .arg(Arg::with_name("template")
                         .help("Use a particular template")
                         .short("t")
                         .long("template")
                         .takes_value(true)
                         )
                   )

        .subcommand(SubCommand::with_name("delete")
                    .about("Deletes a project")
                    .aliases(&["rm"])

                    .arg(Arg::with_name("dry-run")
                         .help("Do not create final output file")
                         .short("d")
                         .long("dry")
                        )

                    .arg(Arg::with_name("search_term")
                         .help("Search term, possibly event name")
                         .required(true)
                         .multiple(true))

                    .arg(Arg::with_name("archive")
                         .help("list archived projects")
                         .short("a")
                         .long("archive")
                         .min_values(0)
                         .takes_value(true)
                        )
                    //.arg(Arg::with_name("template")
                    //     .help("A template")
                    //     .short("t")
                    //     .long("template")
                    //    )
                   )


        .subcommand(SubCommand::with_name("config")
                    .aliases(&["settings"])
                    .about("Show and edit your config")
                    .arg(Arg::with_name("edit")
                         .help("Edit your config")
                         .short("e")
                         .long("edit")
                        )

                    .arg(Arg::with_name("show")
                         .help("Show a specific config value")
                         .short("s")
                         .long("show")
                         .takes_value(true)
                        )

                    .arg(Arg::with_name("default")
                         .help("Show default config")
                         .short("d")
                         .long("default")
                        )

                    .arg(Arg::with_name("location")
                         .help("Show default config")
                         .short("l")
                         .long("location")
                        )

                    // TODO unimplemented!()
                    .arg(Arg::with_name("init")
                         .help("Create config file.")
                         .short("i")
                         .long("init")
                        )

                    )


        .subcommand(SubCommand::with_name("whoami")
                    .about("Show your name from config")
                   )

        .subcommand(SubCommand::with_name("dues")
                    .about("Experimental: open does")

                    .arg(Arg::with_name("invoices")
                         .help("Show unpayed wages")
                         .long("invoices")
                         .short("i")
                        )

                    .arg(Arg::with_name("wages")
                         .help("Show unpayed wages")
                         .long("wages")
                         .short("w")
                         .conflicts_with("invoices")
                        )

                   )

        //# GIT STUFF
        .subcommand(SubCommand::with_name("status")
                    .about("Show the working tree status")
                    .aliases(&["st"])
                   )

        .subcommand(SubCommand::with_name("pull")
                    .about("Pull and merge new commits from remote")
                    .aliases(&["update"])
                    .arg(Arg::with_name("rebase")
                         .help("git pull with --rebase")
                         .long("rebase")
                        )
                   )

        .subcommand(SubCommand::with_name("shell")
                    .about("(experimental) starts interactive shell")
                   )

        .subcommand(SubCommand::with_name("diff")
                    .about("git diff")
                    .arg(Arg::with_name("search_term")
                         .help("Search term, possibly event name")
                         .multiple(true)
                        )
                    .arg(Arg::with_name("archive")
                         .help("list archived projects")
                         .short("a")
                         .long("archive")
                         .min_values(0)
                         .takes_value(true)
                        )
                    .arg(Arg::with_name("template")
                         .help("A template")
                         .short("t")
                         .long("template")
                        )
                   )

        .subcommand(SubCommand::with_name("cleanup")
                    .about("cleans changes and untracked files in project folder")
                    .arg(Arg::with_name("search_term")
                         .help("Search term, possibly event name")
                         .multiple(true)
                         .required(true)
                        )
                    .arg(Arg::with_name("archive")
                         .help("list archived projects")
                         .short("a")
                         .long("archive")
                         .min_values(0)
                         .takes_value(true)
                        )
                    .arg(Arg::with_name("template")
                         .help("A template")
                         .short("t")
                         .long("template")
                        )
                   )

        .subcommand(SubCommand::with_name("add")
                    .about("Add file contents to the index")
                    .arg(Arg::with_name("search_term")
                         .help("Search term, possibly event name")
                         .required(true)
                         .multiple(true)
                        )
                    .arg(Arg::with_name("archive")
                         .help("list archived projects")
                         .short("a")
                         .long("archive")
                         .min_values(0)
                         .takes_value(true)
                        )
                    .arg(Arg::with_name("template")
                         .help("A template")
                         .short("t")
                         .long("template")
                        )

                   )
        .subcommand(SubCommand::with_name("commit")
                    .aliases(&["cm"])
                    .about("Save changes locally")
                   )
        .subcommand(SubCommand::with_name("remote")
                    .about("Show information about the remote")
                   )

        .subcommand(SubCommand::with_name("push")
                    .about("Upload locally saved changes to the remote")
                   )

        .subcommand(SubCommand::with_name("stash").about(""))
        .subcommand(SubCommand::with_name("pop").about(""))

        .subcommand(SubCommand::with_name("log")
                    .aliases(&["lg", "hist", "history"])
                    .about("Show commit logs")
                   )
}

/// Starting point for handling commandline matches
pub fn match_matches(matches: &ArgMatches) {
    match matches.subcommand() {
     ("list",      Some(sub_m)) => subcommands::list(sub_m),
     ("csv",       Some(sub_m)) => subcommands::csv(sub_m),
     ("new",       Some(sub_m)) => subcommands::new(sub_m),
     ("edit",      Some(sub_m)) => subcommands::edit(sub_m),
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
     ("shell",     Some(sub_m)) => subcommands::shell(sub_m, build_cli()),

     ("remote",    _          ) => subcommands::git_remote(),
     ("pull",      Some(sub_m)) => subcommands::git_pull(sub_m),
     ("diff",      Some(sub_m)) => subcommands::git_diff(sub_m),
     ("cleanup",   Some(sub_m)) => subcommands::git_cleanup(sub_m),
     ("status",    _          ) => subcommands::git_status(),
     ("add",       Some(sub_m)) => subcommands::git_add(sub_m),
     //("unadd",     Some(sub_m)) => subcommands::git_unadd(sub_m),
     ("commit",    _          ) => subcommands::git_commit(),
     ("push",      _          ) => subcommands::git_push(),
     ("stash",     _          ) => subcommands::git_stash(),
     ("pop",       _          ) => subcommands::git_stash_pop(),
     ("log",       _          ) => subcommands::git_log(),
     _                          => ()
    }
}

pub mod validators {
    use asciii::util::yaml::parse_dmy_date;

    pub fn is_dmy(val: String) -> Result<(), String> {
        match parse_dmy_date(&val) {
            Some(_) => Ok(()),
            None => Err(String::from("Date Format must be DD.MM.YYYY")),
        }
    }
}
