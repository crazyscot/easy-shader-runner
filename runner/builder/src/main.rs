use spirv_builder::SpirvBuilder;
use std::env;
use std::path::Path;

fn build_shader(path_to_crate: impl AsRef<Path>) {
    let builder_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path_to_crate = builder_dir.join(path_to_crate);
    let builder = SpirvBuilder::new(path_to_crate, "spirv-unknown-vulkan1.1");
    #[cfg(feature = "emulate_constants")]
    let builder = builder.shader_crate_features(["emulate_constants".into()]);
    builder.build().unwrap();
}

fn main() {
    let path_to_crate = env::var("SHADER_CRATE").unwrap();
    build_shader(&path_to_crate);
}
