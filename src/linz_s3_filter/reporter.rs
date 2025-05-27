use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

use log::info;

#[derive(Clone)]
pub struct Reporter {
    urls_read: Arc<AtomicUsize>,
    urls_total: Arc<AtomicUsize>,
    collections_read: Arc<AtomicUsize>,
    open_threads: Arc<AtomicUsize>,
    pub collections_total: usize,
    pub stop_flag: Arc<AtomicBool>,
}

impl Reporter {
    pub fn new(collections_total: usize) -> Self {
        Reporter {
            urls_read: Arc::new(AtomicUsize::new(0)),
            urls_total: Arc::new(AtomicUsize::new(0)),
            open_threads: Arc::new(AtomicUsize::new(0)),
            collections_read: Arc::new(AtomicUsize::new(0)),
            collections_total,
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn report(&self) {
        if !self.stop_flag.load(Ordering::Relaxed) {
            info!(
                "Reporting: {}/{} Collections read, {}/{} URLS read, Open threads:{}",
                self.collections_read.load(Ordering::Relaxed),
                self.collections_total,
                self.urls_read.load(Ordering::Relaxed),
                self.urls_total.load(Ordering::Relaxed),
                self.open_threads.load(Ordering::Relaxed)
            );
        }
    }

    pub fn report_finished_collection(&self) {
        self.collections_read.fetch_add(1, Ordering::Relaxed);
    }

    pub fn reset_collection_read(&self) {
        self.collections_read.store(0, Ordering::Relaxed);
    }

    pub fn add_thread(&self) {
        self.open_threads.fetch_add(1, Ordering::Relaxed);
    }

    pub fn report_finished_thread(&self) {
        self.open_threads.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn add_urls(&self, count: usize) {
        self.urls_total.fetch_add(count, Ordering::Relaxed);
    }

    pub fn report_finished_url(&self) {
        self.urls_read.fetch_add(1, Ordering::Relaxed);
    }

    pub fn reset_urls_read(&self) {
        self.urls_read.store(0, Ordering::Relaxed);
    }

    pub fn reset_urls_total(&self) {
        self.urls_total.store(0, Ordering::Relaxed);
    }

    pub fn reset_all(&mut self, collections_total: usize) {
        self.collections_total = collections_total;
        self.reset_collection_read();
        self.reset_urls_read();
        self.reset_urls_total();
        info!("Collections to be read: {}", self.collections_total);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_new_reporter() {
        let collections_total = 5;
        let reporter = Reporter::new(collections_total);
        assert_eq!(reporter.urls_read.load(Ordering::Relaxed), 0);
        assert_eq!(reporter.urls_total.load(Ordering::Relaxed), 0);
        assert_eq!(reporter.collections_read.load(Ordering::Relaxed), 0);
        assert_eq!(reporter.open_threads.load(Ordering::Relaxed), 0);
        assert_eq!(reporter.collections_total, collections_total);
        assert!(!reporter.stop_flag.load(Ordering::Relaxed));
    }

    #[test]
    fn test_report_finished_collection() {
        let reporter = Reporter::new(5);
        reporter.report_finished_collection();
        assert_eq!(reporter.collections_read.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_reset_collection_read() {
        let reporter = Reporter::new(5);
        reporter.report_finished_collection();
        reporter.reset_collection_read();
        assert_eq!(reporter.collections_read.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_add_urls() {
        let reporter = Reporter::new(5);
        reporter.add_urls(10);
        assert_eq!(reporter.urls_total.load(Ordering::Relaxed), 10);
    }

    #[test]
    fn test_report_finished_url() {
        let reporter = Reporter::new(5);
        reporter.add_urls(10);
        reporter.report_finished_url();
        assert_eq!(reporter.urls_read.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_reset_urls_read() {
        let reporter = Reporter::new(5);
        reporter.add_urls(10);
        reporter.report_finished_url();
        reporter.reset_urls_read();
        assert_eq!(reporter.urls_read.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_reset_urls_total() {
        let reporter = Reporter::new(5);
        reporter.add_urls(10);
        reporter.reset_urls_total();
        assert_eq!(reporter.urls_total.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_add_and_finish_thread() {
        let reporter = Reporter::new(5);
        reporter.add_thread();
        assert_eq!(reporter.open_threads.load(Ordering::Relaxed), 1);
        reporter.report_finished_thread();
        assert_eq!(reporter.open_threads.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_reset_all() {
        let mut reporter = Reporter::new(5);
        reporter.add_urls(10);
        reporter.report_finished_url();
        reporter.report_finished_collection();
        reporter.reset_all(3);
        assert_eq!(reporter.urls_read.load(Ordering::Relaxed), 0);
        assert_eq!(reporter.urls_total.load(Ordering::Relaxed), 0);
        assert_eq!(reporter.collections_read.load(Ordering::Relaxed), 0);
        assert_eq!(reporter.collections_total, 3);
    }
}
