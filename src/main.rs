use std::path::Path;

use clap::ValueHint;
use clap_complete::Shell;
use commands::{
    case::{CaseNum, TestCase},
    generate::GenerateCommand,
    TestingCommand,
};
use runnable::Runnable;

// Created with help of example from https://github.com/clap-rs/clap/blob/master/clap_complete/examples/exhaustive.rs

mod commands;
pub mod runnable;

fn main() {
    clap_complete::CompleteEnv::with_factory(cli)
        // Avoid tests snapshotting a path into `target/`
        .completer("testing")
        .complete();

    let matches = cli().get_matches();
    let mut command = None;
    let shell = matches
        .get_one::<Shell>("shell")
        .expect("Please provide a shell!")
        .clone();
    if let Some(case_matches) = matches.subcommand_matches("case") {
        if let Some(executable) = case_matches.get_one::<String>("executable") {
            let executable = Path::new(executable);
            if let Some(_) = case_matches.subcommand_matches("one") {
                command = Some(TestingCommand::Case(TestCase::new(
                    CaseNum::One,
                    executable,
                    &shell,
                )));
            } else if let Some(_) = case_matches.subcommand_matches("two") {
                command = Some(TestingCommand::Case(TestCase::new(
                    CaseNum::Two,
                    executable,
                    &shell,
                )));
            }
        }
    } else if let Some(_) = matches.subcommand_matches("generate") {
        command = Some(TestingCommand::Generate(GenerateCommand::new(shell, cli())));
    }

    if let Some(mut command) = command {
        command.run();
    } else {
        eprintln!("Cannot recognize subcommand or no subcommands present.");
        return;
    }
}

#[allow(clippy::let_and_return)]
fn cli() -> clap::Command {
    clap::Command::new("testing")
        .args([clap::Arg::new("shell")
            .short('s')
            .long("shell")
            .value_parser(clap::value_parser!(Shell))
            .required(true)
            .help("Please select your active shell")])
        .subcommands([
            clap::Command::new("generate").about("Generate shell completions"),
            clap::Command::new("case")
                .about("Run a test case")
                .args([clap::Arg::new("executable")
                    .required(true)
                    .long("surrealdb-executable")
                    .short('e')
                    .help("The path to the executable starting the surrealdb server")
                    .value_hint(ValueHint::FilePath)])
                .subcommands([clap::Command::new("one"), clap::Command::new("two")]),
        ])
}
