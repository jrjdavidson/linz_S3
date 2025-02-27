use clap::ValueEnum;
use stac::Item;

pub struct MatchingItems {
    pub title: String,
    pub items: Vec<Item>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LinzBucketName {
    Elevation,
    Imagery,
}

impl LinzBucketName {
    pub fn as_str(&self) -> &str {
        match self {
            LinzBucketName::Elevation => "nz-elevation",
            LinzBucketName::Imagery => "nz-imagery",
        }
    }
}
