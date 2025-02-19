use clap::Parser;
use env_logger::Env;
use linz_s3::linz_s3_filter::Dataset;
use linz_s3::search_catalog;
use log::info;
use reqwest::get;
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};
use tokio::task;

#[derive(Parser)]
#[command(
    name = "linz_s3_filter",
    version = "1.0",
    author = "Jonathan Davidson <jrjddavidson@gmail.com>",
    about = "",
    allow_negative_numbers = true
)]
struct Args {
    /// The bucket to search, one of imagery, or elevation
    bucket: Dataset,
    lat: f64,
    lon: f64,
    lat1: Option<f64>,
    lon1: Option<f64>,
    /// Flag to download the files
    #[arg(short, long)]
    download: bool,
}
#[tokio::main]

async fn main() {
    let args = Args::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // env_logger::Builder::from_default_env().init();

    let tile_list: Vec<(Vec<String>, String)> =
        search_catalog(args.bucket, args.lat, args.lon, args.lat1, args.lon1).await;
    // Process the results
    for (i, (tile_paths, description)) in tile_list.iter().enumerate() {
        // Example processing: count the number of tiles
        let tile_count = tile_paths.len();
        info!("{}. {} - Number of Tiles: {}", i, description, tile_count);
    }
    // Prompt the user to choose a dataset
    println!("Please choose a dataset (enter index):");
    print!("> ");
    io::stdout().flush().unwrap(); // Ensure the prompt is displayed

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    match input.parse::<usize>() {
        Ok(index) if index < tile_list.len() => {
            let mut tasks = vec![];
            for line in &tile_list[index].0 {
                if args.download {
                    let url = line.to_string();
                    tasks.push(task::spawn(async move {
                        download_file(&url).await;
                    }));
                } else {
                    println!("{}", line);
                }
            }
            for task in tasks {
                task.await.unwrap();
            }
        }
        _ => {
            info!("Invalid index. Please enter a valid number.");
        }
    }
}
async fn download_file(url: &str) {
    let response = get(url).await.unwrap();
    let path = Path::new(url).file_name().unwrap().to_str().unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(&response.bytes().await.unwrap()).unwrap();
    info!("Downloaded: {}", path);
}
