use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use futures::future::join_all;
use stac::{Assets, Catalog, Collection, Href, Item, Links};
use tokio::runtime::Runtime;

struct LinzBucket {
    collections: Vec<Collection>,
    reporter: Arc<Reporter>,
}

struct MatchingItems {
    title: String,
    items: Vec<Item>,
}

struct Reporter {
    urls_read: Mutex<u64>,
    urls_total: Mutex<u64>,
    collections_read: Mutex<u64>,
    collections_total: usize,
}

impl Reporter {
    fn new(collections_total: usize) -> Self {
        Reporter {
            urls_read: Mutex::new(0),
            urls_total: Mutex::new(0),
            collections_read: Mutex::new(0),
            collections_total,
        }
    }

    fn report(&self) {
        let urls_read = self.urls_read.lock().unwrap();
        let urls_total = self.urls_total.lock().unwrap();
        let collections_read = self.collections_read.lock().unwrap();
        println!(
            "Reporting: {}/{} Collections read, {}/{} URLS read",
            collections_read, self.collections_total, urls_read, urls_total
        );
    }

    fn report_finished_collection(&self) {
        let mut collections_read = self.collections_read.lock().unwrap();
        *collections_read += 1;
    }
}

impl LinzBucket {
    async fn initialise_catalog(dataset: &str) -> Self {
        println!("Initialising Catalog...");
        let catalog_url = format!("s3://{}/catalog.json", dataset);

        let mut catalog = stac::io::get_opts::<Catalog, _, _, _>(
            catalog_url,
            [("skip_signature", "true"), ("region", "ap-southeast-2")],
        )
        .await
        .unwrap();
        println!("ID: {}", catalog.id);
        println!("Title: {}", catalog.title.as_deref().unwrap_or("N/A"));
        println!("Description: {}", catalog.description);
        catalog.make_links_absolute().unwrap();
        // Iterate through the links and fetch more details
        let links = catalog.links();
        let mut collections = Vec::new();
        for link in links {
            if link.is_child() {
                if let Href::Url(url) = &link.href {
                    let parsed_url = url.as_str();
                    match stac::io::get_opts::<stac::Collection, _, _, _>(
                        parsed_url,
                        [("skip_signature", "true"), ("region", "ap-southeast-2")],
                    )
                    .await
                    {
                        Ok(mut collection) => {
                            collection.make_links_absolute().unwrap();

                            collections.push(collection);
                        }
                        Err(e) => {
                            println!("Error fetching child item: {}", e);
                        }
                    }
                }
            }
        }
        let collections_total = collections.len();
        println!("Number of Collections in catalog: {}", collections_total);

        LinzBucket {
            collections,
            reporter: Arc::new(Reporter::new(collections_total)),
        }
    }
    fn start_reporting(&self) {
        let reporter = Arc::clone(&self.reporter);
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                loop {
                    interval.tick().await;
                    reporter.report();
                }
            });
        });
    }

    async fn get_tiles_from_lat_lon(&self, lat: f64, lon: f64) -> Vec<(Vec<String>, String)> {
        self.get_tiles(lat, lon, None, None).await
    }

    async fn get_tiles_from_lat_lon_range(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> Vec<(Vec<String>, String)> {
        self.get_tiles(lat1, lon1, Some(lat2), Some(lon2)).await
    }
    async fn get_tiles(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: Option<f64>,
        lon2: Option<f64>,
    ) -> Vec<(Vec<String>, String)> {
        let futures: Vec<_> = self
            .collections
            .iter()
            .map(|collection| {
                let collection = collection.clone();
                let reporter = Arc::clone(&self.reporter);

                tokio::spawn(async move {
                    process_collection_with_range(&collection, lon1, lat1, lon2, lat2, reporter)
                        .await
                })
            })
            .collect();

        let results: Vec<_> = join_all(futures)
            .await
            .into_iter()
            .filter_map(|res| res.ok())
            .flatten()
            .collect();
        get_hrefs(results).await
    }
}

async fn get_hrefs(results: Vec<MatchingItems>) -> Vec<(Vec<String>, String)> {
    let mut hrefs_with_titles = Vec::new();
    for result in results {
        let mut items = vec![];
        for mut item in result.items {
            item.make_links_absolute().unwrap();
            let assets = item.assets();
            for value in assets.values() {
                let asset_href = value.href.clone();
                items.push(asset_href);
            }
        }
        hrefs_with_titles.push((items, result.title));
    }

    hrefs_with_titles.sort_by(|a, b| {
        let a_key = extract_value_before_m(&a.1);
        let b_key = extract_value_before_m(&b.1);
        a_key.cmp(&b_key)
    });

    hrefs_with_titles
}
async fn process_collection_with_range(
    collection: &Collection,
    lon1: f64,
    lat1: f64,
    lon2: Option<f64>,
    lat2: Option<f64>,
    reporter: Arc<Reporter>,
) -> Option<MatchingItems> {
    let (lon_min, lon_max, lat_min, lat_max) = if let (Some(lon2), Some(lat2)) = (lon2, lat2) {
        (
            lon1.min(lon2),
            lon1.max(lon2),
            lat1.min(lat2),
            lat1.max(lat2),
        )
    } else {
        (lon1, lon1, lat1, lat1)
    };

    for bbox in &collection.extent.spatial.bbox {
        if bbox.ymin() <= lat_max
            && bbox.ymax() >= lat_min
            && bbox.xmin() <= lon_max
            && bbox.xmax() >= lon_min
        {
            return collection_extent_overlaps_with_range(
                collection, lon_min, lat_min, lon_max, lat_max, reporter,
            )
            .await;
        }
    }
    reporter.report_finished_collection();
    println!("No items in collection: '{:?}' match", collection.title);
    None
}
fn extract_value_before_m(text: &str) -> i32 {
    let re = regex::Regex::new(r"(\d+(\.\d+)?)m\s+").unwrap();
    if let Some(caps) = re.captures(text) {
        caps[1].parse().unwrap_or(i32::MAX)
    } else {
        i32::MAX
    }
}

async fn collection_extent_overlaps_with_range(
    collection: &Collection,
    lon_min: f64,
    lat_min: f64,
    lon_max: f64,
    lat_max: f64,
    reporter: Arc<Reporter>,
) -> Option<MatchingItems> {
    let mut matching_items = vec![];
    for link in collection.links() {
        if link.is_item() {
            if let Href::Url(url) = &link.href {
                let parsed_url = url.as_str();
                match stac::io::get_opts::<stac::Item, _, _, _>(
                    parsed_url,
                    [("skip_signature", "true"), ("region", "ap-southeast-2")],
                )
                .await
                {
                    Ok(item) => {
                        let bounding_box = item.bbox.unwrap();
                        let geo = item.geometry.unwrap();

                        if bounding_box.ymin() <= lat_max
                            && bounding_box.ymax() >= lat_min
                            && bounding_box.xmin() <= lon_max
                            && bounding_box.xmax() >= lon_min
                        {
                            println!("Item {} from '{:?}' matches", item.id, collection.title);
                            matching_items.push(item);
                            if bounding_box.ymin() == bounding_box.ymax()
                                && bounding_box.xmin() == bounding_box.xmax()
                            {
                                break;
                            };
                        }
                    }
                    Err(e) => {
                        println!("Error fetching child item: {}", e);
                    }
                }
            }
        }
    }

    reporter.report_finished_collection();
    if !matching_items.is_empty() {
        return Some(MatchingItems {
            title: collection.title.clone().unwrap_or_default(),
            items: matching_items,
        });
    }
    None
}
fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let linz_bucket = LinzBucket::initialise_catalog("nz-elevation").await;
        linz_bucket.start_reporting();
        let tiles = linz_bucket.get_tiles_from_lat_lon(-43.5321, 172.6362).await;
        println!("Tiles: {:?}", tiles);
        let tiles = linz_bucket
            .get_tiles_from_lat_lon_range(-43.5321, 172.6362, -43.33, 172.30)
            .await;
        println!("Tiles: {:?}", tiles);
    });
}
