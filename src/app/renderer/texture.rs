pub mod mipmapper;

use image::GenericImageView;

pub struct TextureBuilder {
    img: image::DynamicImage,
    texture_label: Option<String>,
    sampler_label: Option<String>,
    mipmapping_enabled: bool,
}

impl TextureBuilder {
    fn new(img: image::DynamicImage) -> Self {
        Self {
            img,
            texture_label: None,
            sampler_label: None,
            mipmapping_enabled: false,
        }
    }

    pub fn with_labels(mut self, texture_label: String, sampler_label: String) -> Self {
        self.texture_label = Some(texture_label);
        self.sampler_label = Some(sampler_label);
        self
    }

    pub fn with_mipmaps(mut self, enabled: bool) -> Self {
        self.mipmapping_enabled = enabled;
        self
    }

    pub fn build(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> color_eyre::Result<Texture> {
        let rgba = self.img.to_rgba8();
        let dimensions = self.img.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let mip_level_count = if self.mipmapping_enabled {
            dimensions.0.min(dimensions.1).ilog2() + 1
        } else {
            1
        };

        let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let usage = if self.mipmapping_enabled {
            usage | wgpu::TextureUsages::RENDER_ATTACHMENT
        } else {
            usage
        };

        let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: self.texture_label.as_deref(),
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &wgpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: self.sampler_label.as_deref(),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: if self.mipmapping_enabled {
                wgpu::FilterMode::Linear
            } else {
                wgpu::FilterMode::Nearest
            },
            mipmap_filter: if self.mipmapping_enabled {
                wgpu::MipmapFilterMode::Linear
            } else {
                wgpu::MipmapFilterMode::Nearest
            },
            lod_min_clamp: 0.0,
            lod_max_clamp: if self.mipmapping_enabled {
                f32::MAX
            } else {
                0.0
            },
            anisotropy_clamp: if self.mipmapping_enabled { 16 } else { 1 },
            ..Default::default()
        });

        let texture = Texture {
            texture: wgpu_texture,
            view,
            sampler,
        };

        if self.mipmapping_enabled {
            mipmapper::generate_mipmaps(device, queue, &texture);
        }

        Ok(texture)
    }
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("depth_texture_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> TextureBuilder {
        let img = image::load_from_memory(bytes).unwrap();
        TextureBuilder::new(img)
    }

    pub fn from_image(img: image::DynamicImage) -> TextureBuilder {
        TextureBuilder::new(img)
    }
}
