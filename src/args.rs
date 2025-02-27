use crate::linz_s3_filter::dataset;
use clap::{command, Parser, Subcommand};

/// Enum for search mode.

#[derive(Parser)]
#[command(
    name = "linz_s3_filter",
    version = "0.2.1",
    author = "Jonathan Davidson <jrjddavidson@gmail.com>",
    about = "A tool to search for, filter, and download datasets from LINZ S3 buckets.",
    allow_negative_numbers = true
)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The dataset bucket to search (e.g., imagery or elevation).
    pub bucket: dataset::LinzBucketName,
    /// Search mode: "coordinates" for lat/lon range, "dimensions" for search by approx height/width in m.
    #[command(subcommand)]
    pub spatial_filter: Option<SpatialFilter>,
    #[arg(short, long)]
    pub download: bool,
    #[arg(short, long)]
    pub first: bool,
    #[arg(short = 'n', group = "filter_content", long)]
    pub by_collection_name: Option<String>,
    #[arg(short = 's', group = "filter_content", long)]
    pub by_size: bool,
}

#[derive(Subcommand)]
pub enum SpatialFilter {
    /// Filter by coordinates.
    #[command(allow_negative_numbers = true)]
    Coordinate {
        /// Latitude of the point to search.
        lat1: f64,
        /// Longitude of the point to search.
        lon1: f64,
        /// Optional second latitude.
        lat2_opt: Option<f64>,
        /// Optional second longitude or width in meters.
        lon2_opt: Option<f64>,
    },
    /// Filter by area.
    #[command(allow_negative_numbers = true)]
    Area {
        /// Latitude of the center of filter area.
        lat1: f64,
        /// Longitude of the center of filter area.
        lon1: f64,
        /// Width in meters. If no other argument is provided, this will also be the height.
        width_m: f64,
        /// Optional second argument height in meters.
        height_m_opt: Option<f64>,
    },
}

#[derive(Debug)]
pub struct SpatialFilterParams {
    pub lat1: f64,
    pub lon1: f64,
    pub lat2_opt: Option<f64>,
    pub lon2_opt: Option<f64>,
    pub width_m_opt: Option<f64>,
    pub height_m_opt: Option<f64>,
}
impl SpatialFilterParams {
    pub fn new(command: SpatialFilter) -> Self {
        match command {
            SpatialFilter::Coordinate {
                lat1,
                lon1,
                lat2_opt,
                lon2_opt,
            } => Self {
                lat1,
                lon1,
                lat2_opt,
                lon2_opt,
                width_m_opt: None,
                height_m_opt: None,
            },
            SpatialFilter::Area {
                lat1,
                lon1,
                width_m,
                height_m_opt,
            } => Self {
                lat1,
                lon1,
                lat2_opt: None,
                lon2_opt: None,
                width_m_opt: Some(width_m),
                height_m_opt,
            },
        }
    }
}
