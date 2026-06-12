use crate::args::PostProcessMode;
use log::info;
use oxigdal::vrt_builder::{build_vrt, VrtOptions};
use std::io;
use std::path::{Path, PathBuf};

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
    // Convert all paths to absolute for processing
    let sources_abs: Vec<PathBuf> = tiff_paths
        .iter()
        .map(|p| resolve_absolute(Path::new(p.as_str())))
        .collect::<io::Result<Vec<_>>>()?;

    let output_abs = resolve_absolute(Path::new(output_path))?;
    let output_parent = output_abs.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "output path must have a parent directory",
        )
    })?;

    // Convert to relative paths where possible (relative to VRT parent directory)
    let sources: Vec<PathBuf> = sources_abs
        .iter()
        .map(|abs_path| {
            abs_path
                .strip_prefix(output_parent)
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|_| abs_path.clone())
        })
        .collect();

    let source_refs: Vec<&Path> = sources.iter().map(PathBuf::as_path).collect();

    build_vrt(&source_refs, &output_abs, VrtOptions::default())
        .map_err(|err| io::Error::other(format!("oxigdal VRT build failed: {}", err)))?;

    info!("Post-processing succeeded: {}", output_abs.display());
    Ok(())
}

fn resolve_absolute(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    Ok(std::env::current_dir()?.join(path))
}

#[cfg(test)]
mod tests {
    use super::resolve_absolute;
    use std::path::Path;

    #[test]
    fn resolves_relative_paths_to_absolute() {
        let rel = Path::new("tiles/subfolder/tile_01.tif");
        let resolved = resolve_absolute(rel).expect("expected relative path to resolve");

        assert!(resolved.is_absolute());
        assert!(resolved.ends_with(rel));
    }

    #[test]
    fn keeps_absolute_paths_unchanged() {
        let cwd = std::env::current_dir().expect("cwd should be available");
        let abs = cwd.join("tiles/subfolder/mosaic.vrt");

        let resolved = resolve_absolute(&abs).expect("expected absolute path to pass through");

        assert_eq!(resolved, abs);
    }
}
