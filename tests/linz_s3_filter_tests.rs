use linz_s3::linz_s3_filter::{Dataset, LinzBucket};

#[tokio::test]
async fn test_get_tiles_from_lat_lon_empty() {
    let dataset = Dataset::Elevation;
    let linz_bucket = LinzBucket::initialise_catalog(dataset).await;
    let lat = 40.9006;
    let lon = 174.8860;
    let tiles = linz_bucket.get_tiles_from_lat_lon(lat, lon).await;
    assert!(tiles.is_empty());
}
#[tokio::test]
async fn test_get_tiles_from_lat_lon() {
    let dataset = Dataset::Imagery;
    let linz_bucket = LinzBucket::initialise_catalog(dataset).await;
    let lat = -40.9006;
    let lon = 174.8860;
    let tiles = linz_bucket.get_tiles_from_lat_lon(lat, lon).await;
    assert!(!tiles.is_empty());
}

#[tokio::test]
async fn test_get_tiles_from_lat_lon_range() {
    let dataset = Dataset::Imagery;
    let linz_bucket = LinzBucket::initialise_catalog(dataset).await;
    let lat1 = -40.9006;
    let lon1 = 174.8860;
    let lat2 = -41.2865;
    let lon2 = 174.7762;
    let tiles = linz_bucket
        .get_tiles_from_lat_lon_range(lat1, lon1, lat2, lon2)
        .await;
    assert!(!tiles.is_empty());
}
