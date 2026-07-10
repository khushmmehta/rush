mod context;
mod pipeline;

use context::RenderContext;
use nalgebra as na;
use pipeline::PipelineBuilder;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: na::Point3<f32>,
    pub color: na::Point3<f32>,
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// lib.rs
const VERTICES: &[ModelVertex] = &[
    ModelVertex {
        position: na::Point3::new(0.0, 0.5, 0.0),
        color: na::Point3::new(1.0, 0.0, 0.0),
    },
    ModelVertex {
        position: na::Point3::new(-0.5, -0.5, 0.0),
        color: na::Point3::new(0.0, 1.0, 0.0),
    },
    ModelVertex {
        position: na::Point3::new(0.5, -0.5, 0.0),
        color: na::Point3::new(0.0, 0.0, 1.0),
    },
];

pub struct Engine {
    window: Arc<Window>,
    context: RenderContext,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

impl Engine {
    pub async fn new(window: Arc<Window>) -> color_eyre::Result<Self> {
        let context = RenderContext::new(window.clone()).await?;
        let render_pipeline = PipelineBuilder::new()
            .with_labels("Render Pipeline Layout", "Render Pipeline")
            .with_shader(
                &context
                    .device
                    .create_shader_module(wgpu::include_wgsl!("../../res/shaders/shader.wgsl")),
            )
            .with_buffer_layouts(vec![Some(ModelVertex::desc())])
            .build(&context.device);

        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        Ok(Self {
            window,
            context,
            render_pipeline,
            vertex_buffer,
        })
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.context.configure_surface(size);
        }
    }

    pub fn render(&mut self) -> color_eyre::Result<()> {
        let output = match self.context.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.context.configure_surface(self.window.inner_size());
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
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, 0..1);

        drop(render_pass);

        self.context.queue.submit([encoder.finish()]);
        self.context.queue.present(output);

        Ok(())
    }
}
