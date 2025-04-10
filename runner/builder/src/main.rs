use spirv_builder::SpirvBuilder;
use std::path::Path;

fn build_shader(path_to_crate: impl AsRef<Path>) {
    let builder = SpirvBuilder::new(path_to_crate, "spirv-unknown-vulkan1.1");
    #[cfg(feature = "emulate_constants")]
    let builder = builder.shader_crate_features(["emulate_constants".into()]);
    builder.build().unwrap();
}

fn main() {
    let path_to_crate = std::env::args()
        .nth(1)
        .expect("expected path_to_crate command line arg");
    build_shader(&path_to_crate);
}
