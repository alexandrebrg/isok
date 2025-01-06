use core::str;

use std::io;

mod private_key;

mod authority_file;

mod new_agent;
use self::new_agent::NewAgentCommand;

mod new_broker;
use self::new_broker::NewBrokerCommand;

// CLI

/// Command line interface for creating `isok` brokers and agents.
#[derive(Debug, Clone, clap::Parser)]
#[clap(version, author)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn from_env() -> Self {
        match <Self as clap::Parser>::try_parse() {
            Ok(cli) => cli,
            Err(error) => error.exit(),
        }
    }

    pub fn run(self) -> io::Result<()> {
        self.command.run(&mut io::stdout().lock())
    }
}

// COMMAND

/// Subcommands for the [`Cli`].
#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    NewBroker(NewBrokerCommand),
    NewAgent(NewAgentCommand),
}

impl Command {
    pub fn run(self, w: &mut impl io::Write) -> io::Result<()> {
        match self {
            Self::NewBroker(command) => command.run(w),
            Self::NewAgent(command) => command.run(w),
        }
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}

fn main() -> io::Result<()> {
    Cli::from_env().run()
}
