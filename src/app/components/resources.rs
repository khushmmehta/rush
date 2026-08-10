use std::path::PathBuf;

use super::super::renderer::texture;

#[allow(unused)]
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
