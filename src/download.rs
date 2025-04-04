use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use reqwest::get;
use sanitize_filename::sanitize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::task;

pub async fn process_tile_list(
    tile_list: &[(Vec<String>, String)],
    index: usize,
    download: bool,
    cache_opt: Option<PathBuf>,
) {
    let mut tasks = vec![];
    let mut cache_count = 0;
    let progress_bar = ProgressBar::new(tile_list[index].0.len() as u64);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files downloaded. Time left :{eta}")
        .unwrap()
        .progress_chars("#>-"));
    info!("Starting downloads...");
    for tile_url in &tile_list[index].0 {
        if download {
            let url = tile_url.to_string();
            let progress_bar_clone = progress_bar.clone();
            let subfolder = sanitize(tile_list[index].1.clone());
            let output_folder = cache_opt
                .clone()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(&subfolder);
            let file_name = Path::new(&url).file_name().unwrap().to_str().unwrap();
            let current_path = output_folder.join(file_name);
            // Print file to stdout
            println!("{}", current_path.display());
            // Check if the file already exists in the cache or current directory
            if current_path.exists() {
                debug!(
                    "File already exists in current directory: {}",
                    current_path.display()
                );
                cache_count += 1;

                continue;
            }
            // Create the subfolder if it doesn't exist
            fs::create_dir_all(&output_folder).unwrap();
            tasks.push(task::spawn(async move {
                download_file(&url, &progress_bar_clone, current_path).await;
                progress_bar_clone.inc(1);
            }));
        } else {
            println!("{}", tile_url);
        }
    }
    let download_count = tasks.len();
    for task in tasks {
        task.await.unwrap();
    }
    progress_bar.finish_with_message("Download complete");
    info!(
        "{} files found in cache, {} files downloaded",
        cache_count, download_count
    );
}

async fn download_file(url: &str, progress_bar: &ProgressBar, output_file: PathBuf) {
    let response = get(url).await.unwrap();

    let mut file = File::create(output_file).unwrap();

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        file.write_all(&chunk).unwrap();
        progress_bar.tick();
    }
}
