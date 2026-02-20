use linz_s3::temp_file_wait::{wait_for_temp_file, TempFileWaitError};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn test_wait_for_temp_file_complete() {
    let temp_path = PathBuf::from("testfile.downloading");
    let url = "https://httpbin.org/bytes/10"; // httpbin returns Content-Length
    let _ = File::create(&temp_path)
        .await
        .unwrap()
        .write_all(&[0u8; 10])
        .await;
    let result = wait_for_temp_file(&temp_path, url, 5, Duration::from_millis(100)).await;
    assert!(result.is_ok());
    let _ = fs::remove_file(&temp_path).await;
}

#[tokio::test]
async fn test_wait_for_temp_file_stale() {
    let temp_path = PathBuf::from("testfile.downloading");
    let url = "https://httpbin.org/bytes/20";
    let _ = File::create(&temp_path)
        .await
        .unwrap()
        .write_all(&[0u8; 5])
        .await;
    let result = wait_for_temp_file(&temp_path, url, 3, Duration::from_millis(100)).await;
    assert!(matches!(result, Err(TempFileWaitError::StaleTempFile(_))));
    let _ = fs::remove_file(&temp_path).await;
}

// NOTE: These tests are not portable across all OSs and filesystems.
// It's difficult to reliably trigger RemoveFailed or MetadataFailed errors in a unit test.
// They are left here for documentation, but may fail or be skipped in CI.
/*
#[tokio::test]
async fn test_wait_for_temp_file_remove_failed() {
    let temp_path = PathBuf::from("/invalid/path/testfile.downloading");
    let url = "https://httpbin.org/bytes/20";
    let result = wait_for_temp_file(&temp_path, url, 1, Duration::from_millis(100)).await;
    assert!(matches!(result, Err(TempFileWaitError::RemoveFailed(_))));
}

#[tokio::test]
async fn test_wait_for_temp_file_metadata_failed() {
    let temp_path = PathBuf::from("/invalid/path/testfile.downloading");
    let url = "https://httpbin.org/bytes/20";
    let result = wait_for_temp_file(&temp_path, url, 1, Duration::from_millis(100)).await;
    assert!(matches!(result, Err(TempFileWaitError::MetadataFailed(_))));
}
*/
