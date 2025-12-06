use std::io::{Result};

fn main() -> Result<()> {
    #[cfg(feature = "compile-protobuf-src")]
    std::env::set_var("PROTOC", protobuf_src::protoc());

    let mut config = prost_build::Config::new();
    #[cfg(feature = "documented")]
    config.type_attribute(".", "#[derive(documented::DocumentedFields)]");
    #[cfg(feature = "serde")]
    config.type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]");

    config.compile_protos(&["bokeh_creator.proto"], &["proto/"])?;

    Ok(())
}
