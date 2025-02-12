use spirv_builder::{CompileResult, MetadataPrintout, ModuleResult, SpirvBuilder};
use std::path::PathBuf;
#[cfg(feature = "watch")]
use {crate::user_event::UserEvent, egui_winit::winit::event_loop::EventLoopProxy};

pub fn compile_shader(#[cfg(feature = "watch")] event_proxy: EventLoopProxy<UserEvent>) -> PathBuf {
    // Hack: spirv_builder builds into a custom directory if running under cargo, to not
    // deadlock, and the default target directory if not. However, packages like `proc-macro2`
    // have different configurations when being built here vs. when building
    // rustc_codegen_spirv normally, so we *want* to build into a separate target directory, to
    // not have to rebuild half the crate graph every time we run. So, pretend we're running
    // under cargo by setting these environment variables.
    std::env::set_var(
        "OUT_DIR",
        option_env!("SHADERS_TARGET_DIR").unwrap_or(env!("OUT_DIR")),
    );
    std::env::set_var("PROFILE", env!("PROFILE"));
    let manifest_dir = option_env!("SHADERS_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR"));
    let crate_path = [manifest_dir, "..", "shader", "shader"]
        .iter()
        .copied()
        .collect::<PathBuf>();

    let builder = SpirvBuilder::new(crate_path, "spirv-unknown-vulkan1.1")
        .print_metadata(MetadataPrintout::None)
        .shader_panic_strategy(spirv_builder::ShaderPanicStrategy::SilentExit);
    fn handle_compile_result(compile_result: CompileResult) -> PathBuf {
        match compile_result.module {
            ModuleResult::SingleModule(result) => result,
            ModuleResult::MultiModule(_) => {
                panic!("called `ModuleResult::unwrap_single()` on a `MultiModule` result")
            }
        }
    }
    #[cfg(feature = "watch")]
    let initial_result = builder
        .watch(move |compile_result| {
            std::assert!(event_proxy
                .send_event(UserEvent::NewModule(handle_compile_result(compile_result)))
                .is_ok())
        })
        .expect("Configuration is correct for watching");
    #[cfg(not(feature = "watch"))]
    let initial_result = builder.build().unwrap();
    handle_compile_result(initial_result)
}
