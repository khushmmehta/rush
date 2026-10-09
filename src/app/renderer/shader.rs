/// Main scene shader (`res/shaders/shader.wgsl`). Expects `vs_main`, `fs_main`.
pub fn main() -> wgpu::ShaderModuleDescriptor<'static> {
    wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shader.spv"))
}

/// Blit shader for mipmap generation (`res/shaders/blit.wgsl`).
pub fn blit() -> wgpu::ShaderModuleDescriptor<'static> {
    wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/blit.spv"))
}
