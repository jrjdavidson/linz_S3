use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use reqwest::get;
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
    let progress_bar = ProgressBar::new(tile_list[index].0.len() as u64);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files downloaded. Time left :{eta}")
        .unwrap()
        .progress_chars("#>-"));

    for tile_url in &tile_list[index].0 {
        if download {
            let url = tile_url.to_string();
            let progress_bar_clone = progress_bar.clone();
            let subfolder = tile_list[index].1.clone();
            let output_folder = cache_opt
                .clone()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(&subfolder);
            let file_name = Path::new(&url).file_name().unwrap().to_str().unwrap();
            let current_path = output_folder.join(file_name);
            // Check if the file already exists in the cache or current directory
            if current_path.exists() {
                info!(
                    "File already exists in current directory: {}",
                    current_path.display()
                );
                continue;
            }
            // Create the subfolder if it doesn't exist
            fs::create_dir_all(&output_folder).unwrap();
            println!("{}", current_path.display());
            tasks.push(task::spawn(async move {
                download_file(&url, &progress_bar_clone, current_path).await;
                progress_bar_clone.inc(1);
            }));
        } else {
            println!("{}", tile_url);
        }
    }
    for task in tasks {
        task.await.unwrap();
    }
    progress_bar.finish_with_message("Download complete");
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
