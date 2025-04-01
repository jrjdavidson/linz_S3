use log::info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
#[derive(Clone)]
pub struct Reporter {
    urls_read: Arc<Mutex<u64>>,
    urls_total: Arc<Mutex<u64>>,
    collections_read: Arc<Mutex<u64>>,
    pub collections_total: usize,
    pub stop_flag: Arc<AtomicBool>,
}

impl Reporter {
    pub async fn new(collections_total: usize) -> Self {
        Reporter {
            urls_read: Arc::new(Mutex::new(0)),
            urls_total: Arc::new(Mutex::new(0)),
            collections_read: Arc::new(Mutex::new(0)),
            collections_total,
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn report(&self) {
        let urls_read = self.urls_read.lock().await;
        let urls_total = self.urls_total.lock().await;
        let collections_read = self.collections_read.lock().await;
        if !self.stop_flag.load(Ordering::Relaxed) {
            info!(
                "Reporting: {}/{} Collections read, {}/{} URLS read",
                collections_read, self.collections_total, urls_read, urls_total
            );
        }
    }

    pub async fn report_finished_collection(&self) {
        let mut collections_read = self.collections_read.lock().await;
        *collections_read += 1;
    }
    pub async fn reset_collection_read(&self) {
        let mut collections_read = self.collections_read.lock().await;
        *collections_read = 0;
    }

    pub async fn add_urls(&self, count: u64) {
        let mut urls_total = self.urls_total.lock().await;
        *urls_total += count;
    }

    pub async fn report_finished_url(&self) {
        let mut urls_read = self.urls_read.lock().await;
        *urls_read += 1;
    }
    pub async fn reset_urls_read(&self) {
        let mut urls_read = self.urls_read.lock().await;
        *urls_read = 0;
    }
    pub async fn reset_urls_total(&self) {
        let mut urls_total = self.urls_total.lock().await;
        *urls_total = 0;
    }
    pub async fn reset_all(&mut self, collections_total: usize) {
        self.collections_total = collections_total;
        self.reset_collection_read().await;
        self.reset_urls_read().await;
        self.reset_urls_total().await;
        info!("Collections to be read: {}", self.collections_total,);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn test_new_reporter() {
        let collections_total = 5;
        let reporter = Reporter::new(collections_total).await;
        assert_eq!(*reporter.urls_read.lock().await, 0);
        assert_eq!(*reporter.urls_total.lock().await, 0);
        assert_eq!(*reporter.collections_read.lock().await, 0);
        assert_eq!(reporter.collections_total, collections_total);
        assert!(!reporter.stop_flag.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_report_finished_collection() {
        let collections_total = 5;
        let reporter = Reporter::new(collections_total).await;
        reporter.report_finished_collection().await;
        assert_eq!(*reporter.collections_read.lock().await, 1);
    }

    #[tokio::test]
    async fn test_reset_collection_read() {
        let collections_total = 5;
        let reporter = Reporter::new(collections_total).await;
        reporter.report_finished_collection().await;
        reporter.reset_collection_read().await;
        assert_eq!(*reporter.collections_read.lock().await, 0);
    }

    #[tokio::test]
    async fn test_add_urls() {
        let collections_total = 5;
        let reporter = Reporter::new(collections_total).await;
        reporter.add_urls(10).await;
        assert_eq!(*reporter.urls_total.lock().await, 10);
    }

    #[tokio::test]
    async fn test_report_finished_url() {
        let collections_total = 5;
        let reporter = Reporter::new(collections_total).await;
        reporter.add_urls(10).await;
        reporter.report_finished_url().await;
        assert_eq!(*reporter.urls_read.lock().await, 1);
    }

    #[tokio::test]
    async fn test_reset_urls_read() {
        let collections_total = 5;
        let reporter = Reporter::new(collections_total).await;
        reporter.add_urls(10).await;
        reporter.report_finished_url().await;
        reporter.reset_urls_read().await;
        assert_eq!(*reporter.urls_read.lock().await, 0);
    }

    #[tokio::test]
    async fn test_reset_urls_total() {
        let collections_total = 5;
        let reporter = Reporter::new(collections_total).await;
        reporter.add_urls(10).await;
        reporter.reset_urls_total().await;
        assert_eq!(*reporter.urls_total.lock().await, 0);
    }

    #[tokio::test]
    async fn test_reset_all() {
        let collections_total = 5;
        let mut reporter = Reporter::new(collections_total).await;
        reporter.add_urls(10).await;
        reporter.report_finished_url().await;
        reporter.report_finished_collection().await;
        reporter.reset_all(3).await;
        assert_eq!(*reporter.urls_read.lock().await, 0);
        assert_eq!(*reporter.urls_total.lock().await, 0);
        assert_eq!(*reporter.collections_read.lock().await, 0);
        assert_eq!(reporter.collections_total, 3);
    }
}
