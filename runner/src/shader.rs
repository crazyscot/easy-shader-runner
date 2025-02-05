use std::borrow::Cow;

pub struct CompiledShaderModule {
    pub module: wgpu::ShaderModuleDescriptorSpirV<'static>,
}

pub fn maybe_watch(
    on_watch: Option<Box<dyn FnMut(CompiledShaderModule) + Send + 'static>>,
) -> CompiledShaderModule {
    {
        use spirv_builder::{CompileResult, MetadataPrintout, SpirvBuilder};
        use std::path::PathBuf;
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
        let initial_result = if let Some(mut f) = on_watch {
            builder
                .watch(move |compile_result| f(handle_compile_result(compile_result)))
                .expect("Configuration is correct for watching")
        } else {
            builder.build().unwrap()
        };
        fn handle_compile_result(compile_result: CompileResult) -> CompiledShaderModule {
            let path = compile_result.module.unwrap_single();
            let data = std::fs::read(path).unwrap();
            // FIXME(eddyb) this reallocates all the data pointlessly, there is
            // not a good reason to use `ShaderModuleDescriptorSpirV` specifically.
            let spirv = Cow::Owned(wgpu::util::make_spirv_raw(&data).into_owned());
            // let spirv = wgpu::util::make_spirv(&data);
            let module = wgpu::ShaderModuleDescriptorSpirV {
                label: None,
                source: spirv,
            };
            CompiledShaderModule { module }
        }
        handle_compile_result(initial_result)
    }
}
