use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use linz_s3::linz_s3_filter::{dataset, linz_bucket::LinzBucket};
use tokio::{runtime::Runtime, time::sleep};

async fn empty_collections(multiplier: &usize) {
    let dataset = dataset::BucketName::Imagery;
    let lat = 40.9006;
    let lon = 174.8860;
    get_tiles_from_lat_lon(lat, lon, dataset, multiplier).await;
}
async fn southland_collections(multiplier: &usize) {
    let dataset = dataset::BucketName::Elevation;
    let lat = -45.;
    let lon = 170.;
    get_tiles_from_lat_lon(lat, lon, dataset, multiplier).await;
}

async fn get_tiles_from_lat_lon(
    lat: f64,
    lon: f64,
    dataset: dataset::BucketName,
    multiplier: &usize,
) {
    let mut retries = 0;

    loop {
        match LinzBucket::initialise_catalog(dataset, Some(*multiplier)).await {
            Ok(mut linz_bucket) => {
                let _tiles = linz_bucket
                    .get_tiles(Some(lat), Some(lon), None, None)
                    .await;
                break;
            }
            Err(e) => {
                retries += 1;
                if retries >= 3 {
                    eprintln!("Failed after 3 retries: {:?}", e);
                    break;
                }
                eprintln!(
                    "Error initializing catalog: {:?}. Retrying in 1 minute...",
                    e
                );
                sleep(Duration::from_secs(60)).await;
            }
        }
    }
}

fn bench_with_concurrency(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    let multipliers = [1, 2, 4, 8, 16, 32, 1000]; // You can adjust these values
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
