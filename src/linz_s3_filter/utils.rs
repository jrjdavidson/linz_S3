use crate::linz_s3_filter::dataset::MatchingItems;
use crate::linz_s3_filter::reporter::Reporter;
use log::info;
use regex::Regex;
use stac::{Assets, Collection, Href, Links, SelfHref};
use std::sync::Arc;
use tokio::sync::mpsc;

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
    hrefs_with_titles.sort_by(|a, b| {
        let a_key = extract_value_before_m(&a.1);
        let b_key = extract_value_before_m(&b.1);
        a_key.partial_cmp(&b_key).unwrap()
    });

    hrefs_with_titles.sort_by(|a, b| a.1.cmp(&b.1));

    hrefs_with_titles
}

pub async fn process_collection(
    collection: Collection,
    lon1_opt: Option<f64>,
    lat1_opt: Option<f64>,
    lon2_opt: Option<f64>,
    lat2_opt: Option<f64>,
    reporter: Arc<Reporter>,
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
                    collection, lon_min, lat_min, lon_max, lat_max, reporter,
                )
                .await;
            }
        }
        reporter.report_finished_collection().await;
        None
    } else {
        add_collection_without_filters(collection, reporter).await
    }
}

pub fn extract_value_before_m(text: &str) -> f64 {
    let re = Regex::new(r"(\d+(\.\d+)?)m\s+").unwrap();
    if let Some(caps) = re.captures(text) {
        caps[1].parse().unwrap_or(f64::MAX)
    } else {
        f64::MAX
    }
}

async fn add_collection_with_spatial_filter(
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
    let urls = extract_urls(&collection);

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

                    info!("Error fetching child item: {}", e);
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
pub async fn add_collection_without_filters(
    collection: Collection,
    reporter: Arc<Reporter>,
) -> Option<MatchingItems> {
    let mut matching_items = vec![];
    let (tx, mut rx) = mpsc::channel(100);
    let title = collection.title.clone().unwrap_or_default();
    let urls = extract_urls(&collection);

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
                    tx.send(Some(item)).await.unwrap();
                }
                Err(e) => {
                    reporter.report_finished_url().await;
                    info!("Error fetching child item: {}", e);
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
