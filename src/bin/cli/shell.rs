use rustyline::{
    completion, error::ReadlineError, history::DefaultHistory, Context, Editor, Helper, Highlighter, Hinter,
    Result as LineResult, Validator,
};

use asciii::CONFIG;

use super::app::with_cli;
use anyhow::Error;
use clap::Command;
use std::collections::BTreeSet;

static ESCAPE_CHAR: Option<char> = Some('\\');

#[derive(Hinter, Helper, Validator, Highlighter)]
struct ClapCompleter {
    commands: Vec<String>,
}

impl ClapCompleter {
    pub fn from_app(app: &Command) -> Self {
        ClapCompleter {
            commands: app
                .get_subcommands()
                .map(|s| s.get_name().to_owned())
                .collect::<Vec<_>>(),
        }
    }

    pub fn naive_complete(
        &self,
        start: &str,
        _esc_char: Option<char>,
        _break_chars: &BTreeSet<char>,
    ) -> LineResult<Vec<String>> {
        Ok(self.commands.iter().filter(|s| s.starts_with(start)).cloned().collect())
    }
}

impl completion::Completer for ClapCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> LineResult<(usize, Vec<String>)> {
        let break_chars = BTreeSet::new();
        let (start, path) = completion::extract_word(line, pos, None, |_| false);
        //let path = completion::unescape(path, ESCAPE_CHAR);
        let matches = self.naive_complete(path, ESCAPE_CHAR, &break_chars)?;
        Ok((start, matches))
    }
}

pub fn launch_shell() -> Result<(), Error> {
    with_cli(|mut app| {
        //let file_compl = FilenameCompleter::new();
        let clap_compl = ClapCompleter::from_app(&app);
        let mut rl: Editor<ClapCompleter, DefaultHistory> = Editor::new().unwrap();

        //rl.set_completer(Some(file_compl));
        rl.set_helper(Some(clap_compl));
        //if rl.load_history("history.txt").is_err() { log::debug!("No previous shell history."); }

        let exit_cmds = ["exit", "quit", "stop", "kill", "halt"];

        let username = CONFIG
            .get_str_or("user.name")
            .and_then(|full_name| full_name.split_whitespace().next())
            .map(str::to_lowercase);
        let shell_key = "asciii > ";
        let ps1 = if let Some(username) = username {
            format!("{}@{}", username, shell_key)
        } else {
            String::from(shell_key)
        };

        loop {
            let readline = rl.readline(&ps1);
            match readline {
                Ok(line) => {
                    if let Err(error) = rl.add_history_entry(&line) {
                        log::warn!("failed to add history entry to shell {error}");
                    }

                    if exit_cmds.contains(&line.trim()) {
                        break;
                    }

                    if line.trim().is_empty() {
                        continue;
                    }

                    // this operators are not allowed
                    if line.contains('>') || line.contains('>') || line.contains('|') {
                        log::error!("What do you think this is? A shell?");
                    }

                    let mut argv: Vec<_> = line.trim().split(' ').collect();

                    // you have to insert the binary name since clap expects it
                    argv.insert(0, "prog");
                    log::debug!("shell: {} -> {:?}", line, argv);
                    match app.get_matches_from_safe_borrow(argv) {
                        Ok(matches) => super::match_matches(&matches),
                        Err(e) => println!("{}", e),
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    //println!("CTRL-C");
                    break;
                },
                Err(ReadlineError::Eof) => {
                    //println!("CTRL-D");
                    break;
                },
                Err(err) => {
                    log::error!("{:?}", err);
                    break;
                },
            }
        }
        //rl.save_history("history.txt").unwrap();
    });
    Ok(())
}
