use crate::linz_s3_filter::dataset::LinzBucketName;
use crate::linz_s3_filter::reporter::Reporter;
use crate::linz_s3_filter::utils::{get_hrefs, process_collection};
use futures::future::join_all;
use log::info;
use stac::{Catalog, Collection, Href, Links};
use std::sync::{atomic::Ordering, Arc};
use tokio::sync::{mpsc, Semaphore};
use tokio::time::{self, Duration};

use crate::linz_s3_filter::bucket_config::{
    self, CONCURRENCY_LIMIT_COLLECTIONS, CONCURRENCY_LIMIT_CPU_MULTIPLIER,
};

pub struct LinzBucket {
    pub collections: Vec<Collection>,
    pub filtered_collections: Option<Vec<Collection>>,
    pub reporter: Reporter, // Use Mutex for interior mutability
}

impl LinzBucket {
    pub async fn initialise_catalog(dataset: LinzBucketName) -> Result<Self, stac::Error> {
        info!("Initialising Catalog...");
        let catalog_url = format!(
            "https://{}.s3.ap-southeast-2.amazonaws.com/catalog.json",
            dataset.as_str()
        );

        let options: Vec<(&'static str, String)> = bucket_config::get_opts();

        let mut catalog: Catalog = stac::io::get_opts(catalog_url, options).await?;

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

        let semaphore = Arc::new(Semaphore::new(CONCURRENCY_LIMIT_COLLECTIONS));
        let num_channels = urls.len();
        let (tx, mut rx) = mpsc::channel(num_channels);

        for url in urls {
            let tx: mpsc::Sender<Option<Collection>> = tx.clone();
            let semaphore = Arc::clone(&semaphore);
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap(); // Acquire a permit
                let options: Vec<(&'static str, String)> = bucket_config::get_opts();

                let collection_result: Result<Collection, stac::Error> =
                    stac::io::get_opts(url, options).await;
                match collection_result {
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
        info!(
            "Total number of Collections in catalog: {}",
            collections_total
        );

        let bucket = LinzBucket {
            collections,
            filtered_collections: None,
            reporter: Reporter::new(collections_total).await,
        };
        Ok(bucket)
    }

    fn start_reporting(&self, reporter: Arc<Reporter>) {
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            while !reporter.stop_flag.load(Ordering::Relaxed) {
                interval.tick().await;
                reporter.report().await;
            }
        });
    }

    pub async fn get_tiles(
        &mut self,
        lat1_opt: Option<f64>,
        lon1_opt: Option<f64>,
        lat2_opt: Option<f64>,
        lon2_opt: Option<f64>,
    ) -> Vec<(Vec<String>, String)> {
        let filtered_collections = self
            .filtered_collections
            .as_ref()
            .unwrap_or(&self.collections);
        self.reporter.reset_all(filtered_collections.len()).await;
        let reporter = Arc::new(self.reporter.clone());

        self.start_reporting(Arc::clone(&reporter));

        let num_cpus = num_cpus::get();
        let semaphore = Arc::new(Semaphore::new(num_cpus * CONCURRENCY_LIMIT_CPU_MULTIPLIER)); // Limit concurrent threads
        let futures: Vec<_> = filtered_collections
            .iter()
            .map(|collection| {
                let collection = collection.clone();
                let reporter = Arc::clone(&reporter);
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

    pub async fn get_all_tiles(&mut self) -> Vec<(Vec<String>, String)> {
        self.get_tiles(None, None, None, None).await
    }

    pub fn set_collection_filter(
        &mut self,
        collection_name_filters: Option<&[String]>,
        exclusion_filters: Option<&[String]>,
        extent: Option<(f64, f64, Option<f64>, Option<f64>)>,
    ) {
        let filtered_collections: Vec<_> = self
            .collections
            .iter()
            .filter(|collection| {
                let include = collection_name_filters.map_or(true, |filters| {
                    filters.is_empty()
                        || filters.iter().any(|filter| {
                            collection.id.contains(filter)
                                || collection.title.as_deref().unwrap_or("").contains(filter)
                        })
                });

                let exclude = exclusion_filters.is_some_and(|filters| {
                    filters.iter().any(|filter| {
                        collection.id.contains(filter)
                            || collection.title.as_deref().unwrap_or("").contains(filter)
                    })
                });

                let within_extent =
                    extent.map_or(true, |(min_lat, min_lon, max_lat_opt, max_lon_opt)| {
                        collection.extent.spatial.bbox.iter().any(|bbox| {
                            bbox.xmin() <= max_lon_opt.unwrap_or(min_lon)
                                && bbox.xmax() >= min_lon
                                && bbox.ymin() <= max_lat_opt.unwrap_or(min_lat)
                                && bbox.ymax() >= min_lat
                        })
                    });

                include && !exclude && within_extent
            })
            .cloned()
            .collect();
        self.filtered_collections = Some(filtered_collections);
    }
}
