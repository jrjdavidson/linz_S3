use clap::{Parser, ValueEnum};
use env_logger::Env;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use linz_s3::linz_s3_filter::Dataset;
use linz_s3::search_catalog;
use log::{error, info};
use reqwest::get;
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};
use tokio::task;
/// Enum for search mode.
#[derive(ValueEnum, Clone, Debug)]
enum SearchMode {
    Coordinates,
    Dimensions,
}
/// Command-line arguments for the LINZ S3 filter tool.
#[derive(Parser)]
#[command(
    name = "linz_s3_filter",
    version = "1.0",
    author = "Jonathan Davidson <jrjddavidson@gmail.com>",
    about = "A tool to search and download datasets from LINZ S3.",
    allow_negative_numbers = true
)]
struct Args {
    /// The dataset bucket to search (e.g., imagery or elevation).
    bucket: Dataset,
    /// Latitude of the point to search.
    lat: f64,
    /// Longitude of the point to search.
    lon: f64,
    /// Search mode: "coordinates" for lat/lon range, "dimensions" for search by approx height/width in m.
    #[arg(short, long, default_value = "coordinates")]
    search_mode: SearchMode,
    /// Optional second latitude or height in meters.
    arg1: Option<f64>,
    /// Optional second longitude or width in meters.
    arg2: Option<f64>,
    /// Flag to download the files.
    #[arg(short, long)]
    download: bool,
    /// Flag to skip user input and grab the first value of tile_list.
    #[arg(long)]
    skip_input: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let tile_list = match args.search_mode {
        SearchMode::Coordinates => {
            search_catalog(
                args.bucket,
                args.lat,
                args.lon,
                args.arg1,
                args.arg2,
                None,
                None,
            )
            .await
        }
        SearchMode::Dimensions => {
            if args.arg1.is_none() || args.arg2.is_none() {
                error!("Error: Both arg1 (height in meters) and arg2 (width in meters) must be specified for dimension search.");
                std::process::exit(1);
            }
            search_catalog(
                args.bucket,
                args.lat,
                args.lon,
                None,
                None,
                args.arg1,
                args.arg2,
            )
            .await
        }
    };

    match tile_list {
        Ok(tile_list) => {
            for (index, (tile_paths, description)) in tile_list.iter().enumerate() {
                let tile_count = tile_paths.len();
                info!(
                    "{}. {} - Number of Tiles: {}",
                    index, description, tile_count
                );
            }
            if args.skip_input {
                process_tile_list(&tile_list, 0, args.download).await;
            } else {
                loop {
                    info!("Please choose a dataset (enter index or type 'cancel' to exit):");
                    info!("> ");
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim();

                    if input.eq_ignore_ascii_case("cancel") {
                        info!("Operation canceled.");
                        break;
                    }

                    match input.parse::<usize>() {
                        Ok(index) if index < tile_list.len() => {
                            process_tile_list(&tile_list, index, args.download).await;

                            break;
                        }
                        _ => {
                            error!("Invalid index. Please enter a valid number.");
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }
}
async fn process_tile_list(tile_list: &[(Vec<String>, String)], index: usize, download: bool) {
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
/// Downloads a file from the given URL and updates the progress bar.
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
