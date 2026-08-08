mod context;
mod pipeline;
mod texture;

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
    pub tex_coords: na::Point2<f32>,
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
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const VERTICES: &[ModelVertex] = &[
    ModelVertex {
        position: na::Point3::new(-0.0868241, 0.49240386, 0.0),
        tex_coords: na::Point2::new(0.4131759, 0.00759614),
    },
    ModelVertex {
        position: na::Point3::new(-0.49513406, 0.06958647, 0.0),
        tex_coords: na::Point2::new(0.0048659444, 0.43041354),
    },
    ModelVertex {
        position: na::Point3::new(-0.21918549, -0.44939706, 0.0),
        tex_coords: na::Point2::new(0.28081453, 0.949397),
    },
    ModelVertex {
        position: na::Point3::new(0.35966998, -0.3473291, 0.0),
        tex_coords: na::Point2::new(0.85967, 0.84732914),
    },
    ModelVertex {
        position: na::Point3::new(0.44147372, 0.2347359, 0.0),
        tex_coords: na::Point2::new(0.9414737, 0.2652641),
    },
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

pub struct Engine {
    window: Arc<Window>,
    context: RenderContext,
    depth_texture: texture::Texture,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
}

impl Engine {
    pub async fn new(window: Arc<Window>) -> color_eyre::Result<Self> {
        let context = RenderContext::new(window.clone()).await?;

        let diffuse_bytes = include_bytes!("../../res/bankrupt.jpg");
        let diffuse_texture = texture::Texture::from_bytes(
            &context.device,
            &context.queue,
            diffuse_bytes,
            "bankrupt.jpg",
        )
        .unwrap();

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

        let diffuse_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("diffuse_bind_group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
            });

        let depth_texture = texture::Texture::create_depth_texture(
            &context.device,
            &context.surface_config,
            "depth_texture",
        );

        let render_pipeline = PipelineBuilder::new()
            .with_labels("Render Pipeline Layout", "Render Pipeline")
            .with_bind_group_layouts(vec![Some(&texture_bind_group_layout)])
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

        let index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        Ok(Self {
            window,
            context,
            depth_texture,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len() as u32,
            diffuse_bind_group,
            diffuse_texture,
        })
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.context.configure_surface(Some(size));
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.context.device,
                &self.context.surface_config,
                "depth_texture",
            );
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
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        drop(render_pass);

        self.context.queue.submit([encoder.finish()]);
        self.context.queue.present(output);

        Ok(())
    }
}
