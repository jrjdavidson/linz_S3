use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Neither spatial filter nor content filter were provided. This will cause a search of the whole bucket- aborting.")]
    NoFilterProvided,
    #[error("Cannot specify both dimensions (height_m, width_m) and a coordinate range. Please choose one.")]
    DimensionAndCoordinateRange,
}
