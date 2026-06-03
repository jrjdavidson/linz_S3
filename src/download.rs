use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, info};
use reqwest::get;
use sanitize_filename::sanitize;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;
use tokio::{signal, task};

pub async fn process_tile_list(
    tile_list: &[(Vec<String>, String)],
    index: usize,
    download: bool,
    cache_opt: &Option<PathBuf>,
) {
    let mut tasks = vec![];
    let mut cache_count = 0;
    if download {
        let multiprogressbar = MultiProgress::new();

        info!("Starting downloads...");

        // Create a channel to signal cancellation
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

        // Spawn a task to listen for the interrupt signal
        tokio::spawn(async move {
            signal::ctrl_c().await.expect("Failed to listen for event");
            let _ = cancel_tx.send(());
        });

        for tile_url in &tile_list[index].0 {
            let multiprogressbar = multiprogressbar.clone();
            let url = tile_url.to_string();

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
            fs::create_dir_all(&output_folder).await.unwrap();
            tasks.push(task::spawn(async move {
                download_file(&url, current_path, multiprogressbar).await;
            }));
        }
        let download_count = tasks.len();

        // Wait for tasks to complete or cancellation signal
        tokio::select! {
            _ = async {
                for task in tasks {
                    task.await.unwrap();
                }
            } => {
                info!(
                    "{} files found in cache, {} files downloaded",
                    cache_count, download_count
                );
            },
            _ = cancel_rx => {
            info!("Download process interrupted by user");
            }
        }
    } else {
        //Just print the URLs
        info!("Download is disabled, printing URLs only:");
        for tile_url in &tile_list[index].0 {
            println!("{}", tile_url);
        }
    }
}

async fn download_file(url: &str, output_file: PathBuf, multi_progress: MultiProgress) {
    match get(url).await {
        Ok(response) => {
            let total_size = response.content_length().unwrap_or(0);

            let pb = multi_progress.add(ProgressBar::new(total_size));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}",
                    )
                    .unwrap()
                    .progress_chars("#>-"),
            );
            let path_str = Path::new(&url)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            pb.set_message(path_str.clone());
            pb.enable_steady_tick(Duration::from_millis(100));

            let mut file = File::create(output_file).await.unwrap();
            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.unwrap();
                file.write_all(&chunk).await.unwrap();
                pb.inc(chunk.len() as u64);
            }

            pb.finish_with_message(format!("{} - Done", path_str));
        }
        Err(e) => {
            eprintln!("Error downloading {}: {}", url, e);
        }
    }
}
