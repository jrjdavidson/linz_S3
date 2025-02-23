pub mod linz_s3_filter;
use std::error::Error;

use linz_s3_filter::{Dataset, LinzBucket};
// use pyo3::prelude::*;
pub async fn search_catalog(
    bucket: Dataset,
    lat: f64,
    lon: f64,
    lat2: Option<f64>,
    lon2: Option<f64>,
    height_m: Option<f64>,
    width_m: Option<f64>,
) -> Result<Vec<(Vec<String>, String)>, Box<dyn Error>> {
    let linz_bucket = LinzBucket::initialise_catalog(bucket).await;

    if (height_m.is_some() || width_m.is_some()) && (lat2.is_some() || lon2.is_some()) {
        return Err("Cannot specify both dimensions (height_m, width_m) and a coordinate range (lat2, lon2). Please choose one.".into());
    }

    let tiles = if let (Some(height_m), Some(width_m)) = (height_m, width_m) {
        linz_bucket
            .get_tiles_from_point_and_dimension(lat, lon, width_m, height_m)
            .await
    } else if let (Some(lat2), Some(lon2)) = (lat2, lon2) {
        linz_bucket
            .get_tiles_from_lat_lon_range(lat, lon, lat2, lon2)
            .await
    } else {
        linz_bucket.get_tiles_from_lat_lon(lat, lon).await
    };

    Ok(tiles)
}
//needs asuync pyo3?
// #[pyfunction]
// fn get_tiles(lat: f64, lon: f64) -> PyResult<Vec<(Vec<String>, String)>> {
//     Ok(search_catalog(lat, lon))
// }

// #[pymodule]
// fn linz_s3(py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(get_tiles, m)?)?;
//     Ok(())
// }
