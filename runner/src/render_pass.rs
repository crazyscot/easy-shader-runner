use crate::{
    bind_group_buffer::{BindGroupBufferType, BufferData, SSBO},
    context::GraphicsContext,
    controller::Controller,
    ui::{Ui, UiState},
};
use egui_winit::winit::{dpi::PhysicalSize, window::Window};
use std::path::Path;
use wgpu::{util::DeviceExt, BindGroupLayout, TextureView};

struct Pipelines {
    render: wgpu::RenderPipeline,
    compute: wgpu::ComputePipeline,
}

struct PipelineLayouts {
    render: wgpu::PipelineLayout,
    compute: wgpu::PipelineLayout,
}

pub struct RenderPass {
    pipelines: Pipelines,
    pipeline_layouts: PipelineLayouts,
    ui_renderer: egui_wgpu::Renderer,
    bind_groups: Vec<wgpu::BindGroup>,
}

impl RenderPass {
    pub fn new(ctx: &GraphicsContext, shader_path: &Path, buffer_data: &BufferData) -> Self {
        let bind_group_layouts = create_bind_group_layouts(ctx, buffer_data);
        let pipeline_layouts = create_pipeline_layouts(ctx, &bind_group_layouts);
        let pipelines = create_pipeline(
            &ctx.device,
            &pipeline_layouts,
            ctx.config.format,
            shader_path,
        );
        let bind_groups = maybe_create_bind_groups(ctx, buffer_data, &bind_group_layouts);

        let ui_renderer = egui_wgpu::Renderer::new(&ctx.device, ctx.config.format, None, 1, false);

        Self {
            pipelines,
            pipeline_layouts,
            ui_renderer,
            bind_groups,
        }
    }

    pub fn compute(
        &mut self,
        ctx: &GraphicsContext,
        inner_size: &PhysicalSize<u32>,
        controller: &Controller,
    ) {
        let m = inner_size.width / 2;
        let n = inner_size.height / 2;
        let w = glam::UVec3::new(16, 16, 1);
        let x = ((m as f32) / (w.x as f32)).ceil() as u32;
        let y = ((n as f32) / (w.y as f32)).ceil() as u32;
        self.call(ctx, (x, y, 1), controller);
    }

    pub fn call(
        &mut self,
        ctx: &GraphicsContext,
        workspace: (u32, u32, u32),
        controller: &Controller,
    ) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });

            cpass.set_pipeline(&self.pipelines.compute);
            cpass.set_push_constants(0, controller.compute_constants());
            for (i, bind_group) in self.bind_groups.iter().enumerate() {
                cpass.set_bind_group(i as u32, bind_group, &[]);
            }
            cpass.dispatch_workgroups(workspace.0, workspace.1, workspace.2);
        }
        ctx.queue.submit(Some(encoder.finish()));
    }

    pub fn render(
        &mut self,
        ctx: &GraphicsContext,
        window: &Window,
        ui: &mut Ui,
        ui_state: &mut UiState,
        controller: &mut Controller,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = match ctx.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(err) => {
                eprintln!("get_current_texture error: {err:?}");
                return match err {
                    wgpu::SurfaceError::Lost => {
                        ctx.surface.configure(&ctx.device, &ctx.config);
                        Ok(())
                    }
                    _ => Err(err),
                };
            }
        };
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_shader(ctx, &output_view, controller, window.inner_size());
        self.render_ui(ctx, &output_view, window, ui, ui_state, controller);

        output.present();

        Ok(())
    }

    fn render_shader(
        &mut self,
        ctx: &GraphicsContext,
        output_view: &TextureView,
        controller: &Controller,
        size: PhysicalSize<u32>,
    ) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shader Encoder"),
            });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shader Render Pass"),
                occlusion_query_set: None,
                timestamp_writes: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_viewport(
                0.0,
                shared::UI_MENU_HEIGHT as f32,
                (size.width - shared::UI_SIDEBAR_WIDTH) as f32,
                (size.height - shared::UI_MENU_HEIGHT) as f32,
                0.0,
                1.0,
            );

            rpass.set_pipeline(&self.pipelines.render);
            rpass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                0,
                controller.fragment_constants(),
            );
            for (i, bind_group) in self.bind_groups.iter().enumerate() {
                rpass.set_bind_group(i as u32, bind_group, &[]);
            }
            rpass.draw(0..3, 0..1);
        }

        ctx.queue.submit(Some(encoder.finish()));
    }

    fn render_ui(
        &mut self,
        ctx: &GraphicsContext,
        output_view: &TextureView,
        window: &Window,
        ui: &mut Ui,
        ui_state: &mut UiState,
        controller: &mut Controller,
    ) {
        let (clipped_primitives, textures_delta) = ui.prepare(window, ui_state, controller);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [ctx.config.width, ctx.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, delta) in &textures_delta.set {
            self.ui_renderer
                .update_texture(&ctx.device, &ctx.queue, *id, delta);
        }

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UI Encoder"),
            });

        self.ui_renderer.update_buffers(
            &ctx.device,
            &ctx.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                occlusion_query_set: None,
                timestamp_writes: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });

            for id in &textures_delta.free {
                self.ui_renderer.free_texture(id);
            }

            self.ui_renderer.render(
                &mut rpass.forget_lifetime(),
                &clipped_primitives,
                &screen_descriptor,
            );
        }

        ctx.queue.submit(Some(encoder.finish()));
    }

    pub fn new_module(&mut self, ctx: &GraphicsContext, shader_path: &Path) {
        self.pipelines = create_pipeline(
            &ctx.device,
            &self.pipeline_layouts,
            ctx.config.format,
            shader_path,
        );
    }
}

fn maybe_create_bind_groups(
    ctx: &GraphicsContext,
    buffer_data: &BufferData,
    bind_group_layouts: &Vec<BindGroupLayout>,
) -> Vec<wgpu::BindGroup> {
    buffer_data
        .bind_group_buffers
        .iter()
        .zip(bind_group_layouts)
        .enumerate()
        .map(|(i, (buffer, layout))| {
            let buffer = ctx.device.create_buffer_init(&match buffer {
                BindGroupBufferType::SSBO(ssbo) => wgpu::util::BufferInitDescriptor {
                    label: Some("Bind Group Buffer"),
                    contents: ssbo.data,
                    usage: wgpu::BufferUsages::STORAGE,
                },
                BindGroupBufferType::Uniform(uniform) => wgpu::util::BufferInitDescriptor {
                    label: Some("Bind Group Buffer"),
                    contents: uniform.data,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                },
            });
            ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some(&format!("bind_group {}", i)),
            })
        })
        .collect()
}

fn create_pipeline(
    device: &wgpu::Device,
    pipeline_layouts: &PipelineLayouts,
    surface_format: wgpu::TextureFormat,
    shader_path: &Path,
) -> Pipelines {
    let data = std::fs::read(shader_path).unwrap();
    let module = unsafe {
        &device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
            label: None,
            source: wgpu::util::make_spirv_raw(&data),
        })
    };
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layouts.render),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("main_vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some("main_fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview: None,
        cache: None,
    });
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layouts.compute),
        module,
        entry_point: Some("main_cs"),
        compilation_options: Default::default(),
        cache: None,
    });
    Pipelines {
        render: render_pipeline,
        compute: compute_pipeline,
    }
}

fn create_bind_group_layouts(
    ctx: &GraphicsContext,
    buffer_data: &BufferData,
) -> Vec<BindGroupLayout> {
    buffer_data
        .bind_group_buffers
        .iter()
        .enumerate()
        .map(|(i, buffer)| {
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: (match buffer {
                                BindGroupBufferType::Uniform(_) => wgpu::BufferBindingType::Uniform,
                                BindGroupBufferType::SSBO(SSBO { read_only, .. }) => {
                                    wgpu::BufferBindingType::Storage {
                                        read_only: *read_only,
                                    }
                                }
                            }),
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some(&format!("bind_group_layout {}", i)),
                })
        })
        .collect()
}

fn create_pipeline_layouts(
    ctx: &GraphicsContext,
    bind_group_layouts: &[BindGroupLayout],
) -> PipelineLayouts {
    let bind_group_layouts = &bind_group_layouts.iter().collect::<Vec<_>>();
    let create = |stages, mem_size| {
        ctx.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages,
                    range: 0..mem_size as u32,
                }],
            })
    };
    use shared::push_constants::shader::*;
    PipelineLayouts {
        render: create(wgpu::ShaderStages::FRAGMENT, FragmentConstants::mem_size()),
        compute: create(wgpu::ShaderStages::COMPUTE, ComputeConstants::mem_size()),
    }
}
