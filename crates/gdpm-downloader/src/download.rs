//! Download module.

use std::io::Write;

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Response, StatusCode, Url};

use crate::{
    error::DownloadError,
    version::{GodotVersion, GodotVersionKind, SystemVersion},
};

/// Downloader.
pub struct Downloader;

impl Downloader {
    /// Get URL for version from a mirror URL.
    pub fn get_official_url_for_version(
        version: GodotVersion,
        system: SystemVersion,
        mirror_url: &str,
    ) -> String {
        let kind = version.kind().clone();
        let mut route = String::new();
        let path = Url::parse(mirror_url).unwrap();

        // Get version path
        route.push_str(version.version());

        // Get special version
        if let GodotVersionKind::Beta(_)
        | GodotVersionKind::ReleaseCandidate(_)
        | GodotVersionKind::Alpha(_) = kind
        {
            route.push('/');
            route.push_str(&kind.to_string());
        }

        // Get mono version
        if version.mono() {
            route.push('/');
            route.push_str("mono");
        }

        // Get name
        let filename = format!(
            "Godot_v{}-{}_{}.zip",
            version.version(),
            kind,
            system.get_archive_basename(version.mono())
        );
        route.push('/');
        route.push_str(&filename);

        path.join(&route).unwrap().to_string()
    }

    /// Download file at URL.
    pub fn download_file_at_url(url: &str) -> Result<Vec<u8>, DownloadError> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(DownloadError::AsyncRuntimeError)?;
        rt.block_on(Self::download_file_at_url_async(url))
    }

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
        let total_size = res.content_length().unwrap();

        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
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

#[cfg(test)]
mod tests {
    use crate::{
        download::Downloader,
        version::{GodotVersion, GodotVersionKind, SystemVersion},
    };

    #[test]
    fn test_get_download_url_for_version() {
        let root = "http://localhost/subdir/";
        let check = |(v, k, s, m), expected| {
            assert_eq!(
                Downloader::get_official_url_for_version(GodotVersion::new(v, k, m), s, root),
                expected
            );
        };

        check(
            (
                "1.2.3",
                GodotVersionKind::Stable,
                SystemVersion::Win64,
                false,
            ),
            "http://localhost/subdir/1.2.3/Godot_v1.2.3-stable_win64.exe.zip",
        );

        check(
            (
                "1.2.3",
                GodotVersionKind::Beta(1),
                SystemVersion::Win32,
                false,
            ),
            "http://localhost/subdir/1.2.3/beta1/Godot_v1.2.3-beta1_win32.exe.zip",
        );

        check(
            (
                "1.2.3",
                GodotVersionKind::ReleaseCandidate(2),
                SystemVersion::LinuxServer64,
                false,
            ),
            "http://localhost/subdir/1.2.3/rc2/Godot_v1.2.3-rc2_linux_server.64.zip",
        );

        check(
            (
                "1.2.3",
                GodotVersionKind::ReleaseCandidate(2),
                SystemVersion::LinuxServer64,
                true,
            ),
            "http://localhost/subdir/1.2.3/rc2/mono/Godot_v1.2.3-rc2_mono_linux_server_64.zip",
        );
    }
}
