use core::str;
use std::{
    env, fs,
    io::{self, Read},
    path::Path,
    process::Command,
};

pub static BUFFER_LEN: usize = 1024;

fn editor_command() -> io::Result<Command> {
    match env::var_os("EDITOR") {
        None => Ok(Command::new("vim")),
        Some(os_string) => match os_string.to_str() {
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "value of `$EDITOR` environment variable is not valid UTF-8",
            )),
            Some(s) => match shell_words::split(s) {
                Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
                Ok(editor_parts) => match editor_parts.split_first() {
                    None => Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "missing program in value of `$EDITOR` environment variable",
                    )),
                    Some((program, args)) => {
                        let mut command = Command::new(program);
                        command.args(args);
                        Ok(command)
                    }
                },
            },
        },
    }
}

fn editor() -> io::Result<fs::File> {
    let file = tempfile::Builder::new()
        .suffix(".isok-biscuit-datalog")
        .tempfile()?;

    if editor_command()?
        .arg(file.path())
        .spawn()?
        .wait()?
        .success()
    {
        Ok(file.into_file())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "editor process exited with a non-zero status",
        ))
    }
}

pub(super) fn source<'a>(
    path: Option<&Path>,
    buf: &'a mut [u8; BUFFER_LEN],
) -> io::Result<&'a str> {
    let reader = match path {
        None => &mut editor()? as &mut dyn Read,
        Some(path) if path.as_os_str() == "-" => &mut io::stdin(),
        Some(path) => &mut fs::File::open(path)?,
    };
    let n = reader.read(&mut *buf)?;
    match str::from_utf8(&buf[..n]) {
        Err(error) => Err(io::Error::new(io::ErrorKind::InvalidData, error)),
        Ok(s) => Ok(s),
    }
}
