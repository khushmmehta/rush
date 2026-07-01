mod context;
mod pipeline;

use context::RenderContext;
use pipeline::PipelineBuilder;
use std::sync::Arc;
use winit::window::Window;

pub struct Engine {
    window: Arc<Window>,
    context: RenderContext,
    render_pipeline: wgpu::RenderPipeline,
}

impl Engine {
    pub async fn new(window: Arc<Window>) -> color_eyre::Result<Self> {
        let context = RenderContext::new(window.clone()).await?;
        let pipeline = PipelineBuilder::new()
            .with_labels("Render Pipeline Layout", "Render Pipeline")
            .with_shader(
                &context
                    .device
                    .create_shader_module(wgpu::include_wgsl!("../../res/shaders/shader.wgsl")),
            )
            .build(&context.device);

        Ok(Self {
            window,
            context,
            render_pipeline: pipeline,
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
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        self.context.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }
}
