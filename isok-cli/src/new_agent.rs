use std::{collections::HashMap, io, path::Path, time::SystemTime};

use biscuit_auth::{
    builder::Term, builder_ext::BuilderExt, Biscuit, KeyPair, PrivateKey, PublicKey,
};
use chrono::DateTime;
use zeroize::Zeroizing;

use super::{authority_file, private_key};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ValueType {
    Pubkey,

    #[default]
    String,
    Integer,
    Date,
    Bytes,
    Bool,
}

impl ValueType {
    pub fn parse_param(&self, name: &str, value: &str) -> io::Result<Param> {
        match self {
            Self::Pubkey => {
                match value.strip_prefix("ed25519/") {
                    None => Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "unsupported public key type: only hex-encoded ed25519 public keys are supported (they must start with `ed25519/`)",
                    )),
                    Some(hex_key) => {
                        match PublicKey::from_bytes_hex(hex_key) {
                            Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
                            Ok(pubkey) => Ok(Param::PublicKey(name.to_owned(), pubkey)),
                        }
                    }
                }
            }
            Self::String => Ok(Param::Term(name.to_owned(), Term::Str(value.to_owned()))),
            Self::Integer => match value.parse() {
                Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
                Ok(integer) => Ok(Param::Term(name.to_owned(), Term::Integer(integer))),
            }
            Self::Date => match time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339) {
                Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
                Ok(date) => match date.unix_timestamp().try_into() {
                    Err(_) => Err(io::Error::new(io::ErrorKind::InvalidInput, "date must be after Unix Epoch")),
                    Ok(timestamp) =>  Ok(Param::Term(name.to_owned(), Term::Date(timestamp))),
                },
            }
            Self::Bytes => {
                match value.strip_prefix("hex:") {
                    None => Err(io::Error::new(
                        io::ErrorKind::Other,
                        "unsupported literal byte array: byte arrays must be hex-encoded and start with `hex:`",
                    )),
                    Some(data) => {
                        match hex::decode(data) {
                            Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
                            Ok(bytes) => Ok(Param::Term(name.to_owned(), Term::Bytes(bytes)))
                        }
                    }
                }
            }
            Self::Bool => match value.to_ascii_lowercase().as_str() {
                "true" => Ok(Param::Term(name.to_owned(), Term::Bool(true))),
                "false" => Ok(Param::Term(name.to_owned(), Term::Bool(false))),
                _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "bool must be either `true` or `false`")),
            }
        }
    }
}

impl core::str::FromStr for ValueType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pubkey" => Ok(Self::Pubkey),
            "string" => Ok(Self::String),
            "integer" => Ok(Self::Integer),
            "date" => Ok(Self::Date),
            "bytes" => Ok(Self::Bytes),
            "bool" => Ok(Self::Bool),
            _ => Err(format!("unsupported value type: {s}")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Param {
    Term(String, Term),
    PublicKey(String, PublicKey),
}

impl core::str::FromStr for Param {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('=') {
            None => Err(io::Error::new(io::ErrorKind::InvalidInput,
                "param must be `key=value` or `key:type=value` where type is one of pubkey, string, integer, date, bytes or bool",
            )),
            Some((name, value)) => match name.rsplit_once(':') {
                None => ValueType::default().parse_param(name, value),
                Some((name, value_type)) => match value_type.parse::<ValueType>() {
                    Err(error) =>  Err(io::Error::new(io::ErrorKind::InvalidInput,error)),
                    Ok(value_type) => value_type.parse_param(name, value)
                }
            },
        }
    }
}

fn parse_ttl(s: &str) -> Result<SystemTime, chrono::ParseError> {
    // NOTE: unlike biscuit we don't currently support specifying a duration here
    Ok(DateTime::parse_from_rfc3339(s)?.to_utc().into())
}

/// Generate a biscuit for a new agent from the private key of a broker.
#[derive(Debug, Clone, clap::Parser)]
pub struct NewAgentCommand {
    /// Read the authority block from the given file (or use `-` to read from stdin).
    /// If omitted, an interactive `$EDITOR` will be opened.
    #[clap(long, value_name = "PATH", value_hint = clap::ValueHint::FilePath)]
    pub authority_file: Option<Box<Path>>,

    /// Provide a root key id, as a hint for public key selection.
    #[clap(long)]
    pub root_key_id: Option<u32>,

    /// Provide a value for a datalog parameter.
    /// `type` is optional and defaults to `string`.
    /// Possible types are pubkey, string, integer, date, bytes or bool.
    /// Bytes values must be hex-encoded and start with `hex:`
    /// Public keys must be hex-encoded and start with `ed25519/`
    #[clap(long, value_name = "KEY[:TYPE]=VALUE")]
    pub param: Vec<Param>,

    /// Output the biscuit raw bytes directly, with no base64 encoding.
    #[clap(long)]
    pub raw: bool,

    /// The private of the broker, used to sign the token.
    #[clap(long, value_name = "KEY", required_unless_present = "private_key_file", value_parser = PrivateKey::from_bytes_hex)]
    pub private_key: Option<PrivateKey>,

    /// The private key used to sign the token.
    #[clap(
        long,
        value_name = "PATH",
        value_hint = clap::ValueHint::FilePath,
        required_unless_present = "private_key",
        conflicts_with = "private_key"
    )]
    pub private_key_file: Option<Box<Path>>,

    /// Read the private key raw bytes directly (only available when reading the private key from a file)
    #[clap(long, conflicts_with = "private_key", requires = "private_key_file")]
    pub raw_private_key: bool,

    /// Optional context string attached to the authority block.
    #[clap(long)]
    pub context: Option<String>,

    // Add an expiration check to the generated authority block (RFC 3339 datetime).
    #[clap(long, alias = "add-ttl", value_parser = parse_ttl)]
    pub ttl: Option<SystemTime>,
}

impl NewAgentCommand {
    pub fn run(self, w: &mut impl io::Write) -> io::Result<()> {
        let private_key = match self.private_key {
            Some(private_key) => private_key,
            None => match self.private_key_file {
                Some(path) => match private_key::from_file(&path, self.raw_private_key) {
                    Err(error) => return Err(error),
                    Ok(private_key) => private_key,
                },
                None => {
                    unreachable!("clap requires one of `private_key` or `private_key_file`")
                }
            },
        };

        let root_key = KeyPair::from(&private_key);

        let mut builder = Biscuit::builder();

        {
            let buf = &mut Zeroizing::new([0; 1024]);
            let source = authority_file::source(self.authority_file.as_deref(), buf)?;

            let (params, scope_params) = {
                let mut params = HashMap::new();
                let mut scope_params = HashMap::new();
                for param in self.param {
                    match param {
                        Param::Term(k, v) => {
                            let _ = params.insert(k, v);
                        }
                        Param::PublicKey(k, v) => {
                            let _ = scope_params.insert(k, v);
                        }
                    }
                }
                (params, scope_params)
            };

            if let Err(error) = builder.add_code_with_params(source, params, scope_params) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("failed to parse datalog statement: {error}"),
                ));
            }
        }

        if let Some(context) = self.context {
            builder.set_context(context);
        }

        if let Some(ttl) = self.ttl {
            builder.check_expiration_date(ttl);
        }

        if let Some(root_key_id) = self.root_key_id {
            builder.set_root_key_id(root_key_id);
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
