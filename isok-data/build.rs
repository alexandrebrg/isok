use std::io::Result;

fn main() -> Result<()> {
    #[cfg(feature = "prost")]
    prost_build::compile_protos(&["src/event.proto"], &["src/"])?;

    #[cfg(feature = "tonic")]
    tonic_build::compile_protos("src/event.proto")?;
    Ok(())
}