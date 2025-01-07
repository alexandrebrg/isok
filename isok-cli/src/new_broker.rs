use std::io;

use biscuit_auth::{KeyPair, PrivateKey};

use super::private_key;

/// Generate a keypair for a new broker.
#[derive(Debug, Clone, clap::Parser)]
pub struct NewBrokerCommand {
    /// Generate the keypair from a private key.
    ///
    /// Value can be an hex-encoded private key or a path to a file (use `-` to read it from stdin).
    #[clap(long, value_name = "KEY", value_parser = private_key::parse)]
    pub private_key: Option<PrivateKey>,

    /// Only output the public part of the key pair.
    #[clap(long, conflicts_with = "only_private_key")]
    pub only_public_key: bool,

    /// Output the public key raw bytes directly, with no hex encoding.
    #[clap(long, requires = "only_public_key")]
    pub raw_public_key_output: bool,

    /// Only output the public part of the key pair.
    #[clap(long, conflicts_with = "only_public_key")]
    pub only_private_key: bool,

    /// Output the private key raw bytes directly, with no hex encoding.
    #[clap(long, requires = "only_private_key")]
    pub raw_private_key_output: bool,
}

impl NewBrokerCommand {
    pub fn run(self, w: &mut impl io::Write) -> io::Result<()> {
        let key_pair = match self.private_key.as_ref() {
            Some(private_key) => KeyPair::from(private_key),
            None => {
                writeln!(w, "Generating a new random key pair")?;
                KeyPair::new()
            }
        };

        if self.only_public_key {
            let public_key = key_pair.public();
            if self.raw_public_key_output {
                w.write_all(&public_key.to_bytes())
            } else {
                w.write_all(public_key.to_bytes_hex().as_bytes())
            }
        } else if self.only_private_key {
            let private_key = key_pair.private();
            if self.raw_private_key_output {
                w.write_all(&private_key.to_bytes())
            } else {
                w.write_all(private_key.to_bytes_hex().as_bytes())
            }
        } else if self.private_key.is_some() {
            w.write_all(key_pair.public().to_bytes_hex().as_bytes())
        } else {
            writeln!(w, "Private key: `{}`", key_pair.private().to_bytes_hex())?;
            writeln!(w, "Public key:  `{}`", key_pair.public().to_bytes_hex())
        }
    }
}
