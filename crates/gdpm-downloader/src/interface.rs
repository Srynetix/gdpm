use crate::error::DownloadError;
use async_trait::async_trait;

/// Download adapter.
#[async_trait]
#[mockall::automock]
pub trait DownloadAdapter {
    /// Download file at URL.
    async fn download_file_at_url(&self, url: &str) -> Result<Vec<u8>, DownloadError>;
}
