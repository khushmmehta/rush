use std::ops::Range;

use nalgebra as na;
use wgpu::util::DeviceExt;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrimitiveVertex {
    pub position: na::Point3<f32>,
    pub tex_coords: na::Point2<f32>,
}

impl Vertex for PrimitiveVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<PrimitiveVertex>() as wgpu::BufferAddress,
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

pub struct Model {
    pub meshes: Vec<Mesh>,
}

pub struct Mesh {
    #[allow(unused)]
    pub name: String,
    pub primitives: Vec<Primitive>,
}

pub struct Primitive {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl Primitive {
    pub fn generate(device: &wgpu::Device, vertices: &[PrimitiveVertex], indices: &[u32]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_elements = indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            num_elements,
        }
    }
}

pub trait DrawModel<'a> {
    #[allow(unused)]
    fn draw_mesh(&mut self, mesh: &'a Mesh);
    fn draw_primitive_instanced(&mut self, mesh: &'a Primitive, instances: Range<u32>);
    fn draw_model(&mut self, model: &'a Model);
    fn draw_model_instanced(&mut self, model: &'a Model, instances: Range<u32>);
}
impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh) {
        for prim in &mesh.primitives {
            self.draw_primitive_instanced(prim, 0..1);
        }
    }

    fn draw_primitive_instanced(&mut self, prim: &'b Primitive, instances: Range<u32>) {
        self.set_vertex_buffer(0, prim.vertex_buffer.slice(..));
        self.set_index_buffer(prim.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..prim.num_elements, 0, instances);
    }
    fn draw_model(&mut self, model: &'b Model) {
        self.draw_model_instanced(model, 0..1);
    }

    fn draw_model_instanced(&mut self, model: &'b Model, instances: Range<u32>) {
        for mesh in &model.meshes {
            for prim in &mesh.primitives {
                self.draw_primitive_instanced(prim, instances.clone());
            }
        }
    }
}
