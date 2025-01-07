use std::{io, time::SystemTime};

use biscuit_auth::{builder_ext::BuilderExt, Biscuit, KeyPair, PrivateKey};
use chrono::DateTime;

use super::private_key;

fn parse_ttl(s: &str) -> Result<SystemTime, chrono::ParseError> {
    // NOTE: unlike biscuit we don't currently support specifying a duration here
    Ok(DateTime::parse_from_rfc3339(s)?.to_utc().into())
}

fn parse_service(s: &str) -> io::Result<Box<str>> {
    let s = s.trim();

    if s.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "service name cannot be empty",
        ));
    }

    Ok(s.to_owned().into_boxed_str())
}

/// Generate a biscuit token for a new agent.
#[derive(Debug, Clone, clap::Parser)]
pub struct NewAgentCommand {
    /// Private key of the broker, used to sign the token.
    #[clap(long, short, value_name = "KEY",  value_parser = private_key::parse,)]
    pub private_key: PrivateKey,

    /// Name of the service covered by this agent.
    #[clap(long, value_parser = parse_service)]
    pub service: Box<str>,

    /// Output the raw bytes of the token instead of the base64-encoded string.
    #[clap(long)]
    pub raw: bool,

    // Add an expiration check to the generated authority block (RFC 3339 datetime).
    #[clap(long, value_parser = parse_ttl)]
    pub ttl: Option<SystemTime>,
}

impl NewAgentCommand {
    pub fn run(self, w: &mut impl io::Write) -> io::Result<()> {
        let root_key = KeyPair::from(&self.private_key);

        let mut builder = Biscuit::builder();

        if let Err(error) = builder.add_code(format!("service(\"{}\");", self.service)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("failed to parse datalog statement: {error}"),
            ));
        }

        if let Some(ttl) = self.ttl {
            builder.check_expiration_date(ttl);
        }

        match builder.build(&root_key) {
            Err(error) => Err(io::Error::other(format!(
                "failed to build biscuit: {error}"
            ))),
            Ok(biscuit) => {
                let buf = if self.raw {
                    match biscuit.to_vec() {
                        Err(error) => {
                            return Err(io::Error::other(format!(
                                "failed to encode biscuit: {error}"
                            )))
                        }
                        Ok(encoded) => encoded,
                    }
                } else {
                    match biscuit.to_base64() {
                        Err(error) => {
                            return Err(io::Error::other(format!(
                                "failed to encode biscuit: {error}"
                            )))
                        }
                        Ok(encoded) => encoded.into_bytes(),
                    }
                };
                w.write_all(&buf)
            }
        }
    }
}
