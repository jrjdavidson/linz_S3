use clap::Parser;
use env_logger::Env;
use linz_s3::args::SpatialFilterParams;
use linz_s3::process_tile_list;
use linz_s3::{search_catalog, Cli};
use log::{error, info};
use std::io::{self, Write};

/// Command-line arguments for the LINZ S3 filter tool.

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let spatial_filter_params = if args.spatial_filter.is_some() {
        Some(SpatialFilterParams::new(args.spatial_filter.unwrap()))
    } else {
        None
    };
    let tile_list =
        search_catalog(args.bucket, spatial_filter_params, args.by_collection_name).await;
    match tile_list {
        Ok(tile_list) => {
            for (index, (tile_paths, description)) in tile_list.iter().enumerate() {
                let tile_count = tile_paths.len();
                info!(
                    "{}. {} - Number of Tiles: {}",
                    index, description, tile_count
                );
            }
            match tile_list.len() {
                0 => {
                    info!("No datasets found.");
                    return;
                }
                1 => {
                    info!("Exactly 1 dataset found, processing...");
                    process_tile_list(&tile_list, 0, args.download).await;
                }
                _ => {
                    info!("{} datasets found.", tile_list.len());
                    if args.first {
                        process_tile_list(&tile_list, 0, args.download).await;
                    } else if args.by_size {
                        let index_of_longest = tile_list
                            .iter()
                            .enumerate()
                            .max_by_key(|(_, (vec, _))| vec.len())
                            .map(|(index, _)| index)
                            .unwrap();
                        process_tile_list(&tile_list, index_of_longest, args.download).await;
                    } else {
                        loop {
                            info!(
                                "Please choose a dataset (enter index or type 'cancel' to exit):"
                            );
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
            }
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }
}
