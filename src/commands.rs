use crate::runnable::Runnable;

pub(crate) mod case;
pub(crate) mod generate;

pub enum TestingCommand<'e> {
    Case(case::TestCase<'e>),
    Generate(generate::GenerateCommand),
}

impl Runnable for TestingCommand<'_> {
    async fn run(&mut self) -> Result<(), reqwest::Error> {
        match self {
            TestingCommand::Case(test_case) => test_case.run().await?,
            TestingCommand::Generate(generate_command) => generate_command.run().await?,
        }
        Ok(())
    }
}
