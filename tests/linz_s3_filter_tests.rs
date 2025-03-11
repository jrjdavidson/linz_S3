use linz_s3::linz_s3_filter::{dataset, linz_bucket::LinzBucket};
use std::sync::Once;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder().is_test(true).init();
    });
}
#[tokio::test]
async fn test_get_tiles_from_lat_lon_empty() {
    init_logger();

    let dataset = dataset::LinzBucketName::Imagery;
    let linz_bucket = LinzBucket::initialise_catalog(dataset).await;
    let lat = 40.9006;
    let lon = 174.8860;
    let tiles = linz_bucket.get_tiles_from_lat_lon(lat, lon).await;
    assert!(tiles.is_empty());
}
#[tokio::test]
async fn test_get_tiles_from_lat_lon() {
    init_logger();

    let dataset = dataset::LinzBucketName::Elevation;
    let linz_bucket = LinzBucket::initialise_catalog(dataset).await;
    let lat = -45.0;
    let lon = 167.0;
    let tiles = linz_bucket.get_tiles_from_lat_lon(lat, lon).await;
    assert!(!tiles.is_empty());
}

#[tokio::test]
async fn test_get_tiles_from_lat_lon_range() {
    init_logger();

    let dataset = dataset::LinzBucketName::Elevation;
    let linz_bucket = LinzBucket::initialise_catalog(dataset).await;
    let lat1 = -45.9006;
    let lon1 = 170.8860;
    let lat2 = -45.2865;
    let lon2 = 175.7762;
    let tiles = linz_bucket
        .get_tiles_from_lat_lon_range(lat1, lon1, lat2, lon2)
        .await;
    assert!(!tiles.is_empty());
}

#[tokio::test]
async fn test_get_tiles_from_point_and_dimension() {
    init_logger();

    let dataset = dataset::LinzBucketName::Elevation;
    let linz_bucket = LinzBucket::initialise_catalog(dataset).await;
    let lat = -45.0;
    let lon = 167.0;
    let width_m = 100000.0; // 100 km
    let height_m = 100000.0; // 100 km
    let tiles = linz_bucket
        .get_tiles_from_point_and_dimension(lat, lon, width_m, height_m)
        .await;
    assert!(!tiles.is_empty());
}
#[tokio::test]
async fn test_get_tiles_from_point_and_dimension_filter() {
    init_logger();

    let dataset = dataset::LinzBucketName::Elevation;
    let mut linz_bucket = LinzBucket::initialise_catalog(dataset).await;
    linz_bucket.set_collection_filter(Some(&["Southland".to_string()]), None);
    let lat = -45.0;
    let lon = 167.0;
    let width_m = 100000.0; // 100 km
    let height_m = 100000.0; // 100 km
    let tiles = linz_bucket
        .get_tiles_from_point_and_dimension(lat, lon, width_m, height_m)
        .await;
    assert!(!tiles.is_empty());
}
