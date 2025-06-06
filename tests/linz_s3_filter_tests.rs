use linz_s3::linz_s3_filter::{dataset, linz_bucket::LinzBucket, utils};
use serial_test::serial;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder().is_test(true).init();
    });
}
#[tokio::test]
#[serial]
async fn test_get_tiles_from_lat_lon_empty() {
    init_logger();

    let dataset = dataset::BucketName::Imagery;
    let linz_bucket = LinzBucket::initialise_catalog(dataset, Some(1)).await;
    let lat = 40.9006;
    let lon = 174.8860;
    let tiles = linz_bucket
        .unwrap()
        .get_tiles(Some(lat), Some(lon), None, None)
        .await;
    assert!(tiles.is_empty());
}
#[tokio::test]
#[serial]
async fn test_get_tiles_from_lat_lon() {
    init_logger();

    let dataset = dataset::BucketName::Elevation;
    let linz_bucket = LinzBucket::initialise_catalog(dataset, Some(1)).await;
    let lat = -45.0;
    let lon = 167.0;
    let tiles = linz_bucket
        .unwrap()
        .get_tiles(Some(lat), Some(lon), None, None)
        .await;
    assert!(!tiles.is_empty());
}

#[tokio::test]
#[serial]
async fn test_get_tiles_from_lat_lon_range() {
    init_logger();

    let dataset = dataset::BucketName::Elevation;
    let linz_bucket = LinzBucket::initialise_catalog(dataset, Some(1)).await;
    let lat1 = -45.9006;
    let lon1 = 170.8860;
    let lat2 = -45.2865;
    let lon2 = 175.7762;
    let tiles = linz_bucket
        .unwrap()
        .get_tiles(Some(lat1), Some(lon1), Some(lat2), Some(lon2))
        .await;
    assert!(!tiles.is_empty());
}

#[tokio::test]
#[serial]
async fn test_get_tiles_from_point_and_dimension() {
    init_logger();

    let dataset = dataset::BucketName::Elevation;
    let linz_bucket = LinzBucket::initialise_catalog(dataset, Some(1)).await;
    let lat = -45.0;
    let lon = 167.0;
    let width_m = 100000.0; // 100 km
    let height_m = 100000.0; // 100 km
    let (lat1, lon1, lat2, lon2) =
        utils::get_coordinate_from_dimension(lat, lon, width_m, height_m);
    let tiles = linz_bucket
        .unwrap()
        .get_tiles(Some(lat1), Some(lon1), Some(lat2), Some(lon2))
        .await;
    assert!(!tiles.is_empty());
}
#[tokio::test]
#[serial]
async fn test_get_tiles_from_point_and_dimension_filter() {
    init_logger();

    let dataset = dataset::BucketName::Elevation;
    let mut linz_bucket = LinzBucket::initialise_catalog(dataset, Some(1))
        .await
        .unwrap();
    linz_bucket.set_collection_filter(Some(&["Southland".to_string()]), None, None);
    let lat = -45.0;
    let lon = 167.0;
    let width_m = 100000.0; // 100 km
    let height_m = 100000.0; // 100 km
    let (lat1, lon1, lat2, lon2) =
        utils::get_coordinate_from_dimension(lat, lon, width_m, height_m);
    let tiles = linz_bucket
        .get_tiles(Some(lat1), Some(lon1), Some(lat2), Some(lon2))
        .await;
    assert!(!tiles.is_empty());
}
