use crate::search::linz_s3_filter::Dataset;
use clap::{Parser, ValueEnum};

/// Enum for search mode.
#[derive(ValueEnum, Clone, Debug)]
pub enum SearchMode {
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
pub struct Args {
    /// The dataset bucket to search (e.g., imagery or elevation).
    pub bucket: Dataset,
    /// Latitude of the point to search.
    pub lat: f64,
    /// Longitude of the point to search.
    pub lon: f64,
    /// Search mode: "coordinates" for lat/lon range, "dimensions" for search by approx height/width in m.
    #[arg(short, long, default_value = "coordinates")]
    pub search_mode: SearchMode,
    /// Optional second latitude or height in meters.
    pub arg1: Option<f64>,
    /// Optional second longitude or width in meters.
    pub arg2: Option<f64>,
    /// Flag to download the files.
    #[arg(short, long)]
    pub download: bool,
    /// Flag to skip user input and grab the first value of tile_list.
    #[arg(long)]
    pub skip_input: bool,
}
