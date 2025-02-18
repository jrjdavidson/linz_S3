use tokio::sync::Mutex;

pub struct Reporter {
    urls_read: Mutex<u64>,
    urls_total: Mutex<u64>,
    collections_read: Mutex<u64>,
    collections_total: usize,
}

impl Reporter {
    pub async fn new(collections_total: usize) -> Self {
        Reporter {
            urls_read: Mutex::new(0),
            urls_total: Mutex::new(0),
            collections_read: Mutex::new(0),
            collections_total,
        }
    }

    pub async fn report(&self) {
        let urls_read = self.urls_read.lock().await;
        let urls_total = self.urls_total.lock().await;
        let collections_read = self.collections_read.lock().await;
        println!(
            "Reporting: {}/{} Collections read, {}/{} URLS read",
            collections_read, self.collections_total, urls_read, urls_total
        );
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
    pub async fn reset_all(&self) {
        self.reset_collection_read().await;
        self.reset_urls_read().await;
        self.reset_urls_total().await;
    }
}
