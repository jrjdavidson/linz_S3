use criterion::{criterion_group, criterion_main, Criterion};
use stac_io::parse_href;
use std::sync::Arc;
use tokio::runtime::Runtime;

use linz_s3::linz_s3_filter::reporter::Reporter;
use linz_s3::linz_s3_filter::utils;
use linz_s3::linz_s3_filter::{dataset, linz_bucket::CollectionTaskContext};

use stac::{Collection, Item}; // Replace `your_crate` with your actual crate name

#[allow(dead_code)]
fn bench_process_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("process_collection", |b| {
        b.to_async(&rt).iter(|| async {
            let (store, path) = parse_href("tests/data/simple-item.json").unwrap();
            let item: Item = store.get(path).await.unwrap();
            let mut collection = Collection::new_from_item("an-id", "a description", &item);
            collection.title = Some("Test Collection".to_string());

            let semaphore = Arc::new(tokio::sync::Semaphore::new(100));
            let reporter = Arc::new(Reporter::new(1));

            utils::process_collection(
                CollectionTaskContext {
                    collection,
                    store,
                    reporter,
                    semaphore,
                },
                Some(172.93),
                Some(1.35),
                None,
                None,
            )
            .await
        });
    });
}

fn bench_get_hrefs(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("get_hrefs", |b| {
        b.to_async(&rt).iter(|| async {
            let item = Item::new("an-id");
            let items = vec![item];

            let results = vec![dataset::MatchingItems {
                title: "Test Collection".to_string(),
                items,
            }];

            utils::get_hrefs(results).await
        });
    });
}

fn bench_extract_value_before_m(c: &mut Criterion) {
    c.bench_function("extract_value_before_m", |b| {
        b.iter(|| {
            let _ = utils::extract_value_before_m("100m elevation");
            let _ = utils::extract_value_before_m("0.96m elevation");
            let _ = utils::extract_value_before_m("no value");
        });
    });
}

criterion_group!(
    benches,
    // bench_process_collection,
    bench_get_hrefs,
    bench_extract_value_before_m
);
criterion_main!(benches);
