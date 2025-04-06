use spirv_builder::SpirvBuilder;
use std::env;
use std::error::Error;
use std::path::Path;

fn build_shader(path_to_crate: &str) -> Result<(), Box<dyn Error>> {
    let builder_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path_to_crate = builder_dir.join(path_to_crate);
    let builder = SpirvBuilder::new(path_to_crate, "spirv-unknown-vulkan1.1");
    #[cfg(feature = "emulate_constants")]
    let builder = builder.shader_crate_features(["emulate_constants".into()]);
    builder.build()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let path_to_crate = std::env::var("SHADER_CRATE")?;
    build_shader(&path_to_crate)?;
    Ok(())
}
