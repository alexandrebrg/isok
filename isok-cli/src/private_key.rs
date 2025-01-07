use core::str;
use std::{fs, io};

use biscuit_auth::PrivateKey;
use zeroize::Zeroizing;

static BUFFER_LEN: usize = 128;

/// Parse a private key from hex-encoded string.
fn from_hex(s: &str) -> io::Result<PrivateKey> {
    match PrivateKey::from_bytes_hex(s.trim()) {
        Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
        Ok(private_key) => Ok(private_key),
    }
}

/// Parse a private key from raw or hex-encoded bytes.
///
/// `input` is assumed to be a raw private key if it's `32` bytes long,
/// otherwise it is assumed to be hex-encoded.
fn from_raw_or_hex(input: &[u8]) -> io::Result<PrivateKey> {
    match PrivateKey::from_bytes(input) {
        Err(_error) => match str::from_utf8(input) {
            Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
            Ok(s) => self::from_hex(s),
        },
        Ok(private_key) => Ok(private_key),
    }
}

/// Parse a private key from raw or hex-encoded contents read from `reader`.
fn from_reader(reader: &mut impl io::Read, buf: &mut [u8; BUFFER_LEN]) -> io::Result<PrivateKey> {
    match reader.read(buf) {
        Err(error) => Err(error),
        Ok(n) => self::from_raw_or_hex(&buf[..n]),
    }
}

/// Parse a private key
///
/// If the input string is `-` we read raw or hex-encoded key from [`io::stdin`],
/// otherwise it might be a path to a file containing the private key or a private key itself.
/// Thus, we try to open the file and, if it exists, read a raw or hex-encoded key from it,
/// otherwise we assume the string is an hex-encoded private key.
pub(super) fn parse(s: &str) -> io::Result<PrivateKey> {
    let buf = &mut Zeroizing::new([0; BUFFER_LEN]);
    match s {
        "-" => self::from_reader(&mut io::stdin().lock(), buf),
        path => match fs::File::open(path) {
            Err(_) => self::from_hex(s),
            Ok(mut file) => self::from_reader(&mut file, buf),
        },
    }
}
