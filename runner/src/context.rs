use egui_winit::winit::{dpi::PhysicalSize, window::Window};
use std::sync::Arc;

pub struct GraphicsContext {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl GraphicsContext {
    pub async fn new(window: Arc<Window>, initial_size: PhysicalSize<u32>) -> GraphicsContext {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());
        let initial_surface = instance
            .create_surface(window)
            .expect("Failed to create surface from window");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&initial_surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (features, limits) = if cfg!(target_arch = "wasm32") {
            Default::default()
        } else {
            (
                wgpu::Features::PUSH_CONSTANTS,
                wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
            )
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: limits,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        fn auto_configure_surface<'a>(
            adapter: &wgpu::Adapter,
            device: &wgpu::Device,
            surface: wgpu::Surface<'a>,
            size: PhysicalSize<u32>,
        ) -> (wgpu::Surface<'a>, wgpu::SurfaceConfiguration) {
            let mut surface_config = surface
                .get_default_config(adapter, size.width, size.height)
                .unwrap_or_else(|| {
                    panic!(
                        "Missing formats/present modes in surface capabilities: {:#?}",
                        surface.get_capabilities(adapter)
                    )
                });
            surface_config.present_mode = wgpu::PresentMode::AutoVsync;
            surface.configure(device, &surface_config);
            (surface, surface_config)
        }

        let (surface, config) =
            auto_configure_surface(&adapter, &device, initial_surface, initial_size);

        GraphicsContext {
            surface,
            device,
            queue,
            config,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_vsync(&mut self, enable: bool) {
        self.config.present_mode = if enable {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        self.surface.configure(&self.device, &self.config);
    }
}
