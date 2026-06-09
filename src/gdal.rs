use crate::args::PostProcessMode;
use log::info;
use oxigdal::vrt_builder::{build_vrt, VrtOptions};
use std::io;
use std::path::Path;

pub fn run_post_process(
    mode: PostProcessMode,
    tiff_paths: &[String],
    output_path: &str,
) -> io::Result<()> {
    match mode {
        PostProcessMode::Vrt => run_vrt_post_process(tiff_paths, output_path),
    }
}

fn run_vrt_post_process(tiff_paths: &[String], output_path: &str) -> io::Result<()> {
    let sources: Vec<&Path> = tiff_paths.iter().map(|p| Path::new(p.as_str())).collect();
    let output = Path::new(output_path);

    build_vrt(&sources, output, VrtOptions::default())
        .map_err(|err| io::Error::other(format!("oxigdal VRT build failed: {}", err)))?;

    info!("Post-processing succeeded: {}", output_path);
    Ok(())
}
