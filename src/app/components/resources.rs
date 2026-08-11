use std::path::PathBuf;

use nalgebra as na;

use super::{super::renderer::texture, material, model};

pub fn load_path(file_name: &str) -> PathBuf {
    std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name)
}

pub fn load_binary(file_name: &str) -> color_eyre::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    Ok(std::fs::read(path)?)
}

pub fn load_texture(file_name: &str) -> color_eyre::Result<texture::TextureBuilder> {
    let data = load_binary(file_name)?;
    Ok(texture::Texture::from_bytes(&data))
}

pub fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: wgpu::BindGroupLayout,
) -> color_eyre::Result<model::Model> {
    let (doc, bufs, imgs) = gltf::import(load_path(file_name))?;

    let mut materials: Vec<material::Material> = Vec::with_capacity(doc.materials().len());

    for mat in doc.materials() {
        let diffuse_texture = match mat.pbr_metallic_roughness().base_color_texture() {
            Some(info) => {
                let img = &imgs[info.texture().index()];
                texture::Texture::from_image(gltf_img_to_dyn_img(img)?)
                    .with_labels(
                        [mat.name().unwrap(), "__diffuse_texture"].join(""),
                        [mat.name().unwrap(), "__diffuse_sampler"].join(""),
                    )
                    .with_mipmaps(true)
                    .build(device, queue)
            }
            None => {
                let img = image::DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
                    1,
                    1,
                    image::Rgba([255, 255, 255, 255]),
                ));
                texture::Texture::from_image(img)
                    .with_labels(
                        String::from("blank_texture"),
                        String::from("blank_texture_sampler"),
                    )
                    .with_mipmaps(false)
                    .build(device, queue)
            }
        }?;

        materials.push(material::Material::new(
            mat.name().unwrap(),
            diffuse_texture,
            &layout,
            device,
        ));
    }

    let meshes = doc
        .meshes()
        .map(|mesh| {
            let primitives = mesh
                .primitives()
                .map(|prim| {
                    let reader = prim.reader(|buf| Some(&bufs[buf.index()]));

                    let positions: Vec<[f32; 3]> = reader
                        .read_positions()
                        .ok_or_else(|| color_eyre::eyre::anyhow!("failed to read position(s)"))
                        .unwrap()
                        .collect();

                    let tex_coords: Vec<[f32; 2]> = reader
                        .read_tex_coords(0)
                        .map(|t| t.into_f32().collect())
                        .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

                    let vertices: Vec<model::PrimitiveVertex> = (0..positions.len())
                        .map(|i| model::PrimitiveVertex {
                            position: na::Point3::from(positions[i]),
                            tex_coords: na::Point2::from([
                                tex_coords[i][0],
                                1.0 - tex_coords[i][1],
                            ]),
                        })
                        .collect();

                    let indices: Vec<u32> = reader
                        .read_indices()
                        .ok_or_else(|| color_eyre::eyre::anyhow!("primitive missing indices"))
                        .unwrap()
                        .into_u32()
                        .collect();

                    model::Primitive::generate(
                        device,
                        vertices.as_slice(),
                        indices.as_slice(),
                        prim.material().index().unwrap_or(0),
                    )
                })
                .collect::<Vec<_>>();

            model::Mesh {
                name: mesh.name().unwrap_or(file_name).to_string(),
                primitives,
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}

fn gltf_img_to_dyn_img(img: &gltf::image::Data) -> color_eyre::Result<image::DynamicImage> {
    match img.format {
        gltf::image::Format::R8 => Ok(image::DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(img.width, img.height, img.pixels.clone()).unwrap(),
        )),
        gltf::image::Format::R8G8 => Ok(image::DynamicImage::ImageLumaA8(
            image::GrayAlphaImage::from_raw(img.width, img.height, img.pixels.clone()).unwrap(),
        )),
        gltf::image::Format::R8G8B8 => Ok(image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(img.width, img.height, img.pixels.clone()).unwrap(),
        )),
        gltf::image::Format::R8G8B8A8 => Ok(image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(img.width, img.height, img.pixels.clone()).unwrap(),
        )),
        _ => Err(color_eyre::eyre::eyre!("Unsupported image format!")),
    }
}
