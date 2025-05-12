use clap::ValueEnum;
use stac::Item;

pub struct MatchingItems {
    pub title: String,
    pub items: Vec<Item>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum BucketName {
    Elevation,
    Imagery,
}

impl LinzBucketName {
    pub fn as_str(&self) -> &str {
        match self {
            BucketName::Elevation => "https://nz-elevation.s3.ap-southeast-2.amazonaws.com",
            BucketName::Imagery => "https://nz-imagery.s3.ap-southeast-2.amazonaws.com",
        }
    }
}
