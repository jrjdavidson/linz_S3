mod gdal;
mod reporter;
use futures::future::join_all;
use reporter::Reporter;
use stac::{Assets, Catalog, Collection, Href, Item, Links, SelfHref};
use std::{sync::Arc, thread, time::Duration};
use tokio::sync::{mpsc, Semaphore};

pub fn build_vrt_from_paths(tiles: Vec<String>) {
    gdal::build_vrt_from_paths(tiles, "test.vrt");
}
pub struct LinzBucket {
    collections: Vec<Collection>,
    reporter: Arc<Reporter>,
}

struct MatchingItems {
    title: String,
    items: Vec<Item>,
}

impl LinzBucket {
    pub async fn initialise_catalog(dataset: &str) -> Self {
        println!("Initialising Catalog...");
        let catalog_url = format!(
            "https://{}.s3.ap-southeast-2.amazonaws.com/catalog.json",
            dataset
        );

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
        let urls: Vec<String> = catalog
            .links()
            .iter()
            .filter_map(|link| {
                if link.is_child() {
                    if let Href::Url(url) = &link.href {
                        Some(url.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        let (tx, mut rx) = mpsc::channel(100);

        for url in urls {
            let tx = tx.clone();
            tokio::spawn(async move {
                match stac::io::get_opts::<stac::Collection, _, _, _>(
                    url,
                    [("skip_signature", "true"), ("region", "ap-southeast-2")],
                )
                .await
                {
                    Ok(mut collection) => {
                        collection.make_links_absolute().unwrap();
                        tx.send(Some(collection)).await.unwrap();
                    }
                    Err(e) => {
                        println!("Error fetching child item: {}", e);
                        tx.send(None).await.unwrap();
                    }
                }
            });
        }

        drop(tx); // Close the sender channel
        let mut collections = Vec::new();

        while let Some(collection) = rx.recv().await {
            if let Some(collection) = collection {
                collections.push(collection);
            }
        }

        let collections_total = collections.len();
        println!("Number of Collections in catalog: {}", collections_total);

        let bucket = LinzBucket {
            collections,
            reporter: Arc::new(Reporter::new(collections_total).await),
        };
        bucket.start_reporting();
        bucket
    }

    fn start_reporting(&self) {
        let reporter = Arc::clone(&self.reporter);
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                loop {
                    interval.tick().await;
                    reporter.report().await;
                }
            });
        });
    }

    pub async fn get_tiles_from_lat_lon(&self, lat: f64, lon: f64) -> Vec<(Vec<String>, String)> {
        self.get_tiles(lat, lon, None, None).await
    }

    pub async fn get_tiles_from_lat_lon_range(
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
        self.reporter.reset_all().await;

        let semaphore = Arc::new(Semaphore::new(3)); // Limit to 10 concurrent threads
        let futures: Vec<_> = self
            .collections
            .iter()
            .map(|collection| {
                let collection = collection.clone();
                let reporter = Arc::clone(&self.reporter);
                let semaphore = Arc::clone(&semaphore);

                tokio::spawn(async move {
                    let _permit = semaphore.acquire_owned().await.unwrap(); // Await the
                    process_collection(collection, lon1, lat1, lon2, lat2, reporter).await
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
        for item in result.items {
            let assets = item.assets();
            for value in assets.values() {
                let asset_href = value.href.to_string();
                let absolute_path = if asset_href.starts_with("./") {
                    let href = item.self_href().unwrap().to_string();
                    let base_path = href.rsplit_once('/').map(|x| x.0).unwrap_or("");
                    format!("{}/{}", base_path, asset_href.strip_prefix("./").unwrap())
                } else {
                    asset_href
                };
                items.push(absolute_path); // Push the absolute path instead of the original href
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
async fn process_collection(
    collection: Collection,
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
            return collection_extent_overlaps(
                collection, lon_min, lat_min, lon_max, lat_max, reporter,
            )
            .await;
        }
    }
    reporter.report_finished_collection().await;
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

async fn collection_extent_overlaps(
    collection: Collection,
    lon_min: f64,
    lat_min: f64,
    lon_max: f64,
    lat_max: f64,
    reporter: Arc<Reporter>,
) -> Option<MatchingItems> {
    let mut matching_items = vec![];
    let (tx, mut rx) = mpsc::channel(100);
    let title = collection.title.clone().unwrap_or_default();
    // let mut urls = vec![];
    // for link in collection.links() {
    //     if link.is_item() {
    //         if let Href::Url(url) = &link.href {
    //             let parsed_url = url.to_string();
    //             urls.push(parsed_url)
    //         }
    //     }
    // }
    let urls: Vec<String> = collection
        .links()
        .iter()
        .filter_map(|link| {
            if link.is_item() {
                if let Href::Url(url) = &link.href {
                    Some(url.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    reporter.add_urls(urls.len() as u64).await;
    for url in urls {
        let tx = tx.clone();
        let reporter = Arc::clone(&reporter);
        tokio::spawn(async move {
            match stac::io::get_opts::<stac::Item, _, _, _>(
                url,
                [("skip_signature", "true"), ("region", "ap-southeast-2")],
            )
            .await
            {
                Ok(item) => {
                    reporter.report_finished_url().await;

                    let bounding_box = item.bbox.unwrap();

                    if bounding_box.ymin() <= lat_max
                        && bounding_box.ymax() >= lat_min
                        && bounding_box.xmin() <= lon_max
                        && bounding_box.xmax() >= lon_min
                    {
                        tx.send(Some(item)).await.unwrap();
                    }
                }
                Err(e) => {
                    reporter.report_finished_url().await;

                    println!("Error fetching child item: {}", e);
                    tx.send(None).await.unwrap();
                }
            }
        });
    }
    drop(tx); // Close the sender channel

    while let Some(item) = rx.recv().await {
        if let Some(item) = item {
            matching_items.push(item);
        }
    }

    reporter.report_finished_collection().await;
    if !matching_items.is_empty() {
        return Some(MatchingItems {
            title,
            items: matching_items,
        });
    }
    None
}
