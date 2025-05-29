use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use linz_s3::linz_s3_filter::{dataset, linz_bucket::LinzBucket};

use tokio::runtime::Runtime;
// Here we have an async function to benchmark
async fn empty_collections() {
    let dataset = dataset::BucketName::Imagery;
    let linz_bucket = LinzBucket::initialise_catalog(dataset).await;
    let lat = 40.9006;
    let lon = 174.8860;
    let _tiles = linz_bucket
        .unwrap()
        .get_tiles(Some(lat), Some(lon), None, None)
        .await;
}
fn bench_las_s3_filter_empty(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap(); // Create the runtime

    c.bench_function("empty collections", |b| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.

        b.to_async(&runtime).iter(empty_collections);
    });
}

criterion_group! {
  name = benches;
  config = Criterion::default().measurement_time(Duration::from_secs(120)).sample_size(20);
  targets = bench_las_s3_filter_empty
}
criterion_main!(benches);
