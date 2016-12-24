use rustyline::completion;
use rustyline::completion::FilenameCompleter;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::config::Config as LineConfig;
use rustyline::{CompletionType, Editor};
use rustyline::Result as LineResult;

use std::collections::BTreeSet;
use clap::App;
use super::app::build_cli;
use super::app::match_matches;

static ESCAPE_CHAR: Option<char> = Some('\\');

struct ClapCompleter{
    commands: Vec<String>
}

impl ClapCompleter {
    pub fn from_app(app:&App<'static,'static>) -> Self {
        ClapCompleter {
            commands:
                app.p.subcommands.iter()
                .map(|s|s.get_name().to_owned())
                .collect::<Vec<_>>()
        }
    }

    pub fn naiv_complete(&self, start: &str, _esc_char: Option<char>, break_chars: &BTreeSet<char>) -> LineResult<Vec<String>> {
        Ok(self.commands.iter()
                        .filter(|s|s.starts_with(start))
                        .cloned()
                        .collect())
    }
}

impl Completer for ClapCompleter {
    fn complete(&self, line: &str, pos: usize) -> LineResult<(usize, Vec<String>)> {
        let break_chars = BTreeSet::new();
        let (start, path) = completion::extract_word(line, pos, ESCAPE_CHAR, &break_chars);
        let path = completion::unescape(path, ESCAPE_CHAR);
        let matches = self.naiv_complete(&path, ESCAPE_CHAR, &break_chars)?;
        Ok((start, matches))
    }
}

pub fn launch_shell() {

    let mut app = build_cli();

    let config = LineConfig::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .build();

    //let file_compl = FilenameCompleter::new();
    let clap_compl = ClapCompleter::from_app(&app);
    let mut rl = Editor::new(config);

    //rl.set_completer(Some(file_compl));
    rl.set_completer(Some(clap_compl));
    //if rl.load_history("history.txt").is_err() { debug!("No previous shell history."); }

    let exit_cmds = ["exit", "stop", "kill", "halt"];

    loop {
        let readline = rl.readline("asciii > ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());

                if exit_cmds.contains(&line.trim()){
                    println!("Byebye");
                    break;
                }

                let mut argv: Vec<_> = line.trim().split(" ").collect();
                // you have to insert the binary name since clap expects it
                argv.insert(0, "prog");
                debug!("shell: {} -> {:?}", line, argv);
                match app.get_matches_from_safe_borrow(argv) {
                    Ok(matches) => super::match_matches(&matches),
                    Err(e) => println!("{}", e.message)
                }

            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    //rl.save_history("history.txt").unwrap();
}
