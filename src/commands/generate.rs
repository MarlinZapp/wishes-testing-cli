use clap_complete::{generate, Generator, Shell};

use crate::runnable::Runnable;

pub struct GenerateCommand {
    shell: Shell,
    command: clap::Command,
}

impl GenerateCommand {
    pub fn new(shell: Shell, command: clap::Command) -> Self {
        Self { shell, command }
    }
}

impl Runnable for GenerateCommand {
    async fn run(&mut self) -> Result<(), reqwest::Error> {
        eprintln!("Generating completion file for {}...", self.shell);
        print_completions(self.shell, &mut self.command);
        Ok(())
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
