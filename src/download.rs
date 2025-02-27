use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::get;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio::task;

pub async fn process_tile_list(tile_list: &[(Vec<String>, String)], index: usize, download: bool) {
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
            tasks.push(task::spawn(async move {
                download_file(&url, &progress_bar_clone).await;
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

async fn download_file(url: &str, progress_bar: &ProgressBar) {
    let response = get(url).await.unwrap();

    let file_name = Path::new(url).file_name().unwrap().to_str().unwrap();
    let mut file = File::create(file_name).unwrap();

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        file.write_all(&chunk).unwrap();
        progress_bar.tick();
    }
}
