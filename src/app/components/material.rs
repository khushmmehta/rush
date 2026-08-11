use crate::app::renderer::texture;

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        name: &str,
        diffuse_texture: texture::Texture,
        layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&[name, "__bind_group"].join("")),
            layout,
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

        Material {
            name: name.to_string(),
            diffuse_texture,
            bind_group,
        }
    }
}
