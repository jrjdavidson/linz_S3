use crate::linz_s3_filter::dataset;
use clap::{builder::ValueParser, command, Parser, Subcommand};

/// Enum for search mode.
#[derive(Parser)]
#[command(
    name = "linz_s3_filter",
    version = "0.4.6",
    author = "Jonathan Davidson <jrjdavidson@gmail.com>",
    about = "A tool to search for, filter, and download datasets from LINZ S3 buckets.",
    allow_negative_numbers = true
)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The dataset bucket to search (e.g., imagery or elevation).
    pub bucket: dataset::BucketName,
    /// Search mode: "coordinate" for lat/lon range, "area" for search by approx height/width in m.
    #[command(subcommand)]
    pub spatial_filter: Option<SpatialFilter>,
    /// Download the tiles instead of just printing the URLs. Can be used with --cache to download to a specific directory, otherwise will download to the current directory regardless of the presence of the file in the current directory.
    #[arg(short, long)]
    pub download: bool,
    /// Cache directory for downloaded tiles.
    #[arg(short, long, value_parser = folder_parser(), requires = "download")]
    pub cache: Option<String>,
    /// Automatically select the first dataset listed. Datesets are ordered by resolution first, and within each resolution level, alphabetically.
    #[arg(short = 'f', group = "auto_select", long)]
    pub by_first_index: bool,
    /// Automatically select the nth dataset found. Will default to the first if not specified.
    #[arg(short = 'i', long, group = "auto_select")]
    pub by_index: Option<usize>,
    /// Automatically select all datasets. Useful for downloading all datasets that meet the search criteria.
    #[arg(short = 'a', long, group = "auto_select")]
    pub by_all: bool,
    /// Automatically select the dataset with the most tiles. Can be useful for downloading the dataset with the highest area of coverage, however this is not always the case.
    #[arg(short = 's', group = "auto_select", long)]
    pub by_size: bool,
    /// Filter by collection name. Can be used multiple times, will match any of the provided names.
    #[arg(short = 'n', long)]
    pub include_collection_name: Option<Vec<String>>,
    /// Exclude collections by name. Can be used multiple times, will exclude any of the provided names. Exclusion takes precedence over inclusion "include_collection_name" filter.
    #[arg(short = 'x', long)]
    pub exclude_collection_name: Option<Vec<String>>,
    /// Set the log level (e.g., error, warn, info, debug, trace).
    #[arg(short, long, default_value = "info", value_parser = log_level_parser())]
    pub log_level: String,
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
    /// A Spatial filter to filter by point and search area.
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

use std::{path::Path, str::FromStr};

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

fn folder_parser() -> ValueParser {
    ValueParser::new(|s: &str| {
        let path = Path::new(s);
        if path.is_dir() {
            Ok(s.to_string())
        } else {
            Err(format!("'{}' is not a valid directory", s))
        }
    })
}

fn log_level_parser() -> ValueParser {
    ValueParser::new(|s: &str| match s.to_lowercase().as_str() {
        "error" | "warn" | "info" | "debug" | "trace" => Ok(s.to_string()),
        _ => Err(format!("Invalid log level: {}", s)),
    })
}
