pub mod bucket_config;
pub mod dataset;
pub mod linz_bucket;
pub mod reporter;
pub mod utils;
#[cfg(test)]
mod tests {
    use super::*;

    use crate::linz_s3_filter::linz_bucket::CollectionTaskContext;
    use reporter::Reporter;
    use stac::Item;
    use stac_io::parse_href;
    use std::sync::{Arc, Once};
    use utils::{extract_value_before_m, process_collection};
    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::builder().is_test(true).init();
        });
    }
    #[tokio::test]
    #[ignore = "Issue with local store"]
    async fn test_process_collection() {
        init_logger();
        use stac::Collection;
        let (store, path) = parse_href("tests/data/simple-item.json").unwrap();
        let item: Item = store.get(path).await.unwrap();

        let mut collection = Collection::new_from_item("an-id", "a description", &item);
        collection.title = Some("Test Collection".to_string());

        // Create a local object store

        let semaphore = Arc::new(tokio::sync::Semaphore::new(100));
        let reporter = Arc::new(Reporter::new(1));
        let result = process_collection(
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
        .await;

        assert!(result.is_some());
        let matching_items = result.unwrap();
        assert_eq!(matching_items.title, "Test Collection");
    }
    #[test]
    fn test_extract_value_before_m() {
        init_logger();
        let text = "100m elevation";
        let value1 = extract_value_before_m(text);
        assert_eq!(value1, 100.0);

        let text = "0.96m elevation";
        let value2 = extract_value_before_m(text);
        assert_eq!(value2, 0.96);
        let compare = value1.partial_cmp(&value2).unwrap();
        assert_eq!(compare, std::cmp::Ordering::Greater);
        let text = "no value";
        let value = extract_value_before_m(text);
        assert_eq!(value, f64::MAX);
    }
    #[tokio::test]
    async fn test_get_hrefs() {
        init_logger();
        use stac::Item;
        let item = Item::new("an-id");
        let items = vec![item];

        let results = vec![dataset::MatchingItems {
            title: "Test Collection".to_string(),
            items,
        }];

        let hrefs = utils::get_hrefs(results).await;
        assert_eq!(hrefs.len(), 1);
        assert_eq!(hrefs[0].1, "Test Collection");
    }
    #[tokio::test]
    async fn test_get_hrefs_sorting() {
        use stac::Item;
        // Create mock items
        let item1 = Item::new("id1");
        let item2 = Item::new("id2");
        let item3 = Item::new("id3");

        // Create mock MatchingItems
        let matching_items = vec![
            dataset::MatchingItems {
                title: "title 10m 2020".to_string(),
                items: vec![item1.clone()],
            },
            dataset::MatchingItems {
                title: "title 5m 2020".to_string(),
                items: vec![item2.clone()],
            },
            dataset::MatchingItems {
                title: "another title 10m 2020".to_string(),
                items: vec![item3.clone()],
            },
        ];

        // Call the function
        let hrefs = utils::get_hrefs(matching_items).await;

        // Verify the sorting order
        assert_eq!(hrefs.len(), 3);
        assert_eq!(hrefs[0].1, "title 5m 2020");
        assert_eq!(hrefs[1].1, "another title 10m 2020");
        assert_eq!(hrefs[2].1, "title 10m 2020");
    }
}
