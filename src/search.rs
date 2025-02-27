use crate::error::MyError;
use crate::linz_s3_filter::{dataset, linz_bucket::LinzBucket};

use crate::args::SpatialFilterParams;

// use pyo3::prelude::*;

pub async fn search_catalog(
    bucket: dataset::LinzBucketName,
    spatial_params: Option<SpatialFilterParams>,
    collection_name_filter_opt: Option<String>,
) -> Result<Vec<(Vec<String>, String)>, MyError> {
    let mut linz_bucket = LinzBucket::initialise_catalog(bucket).await;
    if let Some(collection_name_filter) = &collection_name_filter_opt {
        linz_bucket.set_collection_filter(collection_name_filter);
    }
    if let Some(SpatialFilterParams {
        lat1: lat,
        lon1: lon,
        lat2_opt,
        lon2_opt,
        width_m_opt,
        height_m_opt,
    }) = spatial_params
    {
        if (height_m_opt.is_some() || width_m_opt.is_some())
            && (lat2_opt.is_some() || lon2_opt.is_some())
        {
            return Err(MyError::DimensionAndCoordinateRange);
        }

        let tiles = if let Some(width_m) = width_m_opt {
            let height_m = height_m_opt.unwrap_or(width_m);
            linz_bucket
                .get_tiles_from_point_and_dimension(lat, lon, width_m, height_m)
                .await
        } else if let (Some(lat2), Some(lon2)) = (lat2_opt, lon2_opt) {
            linz_bucket
                .get_tiles_from_lat_lon_range(lat, lon, lat2, lon2)
                .await
        } else {
            linz_bucket.get_tiles_from_lat_lon(lat, lon).await
        };

        Ok(tiles)
        // Use lat1, lon1, lat2_opt, lon2_opt, width_m_opt, height_m_opt here
    } else if collection_name_filter_opt.is_some() {
        let tiles = linz_bucket.get_all_tiles().await;
        Ok(tiles)
    } else {
        Err(MyError::NoFilterProvided)
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
