use std::process::Command;

pub fn build_vrt_from_paths(tiff_paths: Vec<String>, output_path: &str) {
    let mut command = Command::new("gdalbuildvrt");
    command.arg(output_path);
    for path in tiff_paths {
        command.arg(path);
    }

    let output = command.output().expect("Failed to execute command");

    if output.status.success() {
        info!("VRT file created successfully.");
    } else {
        error!(
            "Error creating VRT file: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
