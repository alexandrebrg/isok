use std::{io, path::Path};

use biscuit_auth::{KeyPair, PrivateKey};
use clap::ValueHint;

use super::private_key;

/// Generate a keypair for a new broker.
#[derive(Debug, Clone, clap::Parser)]
pub struct NewBrokerCommand {
    /// Generate the keypair from a private key stored in the given file (or use `-` to read it from stdin).
    #[clap(long, value_name = "PATH", value_hint = ValueHint::FilePath, conflicts_with = "from_private_key")]
    pub from_private_key_file: Option<Box<Path>>,

    /// Read the private key raw bytes directly, with no hex decoding
    /// (only available when reading the private key from a file)
    #[clap(long, requires = "from_private_key_file")]
    pub from_raw_private_key: bool,

    /// Generate the keypair from the given private key.
    #[clap(long, value_name = "KEY", value_parser = PrivateKey::from_bytes_hex)]
    pub from_private_key: Option<PrivateKey>,

    /// Only output the public part of the key pair
    #[clap(long, conflicts_with = "only_private_key")]
    pub only_public_key: bool,

    /// Output the public key raw bytes directly, with no hex encoding
    #[clap(long, requires = "only_public_key")]
    pub raw_public_key_output: bool,

    /// Only output the public part of the key pair
    #[clap(long, conflicts_with = "only_public_key")]
    pub only_private_key: bool,

    /// Output the private key raw bytes directly, with no hex encoding
    #[clap(long, requires = "only_private_key")]
    pub raw_private_key_output: bool,
}

impl NewBrokerCommand {
    pub fn run(self, w: &mut impl io::Write) -> io::Result<()> {
        let from_private_key = match self.from_private_key {
            Some(from_private_key) => Some(from_private_key),
            None => match self.from_private_key_file {
                Some(path) => match private_key::from_file(&path, self.from_raw_private_key) {
                    Err(error) => return Err(error),
                    Ok(private_key) => Some(private_key),
                },
                None => None,
            },
        };

        let key_pair = match from_private_key.as_ref() {
            Some(private_key) => KeyPair::from(private_key),
            None => KeyPair::new(),
        };

        if self.only_public_key {
            let public_key = key_pair.public();
            if self.raw_public_key_output {
                w.write_all(&public_key.to_bytes())
            } else {
                w.write_all(&public_key.to_bytes_hex().as_bytes())
            }
        } else if self.only_private_key {
            let private_key = key_pair.private();
            if self.raw_private_key_output {
                w.write_all(&private_key.to_bytes())
            } else {
                w.write_all(&private_key.to_bytes_hex().as_bytes())
            }
        } else {
            writeln!(w, "Public key:  `{}`", key_pair.public().to_bytes_hex())?;
            writeln!(w, "Private key: `{}`", key_pair.private().to_bytes_hex())
        }
    }
}
