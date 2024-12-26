use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .compile_protos(&["src/event.proto"], &["src/"])?;
    Ok(())
}