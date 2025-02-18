use linz_s3::{build_vrt_from_paths, LinzBucket};

use tokio::runtime::Runtime;
fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let linz_bucket = LinzBucket::initialise_catalog("nz-elevation").await;
        let tiles = linz_bucket
            .get_tiles_from_lat_lon(-43.70752471, 170.14020944)
            .await;
        println!("Tiles: {:?}", tiles);
        build_vrt_from_paths(tiles[0].0.clone());
    });
}
