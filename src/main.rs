use clap::Parser;
use env_logger::Env;
use linz_s3::args::SpatialFilterParams;
use linz_s3::linz_s3_filter::bucket_config;
use linz_s3::process_tile_list;
use linz_s3::{search_catalog, Cli};
use log::{error, info};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
/// Command-line arguments for the LINZ S3 filter tool.

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or(&args.log_level)).init();
    bucket_config::ConfigFile::init();

    let spatial_filter_params = if args.spatial_filter.is_some() {
        Some(SpatialFilterParams::new(args.spatial_filter.unwrap()))
    } else {
        None
    };
    let cache_path_opt: &Option<PathBuf> = &args.cache.map(|cache| Path::new(&cache).to_owned());
    let tile_list = search_catalog(
        args.bucket,
        spatial_filter_params,
        args.include_collection_name,
        args.exclude_collection_name,
        args.thread_multiplier,
    )
    .await;
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
                    process_tile_list(&tile_list, 0, !args.disable_download, cache_path_opt).await;
                }
                _ => {
                    info!("{} datasets found.", tile_list.len());
                    if args.by_first_index || args.by_index.is_some() {
                        let index = args.by_index.unwrap_or(0); // if none then by_first_index is set

                        if index < tile_list.len() {
                            info!(
                                "Automatically picked dataset by index {}: {}",
                                index, &tile_list[index].1
                            );

                            process_tile_list(
                                &tile_list,
                                index,
                                !args.disable_download,
                                cache_path_opt,
                            )
                            .await;
                        } else {
                            eprintln!("Error: Index {} is out of bounds. There are only {} datasets available.", index, tile_list.len());
                        }
                    } else if args.by_size {
                        let index_of_longest = tile_list
                            .iter()
                            .enumerate()
                            .rev()
                            .max_by_key(|(_, (vec, _))| vec.len())
                            .map(|(index, _)| index)
                            .unwrap();
                        info!(
                            "Automatically picked dataset with most tiles: {}",
                            &tile_list[index_of_longest].1
                        );

                        process_tile_list(
                            &tile_list,
                            index_of_longest,
                            !args.disable_download,
                            cache_path_opt,
                        )
                        .await;
                    } else if args.by_all {
                        info!("Automatically picked all datasets.");
                        for (index, _) in tile_list.iter().enumerate() {
                            process_tile_list(
                                &tile_list,
                                index,
                                !args.disable_download,
                                cache_path_opt,
                            )
                            .await;
                        }
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
                                    info!(
                                        "You picked dataset number {}: {}",
                                        index, &tile_list[index].1
                                    );

                                    process_tile_list(
                                        &tile_list,
                                        index,
                                        !args.disable_download,
                                        cache_path_opt,
                                    )
                                    .await;

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
            e.report();
        }
    }
}
