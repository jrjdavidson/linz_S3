use clap::{Parser, Subcommand};
use env_logger::Env;
use linz_s3::download::auto_choose_index;
use linz_s3::search_catalog;
use linz_s3::{process_tile_list, search::Dataset};
use log::{error, info};
use std::io::{self, Write};

/// Command-line arguments for the LINZ S3 filter tool.
#[derive(Parser)]
#[command(
    name = "linz_s3_filter",
    version = "0.2.1",
    author = "Jonathan Davidson <jrjddavidson@gmail.com>",
    about = "A tool to search and download datasets from LINZ S3.",
    allow_negative_numbers = true
)]
#[command(propagate_version = true)]
struct Args {
    /// The dataset bucket to search (e.g., imagery or elevation).
    bucket: Dataset,
    /// Search mode: "coordinates" for lat/lon range, "dimensions" for search by approx height/width in m.
    #[command(subcommand)]
    command: Option<SearchCommands>,
    #[arg(short, long)]
    download: bool,
    /// Flag to skip user input and grab the first value of tile_list.
    #[arg(long)]
    condition: Option<String>,
}

#[derive(Subcommand)]
enum SearchCommands {
    /// does testing things
    #[command(allow_negative_numbers = true)]
    CoordinateSearch {
        /// Latitude of the point to search.
        lat1: f64,
        /// Longitude of the point to search.
        lon1: f64,
        lat2: Option<f64>,
        /// Optional second longitude or width in meters.
        lon2: Option<f64>,
    },
    #[command(allow_negative_numbers = true)]
    AreaSearch {
        lat: f64,
        /// Longitude of the point to search.
        lon: f64,
        /// Width in meters. If no other argument is provided, this will also be the height.
        width: f64,
        /// Optional second argument height in meters.
        height: Option<f64>,
    },
}
#[tokio::main]
async fn main() {
    let args = Args::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let tile_list = match &args.command {
        Some(SearchCommands::CoordinateSearch {
            lat1,
            lon1,
            lat2,
            lon2,
        }) => search_catalog(args.bucket, *lat1, *lon1, *lat2, *lon2, None, None).await,
        Some(SearchCommands::AreaSearch {
            lat,
            lon,
            height,
            width,
        }) => search_catalog(args.bucket, *lat, *lon, None, None, Some(*width), *height).await,
        None => {
            error!("No command provided. Use the --help flag for more information.");
            return;
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
            if args.condition.is_some() {
                let index: usize = auto_choose_index(&tile_list, &args.condition.unwrap());
                process_tile_list(&tile_list, index, args.download).await;
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
