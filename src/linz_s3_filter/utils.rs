use crate::error::MyError;

use super::bucket_config::{self};
use super::dataset::MatchingItems;
use super::reporter::Reporter;
use futures::future::join_all;
use log::debug;
use regex::Regex;
use stac::{Assets, Collection, Href, Links, SelfHref};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub fn get_coordinate_from_dimension(
    lat: f64,
    lon: f64,
    width_m: f64,
    height_m: f64,
) -> (f64, f64, f64, f64) {
    let lat_offset = height_m / 111_320.0;
    // Approx. meters per degree latitude
    let lon_offset = width_m / (111_320.0 * lat.to_radians().cos());
    // Approx. meters per degree longitude

    let lat1 = lat - lat_offset / 2.0;
    let lon1 = lon - lon_offset / 2.0;
    let lat2 = lat + lat_offset / 2.0;
    let lon2 = lon + lon_offset / 2.0;
    (lat1, lon1, lat2, lon2)
}

pub async fn get_hrefs(results: Vec<MatchingItems>) -> Vec<(Vec<String>, String)> {
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
    hrefs_with_titles.sort_by(|a, b| a.1.cmp(&b.1));

    hrefs_with_titles.sort_by(|a, b| {
        let a_key = extract_value_before_m(&a.1);
        let b_key = extract_value_before_m(&b.1);
        a_key.partial_cmp(&b_key).unwrap()
    });
    hrefs_with_titles
}

pub async fn process_collection(
    collection: Collection,
    lon1_opt: Option<f64>,
    lat1_opt: Option<f64>,
    lon2_opt: Option<f64>,
    lat2_opt: Option<f64>,
    reporter: &Arc<Reporter>,
    semaphore: Arc<Semaphore>,
) -> Option<MatchingItems> {
    if let (Some(lon1), Some(lat1)) = (lon1_opt, lat1_opt) {
        let (lon_min, lon_max, lat_min, lat_max) =
            if let (Some(lon2), Some(lat2)) = (lon2_opt, lat2_opt) {
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
                return add_collection_with_spatial_filter(
                    collection, lon_min, lat_min, lon_max, lat_max, reporter, semaphore,
                )
                .await;
            }
        }
        reporter.report_finished_collection();
        None
    } else {
        add_collection_without_filters(collection, reporter, semaphore).await
    }
}

pub fn extract_value_before_m(text: &str) -> f64 {
    let re = Regex::new(r"(\d+(\.\d+)?)m\s+").unwrap();
    if let Some(caps) = re.captures(text) {
        caps[1].parse().unwrap_or(f64::MAX)
    } else {
        debug!("No match found in: {:?}", text);
        f64::MAX
    }
}

pub async fn add_collection_with_spatial_filter(
    collection: Collection,
    lon_min: f64,
    lat_min: f64,
    lon_max: f64,
    lat_max: f64,
    reporter: &Arc<Reporter>,
    semaphore: Arc<Semaphore>,
) -> Option<MatchingItems> {
    let title = collection.title.clone().unwrap_or_default();
    let urls = extract_urls(&collection);

    reporter.add_urls(urls.len());

    let handles: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let reporter = Arc::clone(reporter);
            let semaphore = Arc::clone(&semaphore);
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                reporter.add_thread();
                debug!("Processing URL: {}", url);
                let options = bucket_config::get_opts();

                let result: Result<stac::Item, stac::Error> =
                    stac::io::get_opts(url, options).await;
                reporter.report_finished_url();
                reporter.report_finished_thread();

                match result {
                    Ok(item) => {
                        let matches = item.bbox.iter().any(|bbox| {
                            bbox.ymin() <= lat_max
                                && bbox.ymax() >= lat_min
                                && bbox.xmin() <= lon_max
                                && bbox.xmax() >= lon_min
                        });
                        if matches {
                            Some(item)
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        MyError::from(e).report();
                        None
                    }
                }
            })
        })
        .collect();

    let results = join_all(handles).await;
    let matching_items: Vec<_> = results
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect();

    reporter.report_finished_collection();
    debug!("Finished processing collection: {}", title);

    if !matching_items.is_empty() {
        Some(MatchingItems {
            title,
            items: matching_items,
        })
    } else {
        None
    }
}

pub async fn add_collection_without_filters(
    collection: Collection,
    reporter: &Arc<Reporter>,
    semaphore: Arc<Semaphore>,
) -> Option<MatchingItems> {
    let title = collection.title.clone().unwrap_or_default();
    let urls = extract_urls(&collection);

    reporter.add_urls(urls.len());

    let handles: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let reporter = Arc::clone(reporter);
            let semaphore = Arc::clone(&semaphore);
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                reporter.add_thread();
                debug!("Processing URL: {}", url);
                let options = bucket_config::get_opts();
                let result = stac::io::get_opts(url, options).await;
                reporter.report_finished_url();
                reporter.report_finished_thread();
                result.ok()
            })
        })
        .collect();

    let results = join_all(handles).await;
    let matching_items: Vec<_> = results
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect();

    reporter.report_finished_collection();

    if !matching_items.is_empty() {
        Some(MatchingItems {
            title,
            items: matching_items,
        })
    } else {
        None
    }
}

fn extract_urls(collection: &Collection) -> Vec<String> {
    collection
        .links()
        .iter()
        .filter_map(|link| {
            if link.is_item() {
                match &link.href {
                    Href::Url(url) => Some(url.to_string()),
                    Href::String(string) => Some(string.to_string()),
                }
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use stac::Item;

    use super::*;

    #[test]
    fn test_get_coordinate_from_dimension() {
        let (lat1, lon1, lat2, lon2) = get_coordinate_from_dimension(0.0, 0.0, 1000.0, 1000.0);
        assert!((lat1 - (-0.004491)).abs() < 1e-6);
        assert!((lon1 - (-0.004491)).abs() < 1e-6);
        assert!((lat2 - 0.004491).abs() < 1e-6);
        assert!((lon2 - 0.004491).abs() < 1e-6);
    }

    #[test]
    fn test_extract_value_before_m() {
        assert_eq!(extract_value_before_m("123.45m some text"), 123.45);
        assert_eq!(extract_value_before_m("no number before m"), f64::MAX);
    }
    #[tokio::test]
    async fn test_get_hrefs() {
        use crate::linz_s3_filter::dataset::MatchingItems;

        let item1: Item = stac::read("tests/data/simple-item.json").unwrap();

        let item2: Item = stac::read("tests/data/simple-item.json").unwrap();

        let matching_items = vec![
            MatchingItems {
                title: "10m title".to_string(),
                items: vec![item1.clone()],
            },
            MatchingItems {
                title: "5m title".to_string(),
                items: vec![item2.clone()],
            },
        ];

        let hrefs = get_hrefs(matching_items).await;
        assert_eq!(hrefs.len(), 2);
        assert_eq!(hrefs[0].1, "5m title");
        assert_eq!(hrefs[1].1, "10m title");
    }
    #[tokio::test]
    #[ignore = "Sets race condition while changing current directory"]
    async fn test_process_collection() {
        use crate::linz_s3_filter::reporter::Reporter;
        use std::env;
        use std::sync::Arc;
        let original_dir = env::current_dir().expect("Failed to get current directory");
        let new_dir = Path::new("tests/data/");
        env::set_current_dir(new_dir).expect("Failed to change directory");
        let collection: Collection = stac::read("collection.json").unwrap();

        let reporter = Arc::new(Reporter::new(1));
        let semaphore = Arc::new(tokio::sync::Semaphore::new(100));
        let result = process_collection(
            collection,
            Some(172.93),
            Some(1.35),
            None,
            None,
            &reporter,
            semaphore,
        )
        .await;
        assert!(result.is_some());
        env::set_current_dir(original_dir).expect("Failed to change directory");
    }
}
