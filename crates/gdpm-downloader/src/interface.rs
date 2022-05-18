use crate::error::DownloadError;

/// Download adapter.
pub trait DownloadAdapter {
    /// Download file at URL.
    fn download_file_at_url(&self, url: &str) -> Result<Vec<u8>, DownloadError>;
}
