mod context;
mod pipeline;
pub mod texture;

use context::RenderContext;
use pipeline::PipelineBuilder;
use std::sync::Arc;
use winit::window::Window;

use super::components::{
    camera,
    model::{self, DrawModel, Vertex},
};

pub struct Engine {
    window: Arc<Window>,
    context: RenderContext,
    camera: camera::Camera,
    pub camera_controller: camera::CameraController,
    camera_gpu: camera::CameraGPU,
    depth_texture: texture::Texture,
    render_pipeline: wgpu::RenderPipeline,
    gltf_model: model::Model,
}

impl Engine {
    pub async fn new(window: Arc<Window>) -> color_eyre::Result<Self> {
        let context = RenderContext::new(window.clone()).await?;

        let texture_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let depth_texture = texture::Texture::create_depth_texture(
            &context.device,
            &context.surface_config,
            "depth_texture",
        );

        let camera = camera::Builder::new()
            .position(0.0, 5.0, 10.0)
            .rotation(-90.0, -20.0)
            .perspective(
                context.surface_config.width,
                context.surface_config.height,
                45.0,
                0.1,
                1000.0,
            )
            .build();
        let camera_controller = camera::CameraController::new(10.0, 4.0);
        let (camera_gpu, camera_bind_group_layout) =
            camera::CameraGPU::new(&context.device, &camera);

        let render_pipeline = PipelineBuilder::new()
            .with_labels("Render Pipeline Layout", "Render Pipeline")
            .with_bind_group_layouts(vec![
                Some(&texture_bind_group_layout),
                Some(&camera_bind_group_layout),
            ])
            .with_shader(
                &context
                    .device
                    .create_shader_module(wgpu::include_wgsl!("../../res/shaders/shader.wgsl")),
            )
            .with_buffer_layouts(vec![Some(model::PrimitiveVertex::desc())])
            .build(&context.device);

        let gltf_model = super::components::resources::load_model(
            "stoned-cube.glb",
            &context.device,
            &context.queue,
            texture_bind_group_layout,
        )?;

        Ok(Self {
            window,
            context,
            camera,
            camera_controller,
            camera_gpu,
            depth_texture,
            render_pipeline,
            gltf_model,
        })
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.context.configure_surface(Some(size));
            self.camera.resize(size.width, size.height);
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.context.device,
                &self.context.surface_config,
                "depth_texture",
            );
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_gpu
            .update_buffer(&self.context.queue, &self.camera);
    }

    pub fn render(&mut self) -> color_eyre::Result<()> {
        let output = match self.context.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.context
                    .configure_surface(Some(self.window.inner_size()));
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                color_eyre::eyre::bail!("Lost device");
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                unreachable!("No error scope registered. Validation errors will panic!")
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.camera_gpu.bind_group, &[]);
        render_pass.draw_model(&self.gltf_model);

        drop(render_pass);

        self.context.queue.submit([encoder.finish()]);
        self.context.queue.present(output);

        Ok(())
    }
}
