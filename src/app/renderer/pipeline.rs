pub struct PipelineBuilder<'a> {
    layout_label: Option<String>,
    pipeline_label: Option<String>,
    shader: Option<&'a wgpu::ShaderModule>,
    bind_group_layouts: Vec<Option<&'a wgpu::BindGroupLayout>>,
    buffer_layouts: Vec<Option<wgpu::VertexBufferLayout<'static>>>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new() -> Self {
        Self {
            layout_label: None,
            pipeline_label: None,
            shader: None,
            bind_group_layouts: vec![],
            buffer_layouts: vec![],
        }
    }

    pub fn with_labels(mut self, layout_label: &str, pipeline_label: &str) -> Self {
        self.layout_label = Some(layout_label.to_string());
        self.pipeline_label = Some(pipeline_label.to_string());
        self
    }

    pub fn with_shader(mut self, module: &'a wgpu::ShaderModule) -> Self {
        self.shader = Some(module);
        self
    }

    #[allow(unused)]
    pub fn with_bind_group_layouts(
        mut self,
        bind_group_layouts: Vec<Option<&'a wgpu::BindGroupLayout>>,
    ) -> Self {
        self.bind_group_layouts = bind_group_layouts;
        self
    }

    pub fn with_buffer_layouts(
        mut self,
        buffer_layouts: Vec<Option<wgpu::VertexBufferLayout<'static>>>,
    ) -> Self {
        self.buffer_layouts = buffer_layouts;
        self
    }

    pub fn build(&self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        let rp_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self.layout_label.as_deref(),
            bind_group_layouts: self.bind_group_layouts.as_slice(),
            immediate_size: 0,
        });

        let shader = self
            .shader
            .ok_or(color_eyre::eyre::eyre!(format!(
                "Missing shader in {}",
                self.pipeline_label
                    .clone()
                    .unwrap_or("Unnamed Render Pipeline".to_string())
            )))
            .unwrap();

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.pipeline_label.as_deref(),
            layout: Some(&rp_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: self.buffer_layouts.as_slice(),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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
            multiview_mask: None,
            cache: None,
        })
    }
}
