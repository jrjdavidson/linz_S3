use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use tokio::fs;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum TempFileWaitError {
    #[error("Temp file stuck for timeout period")]
    StaleTempFile(PathBuf),
    #[error("Failed to remove temp file: {0}")]
    RemoveFailed(String),
    #[error("Failed to get file metadata: {0}")]
    MetadataFailed(String),
}

pub async fn wait_for_temp_file(
    temp_path: &PathBuf,
    url: &str,
    max_waits: usize,
    interval: Duration,
) -> Result<(), TempFileWaitError> {
    let mut wait_count = 0;
    let mut last_size = None;
    let mut unchanged_count = 0;
    while temp_path.exists() {
        let size = match fs::metadata(temp_path).await {
            Ok(meta) => meta.len(),
            Err(e) => return Err(TempFileWaitError::MetadataFailed(e.to_string())),
        };
        if let Some(expected) = get_expected_size(url).await {
            if size == expected {
                // File is complete, rename or return
                return Ok(());
            }
        }
        if last_size == Some(size) {
            unchanged_count += 1;
        } else {
            unchanged_count = 0;
        }
        last_size = Some(size);
        if wait_count >= max_waits || unchanged_count > 10 {
            match fs::remove_file(temp_path).await {
                Ok(_) => return Err(TempFileWaitError::StaleTempFile(temp_path.clone())),
                Err(e) => return Err(TempFileWaitError::RemoveFailed(e.to_string())),
            }
        }
        sleep(interval).await;
        wait_count += 1;
    }
    Ok(())
}

use reqwest::Client;

pub async fn get_expected_size(url: &str) -> Option<u64> {
    let client = Client::new();
    match client
        .head(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok()),
        Err(_) => None,
    }
}
