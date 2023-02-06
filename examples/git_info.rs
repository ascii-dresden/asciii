use std::process::{Command, Output};

fn execute_git(command: &str, args: &[&str]) -> Output {
    let workdir = ".";
    let gitdir = "./.git";

    Command::new("git")
        .arg("--no-pager")
        .args(["--work-tree", workdir])
        .args(["--git-dir", gitdir])
        .arg(command)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("git_failed {error}"))
}

fn main() {
    let git_log =
        String::from_utf8(execute_git("log", &["--oneline", r##"--format=%h"##]).stdout).unwrap();
    let count = git_log.lines().count().to_string();
    let last_commit = git_log.lines().next().unwrap().to_string();
    let description = format!("commit {} ({})", count.trim(), last_commit.trim());
    println!("description string= {description:?}");
}
