pub mod linz_s3_filter;
use linz_s3_filter::{Dataset, LinzBucket};
// use pyo3::prelude::*;

pub async fn search_catalog(
    bucket: Dataset,
    lat: f64,
    lon: f64,
    lat1: Option<f64>,
    lon1: Option<f64>,
) -> Vec<(Vec<String>, String)> {
    let linz_bucket = LinzBucket::initialise_catalog(bucket).await;
    if let (Some(lat1), Some(lon1)) = (lat1, lon1) {
        linz_bucket
            .get_tiles_from_lat_lon_range(lat, lon, lat1, lon1)
            .await
    } else {
        linz_bucket.get_tiles_from_lat_lon(lat, lon).await
    }
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
