use crate::linz_s3_filter::dataset::LinzBucketName;
use crate::linz_s3_filter::reporter::Reporter;
use crate::linz_s3_filter::utils::{get_hrefs, process_collection};
use futures::future::join_all;
use log::info;
use stac::{Catalog, Collection, Href, Links};
use std::sync::{atomic::Ordering, Arc};
use std::{thread, time::Duration};
use tokio::sync::{mpsc, Semaphore};

pub struct LinzBucket {
    pub collections: Vec<Collection>,
    pub reporter: Arc<Reporter>,
}

impl LinzBucket {
    pub async fn initialise_catalog(dataset: LinzBucketName) -> Self {
        info!("Initialising Catalog...");
        let catalog_url = format!(
            "https://{}.s3.ap-southeast-2.amazonaws.com/catalog.json",
            dataset.as_str()
        );

        let mut catalog = stac::io::get_opts::<Catalog, _, _, _>(
            catalog_url,
            [("skip_signature", "true"), ("region", "ap-southeast-2")],
        )
        .await
        .unwrap();
        info!("ID: {}", catalog.id);
        info!("Title: {}", catalog.title.as_deref().unwrap_or("N/A"));
        info!("Description: {}", catalog.description);
        catalog.make_links_absolute().unwrap();
        // Iterate through the links and fetch more details
        let urls: Vec<String> = catalog
            .links()
            .iter()
            .filter_map(|link| {
                if link.is_child() {
                    if let Href::Url(url) = &link.href {
                        Some(url.to_string())
                    } else if let Href::String(string) = &link.href {
                        Some(string.to_string())
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
                        info!("Error fetching child item: {}", e);
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
        info!("Number of Collections in catalog: {}", collections_total);

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
                while !reporter.stop_flag.load(Ordering::Relaxed) {
                    interval.tick().await;
                    reporter.report().await;
                }
            });
        });
    }

    pub async fn get_tiles_from_lat_lon(&self, lat: f64, lon: f64) -> Vec<(Vec<String>, String)> {
        self.get_tiles(Some(lat), Some(lon), None, None).await
    }

    pub async fn get_tiles_from_lat_lon_range(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> Vec<(Vec<String>, String)> {
        self.get_tiles(Some(lat1), Some(lon1), Some(lat2), Some(lon2))
            .await
    }

    pub async fn get_tiles_from_point_and_dimension(
        &self,
        lat: f64,
        lon: f64,
        width_m: f64,
        height_m: f64,
    ) -> Vec<(Vec<String>, String)> {
        // Convert meters to degrees
        let (lat1, lon1, lat2, lon2) = crate::linz_s3_filter::utils::get_coordinate_from_dimension(
            lat, lon, width_m, height_m,
        );

        self.get_tiles(Some(lat1), Some(lon1), Some(lat2), Some(lon2))
            .await
    }

    async fn get_tiles(
        &self,
        lat1_opt: Option<f64>,
        lon1_opt: Option<f64>,
        lat2_opt: Option<f64>,
        lon2_opt: Option<f64>,
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
                    process_collection(collection, lon1_opt, lat1_opt, lon2_opt, lat2_opt, reporter)
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
        self.reporter.stop_flag.store(true, Ordering::Relaxed);
        info!("All collections processed");

        get_hrefs(results).await
    }

    pub async fn get_all_tiles(&self) -> Vec<(Vec<String>, String)> {
        self.get_tiles(None, None, None, None).await
    }

    pub fn set_collection_filter(&mut self, collection_name_filters: &[String]) {
        self.collections.retain(|collection| {
            collection_name_filters.iter().any(|filter| {
                collection.id.contains(filter)
                    || collection.title.as_deref().unwrap_or("").contains(filter)
            })
        });
    }
}
