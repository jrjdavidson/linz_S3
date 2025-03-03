use crate::linz_s3_filter::dataset;
use clap::{builder::ValueParser, command, Parser, Subcommand};

/// Enum for search mode.

#[derive(Parser)]
#[command(
    name = "linz_s3_filter",
    version = "0.3.0",
    author = "Jonathan Davidson <jrjddavidson@gmail.com>",
    about = "A tool to search for, filter, and download datasets from LINZ S3 buckets.",
    allow_negative_numbers = true
)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The dataset bucket to search (e.g., imagery or elevation).
    pub bucket: dataset::LinzBucketName,
    /// Search mode: "coordinate" for lat/lon range, "area" for search by approx height/width in m.
    #[command(subcommand)]
    ///Filter spatially by coordinates or dimensions.
    pub spatial_filter: Option<SpatialFilter>,
    #[arg(short, long)]
    /// Download the tiles.
    pub download: bool,
    /// Automatically select the first dataset found, usually the highest resolution dataset.
    #[arg(short, group = "auto_select", long)]
    pub first: bool,
    /// Filter by collection name. Can be used multiple times, will match any of the provided names.
    #[arg(short = 'n', long)]
    pub by_collection_name: Option<Vec<String>>,
    /// Automatically select the dataset with the most tiles. Useful for downloading the dataset with the highest resolution and cover.
    #[arg(short = 's', group = "auto_select", long)]
    pub by_size: bool,
}

#[derive(Subcommand)]
pub enum SpatialFilter {
    /// A Spatial filter to filter by coordinates or area.
    #[command(allow_negative_numbers = true)]
    Coordinate {
        /// Latitude of the point to search.
        #[arg(value_parser = latitude_parser())]
        lat1: f64,
        /// Longitude of the point to search.
        #[arg(value_parser = longitude_parser())]
        lon1: f64,
        /// Optional second latitude. If this and lon2_opt are not provided, the spatial filter will return all tiles that include just the lat1, lon1 point.
        #[arg(value_parser = latitude_parser())]
        lat2_opt: Option<f64>,
        /// Optional second longitude.
        #[arg(value_parser = longitude_parser())]
        lon2_opt: Option<f64>,
    },
    /// A Spatial filter to filter by pooint and search area.
    #[command(allow_negative_numbers = true)]
    Area {
        /// Latitude of the center of filter area.
        #[arg(value_parser = latitude_parser())]
        lat1: f64,
        /// Longitude of the center of filter area.
        #[arg(value_parser = longitude_parser())]
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
use std::str::FromStr;
fn latitude_parser() -> ValueParser {
    ValueParser::new(|s: &str| {
        let val = f64::from_str(s).map_err(|_| format!("Invalid latitude: {}", s))?;
        if (-90.0..=90.0).contains(&val) {
            Ok(val)
        } else {
            Err(format!(
                "Latitude must be between -90 and 90 degrees: {}",
                s
            ))
        }
    })
}

fn longitude_parser() -> ValueParser {
    ValueParser::new(|s: &str| {
        let val = f64::from_str(s).map_err(|_| format!("Invalid longitude: {}", s))?;
        if (-180.0..=180.0).contains(&val) {
            Ok(val)
        } else {
            Err(format!(
                "Longitude must be between -180 and 180 degrees: {}",
                s
            ))
        }
    })
}
