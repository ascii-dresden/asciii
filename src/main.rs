#![cfg_attr(feature = "lints", allow(unstable_features))]
#![cfg_attr(feature = "lints", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]

#![cfg_attr(feature = "nightly", feature(alloc_system))]

#[feature(deprecated)]

#[cfg(feature = "nightly")]
extern crate alloc_system;

extern crate open;
extern crate yaml_rust;

#[macro_use]
extern crate clap;
use clap::App;

extern crate asciii;
use asciii::cli::subcommands;


pub fn setup_app(){
    //let cli_setup = init_matches(); //TODO Font forget this in production
    let cli_setup = load_yaml!("cli/cli.yml");


    let matches = App::from_yaml(&cli_setup)
        .version(&crate_version!()[..])
        .get_matches();

    match matches.subcommand() {
     ("list",      Some(sub_m)) => subcommands::list(&sub_m),
     ("new",       Some(sub_m)) => subcommands::new(&sub_m),
     ("edit",      Some(sub_m)) => subcommands::edit(&sub_m),
     ("show",      Some(sub_m)) => subcommands::show(&sub_m),
     ("archive",   Some(sub_m)) => subcommands::archive(&sub_m),
     ("unarchive", Some(sub_m)) => subcommands::unarchive(&sub_m),
     ("config",    Some(sub_m)) => subcommands::config(&sub_m),
     ("whoami",    _          ) => subcommands::config_show("storage_name"),

     ("path",      Some(sub_m)) => subcommands::show_path(sub_m),
     ("open",      Some(sub_m)) => subcommands::open_path(sub_m),

     ("term",      _          ) => subcommands::term(),
     ("doc",       _          ) => subcommands::doc(),

     ("remote",    _          ) => subcommands::git_remote(),
     ("pull",      _          ) => subcommands::git_pull(),
     ("status",    _          ) => subcommands::git_status(),
     ("add",       Some(sub_m)) => subcommands::git_add(&sub_m),
     ("commit",    _          ) => subcommands::git_commit(),
     ("push",      _          ) => subcommands::git_push(),
     _                       => ()
    }


}

fn main(){
    setup_app();
}

