use crate::error::MyError;
use crate::linz_s3_filter::dataset::BucketName;
use crate::linz_s3_filter::reporter::Reporter;
use crate::linz_s3_filter::utils::{get_hrefs, process_collection};
use log::{debug, info};
use stac::{Catalog, Collection, Href, Links};
use std::sync::{atomic::Ordering, Arc};
use tokio::sync::Semaphore;
use tokio::time::{self, Duration};

use crate::linz_s3_filter::bucket_config::{self, CONCURRENCY_LIMIT_CPU_MULTIPLIER};

pub struct LinzBucket {
    pub collections: Vec<Collection>,
    pub filtered_collections: Option<Vec<Collection>>,
    pub reporter: Reporter, // Use Mutex for interior mutability
}

impl LinzBucket {
    pub async fn initialise_catalog(dataset: BucketName) -> Result<Self, stac::Error> {
        info!("Initialising Catalog...");
        let catalog_url = format!("{}/catalog.json", dataset.as_str());
        let options = bucket_config::get_opts();

        let mut catalog: Catalog = stac::io::get_opts(catalog_url, options).await?;
        info!("ID: {}", catalog.id);
        info!("Title: {}", catalog.title.as_deref().unwrap_or("N/A"));
        info!("Description: {}", catalog.description);
        catalog.make_links_absolute().unwrap();

        let urls: Vec<String> = catalog
            .links()
            .iter()
            .filter_map(|link| {
                if link.is_child() {
                    match &link.href {
                        Href::Url(url) => Some(url.to_string()),
                        Href::String(s) => Some(s.to_string()),
                    }
                } else {
                    None
                }
            })
            .collect();

        let permits = num_cpus::get() * CONCURRENCY_LIMIT_CPU_MULTIPLIER;
        debug!("Number of permits: {}", permits);
        let semaphore = Arc::new(Semaphore::new(permits));

        let mut handles = Vec::with_capacity(urls.len());
        for url in urls {
            let semaphore = semaphore.clone();
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let options = bucket_config::get_opts();
                let result: Result<Collection, stac::Error> =
                    stac::io::get_opts(url, options).await;
                drop(_permit);
                match result {
                    Ok(mut collection) => {
                        collection.make_links_absolute().unwrap();
                        Some(collection)
                    }
                    Err(e) => {
                        MyError::from(e).report();
                        None
                    }
                }
            });
            handles.push(handle);
        }
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            results.push(handle.await);
        }

        let collections: Vec<_> = results
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect();

        let collections_total = collections.len();
        info!(
            "Total number of Collections in catalog: {}",
            collections_total
        );

        let bucket = LinzBucket {
            collections,
            filtered_collections: None,
            reporter: Reporter::new(collections_total),
        };

        Ok(bucket)
    }

    fn start_reporting(&self, reporter: Arc<Reporter>) {
        tokio::spawn(async move {
            reporter.add_thread();
            let mut interval = time::interval(Duration::from_secs(1));

            while !reporter.stop_flag.load(Ordering::Relaxed) {
                interval.tick().await;
                reporter.report();
            }
            reporter.report_finished_thread();
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
        self.reporter.reset_all(filtered_collections.len());
        let reporter = Arc::new(self.reporter.clone());

        self.start_reporting(Arc::clone(&reporter));

        let semaphore = Arc::new(Semaphore::new(
            num_cpus::get() * CONCURRENCY_LIMIT_CPU_MULTIPLIER,
        )); // Limit concurrent threads
        let mut handles = Vec::with_capacity(filtered_collections.len());
        for collection in filtered_collections {
            let collection = collection.clone();
            let reporter = reporter.clone();
            let semaphore = semaphore.clone();
            let handle = tokio::spawn(async move {
                process_collection(
                    collection, lon1_opt, lat1_opt, lon2_opt, lat2_opt, &reporter, semaphore,
                )
                .await
            });

            handles.push(handle);
        }
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            results.push(handle.await.unwrap_or_else(|e| {
                MyError::from(e).report();
                None
            }));
        }
        let results: Vec<_> = results.into_iter().flatten().collect();
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
                let include = collection_name_filters.is_none_or(|filters| {
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
                    extent.is_none_or(|(min_lat, min_lon, max_lat_opt, max_lon_opt)| {
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
