use core::str;

use std::io;

mod private_key;

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

fn main() -> color_eyre::Result<()> {
    Ok(Cli::from_env().run()?)
}

#[cfg(test)]
mod tests {

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;

        use super::Cli;

        Cli::command().debug_assert();
    }
}
