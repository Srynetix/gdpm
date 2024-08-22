use crate::{error::DownloadError, DownloadAdapter};
use async_trait::async_trait;
use tracing::info;

use std::io::Write;

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Response, StatusCode};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// Default download adapter.
pub struct DefaultDownloadAdapter;

impl DefaultDownloadAdapter {
    async fn download_file_at_url_async(url: &str) -> Result<Vec<u8>, DownloadError> {
        let client = Client::new();
        let res = client
            .get(url)
            .send()
            .await
            .map_err(|e| DownloadError::ReqwestError(url.into(), e))?;

        match res.status() {
            StatusCode::OK => Self::download_file_inner(url, res).await,
            StatusCode::NOT_FOUND => Err(DownloadError::NotFound(url.to_owned())),
            e => Err(DownloadError::UnexpectedStatusCode(e)),
        }
    }

    async fn download_file_inner(url: &str, res: Response) -> Result<Vec<u8>, DownloadError> {
        info!(url = url, status = ?res.status(), "File found");

        let total_size = res.content_length().unwrap();

        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
            .progress_chars("#>-")
        );
        pb.set_message(format!("Downloading {}", url));

        let mut data: Vec<u8> = Vec::with_capacity(total_size as usize);
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| DownloadError::ReqwestError(url.into(), e))?;
            data.write_all(&chunk).map_err(DownloadError::IoError)?;

            let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message(format!("Downloaded {}", url));
        Ok(data)
    }
}

#[async_trait]
impl DownloadAdapter for DefaultDownloadAdapter {
    async fn download_file_at_url(&self, url: &str) -> Result<Vec<u8>, DownloadError> {
        Self::download_file_at_url_async(url).await
    }

    async fn lookup_remote_versions(&self) -> Result<Vec<String>, DownloadError> {
        #[derive(serde::Deserialize, Debug)]
        struct ApiRelease {
            name: String,
        }

        let response = Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .unwrap()
            .get("https://api.github.com/repos/godotengine/godot-builds/releases")
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();

        let releases: Vec<ApiRelease> = response.json().await.unwrap();

        let mut releases: Vec<_> = releases
            .into_iter()
            .map(|r| r.name.replace("-stable", "").replace('-', "."))
            .collect();

        releases.sort();
        Ok(releases)
    }
}
