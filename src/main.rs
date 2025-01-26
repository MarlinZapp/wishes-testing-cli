use std::path::Path;

use clap::ValueHint;
use clap_complete::Shell;
use commands::{
    case::{CaseFourArgs, CaseNum, CaseOneArgs, CaseThreeArgs, CaseTwoArgs, TestCase},
    generate::GenerateCommand,
    TestingCommand,
};
use runnable::Runnable;

// Created with help of example from https://github.com/clap-rs/clap/blob/master/clap_complete/examples/exhaustive.rs

mod commands;
pub mod runnable;

#[tokio::main]
async fn main() {
    clap_complete::CompleteEnv::with_factory(cli)
        // Avoid tests snapshotting a path into `target/`
        .completer("testing")
        .complete();

    let matches = cli().get_matches();
    let mut command = None;
    let shell;
    if let Some(from_arg) = matches.get_one::<Shell>("shell") {
        shell = from_arg.clone();
    } else if let Some(from_env) = clap_complete::Shell::from_env() {
        shell = from_env;
    } else {
        panic!("No SHELL provided in environment arguments, please provide the shell argument!");
    }
    if let Some(case_matches) = matches.subcommand_matches("case") {
        if let Some(executable) = case_matches.get_one::<String>("executable") {
            let executable = Path::new(executable);
            if let Some(case_one_matches) = case_matches.subcommand_matches("one") {
                let n_res = case_one_matches.get_one::<u32>("users");
                let n;
                if let Some(res) = n_res {
                    n = res.clone();
                } else {
                    n = 1000;
                }
                command = Some(TestingCommand::Case(TestCase::new(
                    CaseNum::One(CaseOneArgs { n }),
                    executable,
                    &shell,
                )));
            } else if let Some(case_two_matches) = case_matches.subcommand_matches("two") {
                let n_wishes = case_two_matches.get_one::<u32>("wishes");
                let n;
                if let Some(res) = n_wishes {
                    n = res.clone();
                } else {
                    n = 1000;
                }
                command = Some(TestingCommand::Case(TestCase::new(
                    CaseNum::Two(CaseTwoArgs { wishes: n }),
                    executable,
                    &shell,
                )));
            } else if let Some(case_two_matches) = case_matches.subcommand_matches("three") {
                let n_wishes = case_two_matches.get_one::<u32>("wishes");
                let n;
                if let Some(res) = n_wishes {
                    n = res.clone();
                } else {
                    n = 1000;
                }
                command = Some(TestingCommand::Case(TestCase::new(
                    CaseNum::Three(CaseThreeArgs { wishes: n }),
                    executable,
                    &shell,
                )));
            } else if let Some(case_two_matches) = case_matches.subcommand_matches("four") {
                let n_times = case_two_matches.get_one::<u32>("times");
                let n;
                if let Some(res) = n_times {
                    n = res.clone();
                } else {
                    n = 1000;
                }
                command = Some(TestingCommand::Case(TestCase::new(
                    CaseNum::Four(CaseFourArgs { times: n }),
                    executable,
                    &shell,
                )));
            }
        }
    } else if let Some(_) = matches.subcommand_matches("generate") {
        command = Some(TestingCommand::Generate(GenerateCommand::new(shell, cli())));
    }

    if let Some(mut command) = command {
        command
            .run()
            .await
            .expect("{command} has not been executed successfully!");
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
            .help("Here you can provide a shell if you don't want to use the environment variable SHELL.")])
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
                .subcommands([
                    clap::Command::new("one")
                        .about("Run test case one: Register n users.")
                        .arg(
                            clap::Arg::new("users")
                                .short('n')
                                .long("n-users")
                                .help("Number of users to register. Defaults to 1000.")
                                .value_parser(clap::value_parser!(u32))
                                .default_value("1000"),
                        ),
                    clap::Command::new("two")
                        .about("Run test case one: Register 10 users and create n/10 wishes.")
                        .arg(
                            clap::Arg::new("wishes")
                                .short('n')
                                .long("n-wishes")
                                .help("Number of wishes to create. Defaults to 1000.")
                                .value_parser(clap::value_parser!(u32))
                                .default_value("1000"),
                        ),
                    clap::Command::new("three")
                        .about("Run test case three: Get n wishes with one request.")
                        .arg(
                            clap::Arg::new("wishes")
                                .short('n')
                                .long("n-wishes")
                                .help("Number of wishes to get. Defaults to 1000.")
                                .value_parser(clap::value_parser!(u32))
                                .default_value("1000"),
                        ),
                    clap::Command::new("four")
                        .about("Run test case four: Get one wish n times.")
                        .arg(
                            clap::Arg::new("times")
                                .short('n')
                                .long("n-times")
                                .help("Number of times to get the wish. Defaults to 1000.")
                                .value_parser(clap::value_parser!(u32))
                                .default_value("1000"),
                        ),
                    ]),
        ])
}
