use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{env, fs};

use color_eyre::Result;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use glob::GlobError;
use naga::back::spv::{self, WriterFlags};
use naga::valid::{Capabilities, ValidationFlags, Validator};

const SHADER_DIR: &str = "res/shaders";

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR")?;

    for source in wgsl_sources()? {
        println!("cargo:rerun-if-changed={}", source.display());

        let stem = source
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("shader path has no UTF-8 stem: {}", source.display())
            })?;

        let cached = Path::new(SHADER_DIR).join(format!("{stem}.spv"));

        let spirv = if is_fresh(&cached, &source) {
            fs::read(&cached)?
        } else {
            let spirv = compile(&fs::read_to_string(&source)?)?;
            fs::write(&cached, &spirv)?;
            spirv
        };

        fs::write(format!("{out_dir}/{stem}.spv"), spirv)?;
    }

    // `res/` as a whole, so asset edits still retrigger the copy below.
    println!("cargo:rerun-if-changed=res");

    let mut opts = CopyOptions::new();
    opts.overwrite = true;
    copy_items(&["res/"], &out_dir, &opts)?;

    Ok(())
}

fn wgsl_sources() -> Result<Vec<PathBuf>> {
    glob::glob(&format!("{SHADER_DIR}/*.wgsl"))?
        .collect::<Result<_, GlobError>>()
        .map_err(Into::into)
}

/// True if `cached` exists and is at least as new as `source`.
fn is_fresh(cached: &Path, source: &Path) -> bool {
    modified(cached)
        .zip(modified(source))
        .is_some_and(|(cached, source)| cached >= source)
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

fn compile(wgsl: &str) -> Result<Vec<u8>> {
    let module = naga::front::wgsl::parse_str(wgsl)?;
    let info = Validator::new(ValidationFlags::all(), Capabilities::empty()).validate(&module)?;

    // Pinned, not `Options::default()`: that adds `DEBUG` under debug
    // assertions, and `ADJUST_COORDINATE_SPACE` negates `gl_Position.y`,
    // flipping the view and reversing winding.
    // SPIR-V 1.6 needs Vulkan 1.3+.
    let options = spv::Options {
        lang_version: (1, 6),
        // `Options::default()` flips `gl_Position.y`, inverting the view.
        flags: WriterFlags::LABEL_VARYINGS | WriterFlags::CLAMP_FRAG_DEPTH,
        ..Default::default()
    };

    // `None` keeps every entry point in one module.
    let words = spv::write_vec(&module, &info, &options, None)?;

    Ok(words.iter().flat_map(|word| word.to_le_bytes()).collect())
}
