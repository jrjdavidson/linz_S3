use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use linz_s3::linz_s3_filter::{dataset, linz_bucket::LinzBucket};
use log::info;
use tokio::runtime::Runtime;

async fn empty_collections(multiplier: &usize) {
    let dataset = dataset::BucketName::Imagery;
    let linz_bucket = LinzBucket::initialise_catalog(dataset, Some(*multiplier)).await;
    let lat = 40.9006;
    let lon = 174.8860;
    let _tiles = linz_bucket
        .unwrap()
        .get_tiles(Some(lat), Some(lon), None, None)
        .await;
}
async fn southland_collections(multiplier: &usize) {
    info!("Running empty_collections with multiplier: {}", multiplier);

    let dataset = dataset::BucketName::Elevation;
    let linz_bucket = LinzBucket::initialise_catalog(dataset, Some(*multiplier)).await;
    let lat = -45.;
    let lon = 170.;
    let _tiles = linz_bucket
        .unwrap()
        .get_tiles(Some(lat), Some(lon), None, None)
        .await;
}

fn bench_with_concurrency(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    let multipliers = [1, 2, 4, 8, 16, 32, usize::MAX]; // You can adjust these values
    let mut group = c.benchmark_group("Concurrency Benchmarks");
    // env_logger::builder().is_test(true).try_init().unwrap();

    for &multiplier in &multipliers {
        group.bench_with_input(
            BenchmarkId::new("empty_collections_concurrency", multiplier),
            &multiplier,
            |b, i| {
                b.to_async(&runtime).iter(|| empty_collections(i));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("southland_collections_concurrency", multiplier),
            &multiplier,
            |b, i| {
                b.to_async(&runtime).iter(|| southland_collections(i));
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(400)).sample_size(10);
    targets = bench_with_concurrency
}
criterion_main!(benches);
