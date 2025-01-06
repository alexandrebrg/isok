use core::str;
use std::{fs, io, path::Path};

use biscuit_auth::PrivateKey;
use zeroize::Zeroizing;

static BUFFER_LEN: usize = 128;

fn decode(
    reader: &mut (impl ?Sized + io::Read),
    raw: bool,
    buf: &mut [u8; BUFFER_LEN],
) -> io::Result<PrivateKey> {
    let n = reader.read(buf)?;
    if raw {
        match PrivateKey::from_bytes(&buf[..n]) {
            Err(error) => Err(io::Error::new(io::ErrorKind::InvalidData, error)),
            Ok(private_key) => Ok(private_key),
        }
    } else {
        match str::from_utf8(&buf[..n]) {
            Err(error) => Err(io::Error::new(io::ErrorKind::InvalidData, error)),
            Ok(s) => match PrivateKey::from_bytes_hex(s.trim()) {
                Err(error) => Err(io::Error::new(io::ErrorKind::InvalidData, error)),
                Ok(private_key) => Ok(private_key),
            },
        }
    }
}

pub(super) fn from_file(path: &Path, raw: bool) -> io::Result<PrivateKey> {
    let reader = if path.as_os_str() == "-" {
        &mut io::stdin().lock() as &mut dyn io::Read
    } else {
        &mut fs::File::open(path)? as &mut dyn io::Read
    };
    decode(reader, raw, &mut Zeroizing::new([0; BUFFER_LEN]))
}
