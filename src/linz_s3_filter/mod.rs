pub mod dataset;
pub mod linz_bucket;
mod reporter;
mod utils;
#[cfg(test)]
mod tests {
    use super::*;

    use reporter::Reporter;
    use std::sync::{Arc, Once};
    use utils::{extract_value_before_m, process_collection};

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::builder().is_test(true).init();
        });
    }
    #[tokio::test]
    async fn test_process_collection() {
        init_logger();
        use stac::Collection;
        let item = stac::read("tests/data/simple-item.json").unwrap();
        let mut collection = Collection::new_from_item("an-id", "a description", &item);
        collection.title = Some("Test Collection".to_string());

        let reporter = Arc::new(Reporter::new(1).await);
        let result =
            process_collection(collection, Some(172.93), Some(1.35), None, None, reporter).await;

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
}
