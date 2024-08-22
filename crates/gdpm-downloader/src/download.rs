//! Download module.

use reqwest::Url;
use tracing::info;

use crate::{error::DownloadError, DownloadAdapter};
use gdpm_types::version::{GodotVersion, SystemVersion};

/// Downloader.
pub struct Downloader;

impl Downloader {
    /// Get editor URL for version from a mirror URL.
    pub fn get_official_editor_url_for_version(
        version: GodotVersion,
        system: SystemVersion,
        mirror_url: &str,
    ) -> String {
        let kind = version.kind().clone();
        let mut route = String::new();
        let path = Url::parse(mirror_url).unwrap();

        // Get version path
        route.push_str(version.version());
        route.push('-');
        route.push_str(&kind.to_string());

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

    /// Get export templates URL for version from a mirror URL.
    pub fn get_official_export_templates_url_for_version(
        version: GodotVersion,
        mirror_url: &str,
    ) -> String {
        let kind = version.kind().clone();
        let mut route = String::new();
        let path = Url::parse(mirror_url).unwrap();

        // Get version path
        route.push_str(version.version());
        route.push('-');
        route.push_str(&kind.to_string());

        // Get name
        let filename = format!(
            "Godot_v{}-{}{}_export_templates.tpz",
            version.version(),
            kind,
            if version.mono() { "_mono" } else { "" },
        );
        route.push('/');
        route.push_str(&filename);

        path.join(&route).unwrap().to_string()
    }

    /// Download file at URL.
    pub async fn download_file_at_url<I: DownloadAdapter>(
        download_adapter: &I,
        url: &str,
    ) -> Result<Vec<u8>, DownloadError> {
        info!(url = url, "Will download file at url");
        download_adapter.download_file_at_url(url).await
    }
}

#[cfg(test)]
mod tests {
    use crate::download::Downloader;
    use gdpm_types::version::{GodotVersion, GodotVersionKind, SystemVersion};

    #[test]
    fn test_get_official_editor_url_for_version() {
        let root = "http://localhost/subdir/";
        let check = |(v, k, s, m), expected| {
            assert_eq!(
                Downloader::get_official_editor_url_for_version(
                    GodotVersion::new(v, k, m),
                    s,
                    root
                ),
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
            "http://localhost/subdir/1.2.3-stable/Godot_v1.2.3-stable_win64.exe.zip",
        );

        check(
            (
                "1.2.3",
                GodotVersionKind::Beta(1),
                SystemVersion::Win32,
                false,
            ),
            "http://localhost/subdir/1.2.3-beta1/Godot_v1.2.3-beta1_win32.exe.zip",
        );

        check(
            (
                "1.2.3",
                GodotVersionKind::ReleaseCandidate(2),
                SystemVersion::LinuxServer64,
                false,
            ),
            "http://localhost/subdir/1.2.3-rc2/Godot_v1.2.3-rc2_linux_server.64.zip",
        );

        check(
            (
                "1.2.3",
                GodotVersionKind::ReleaseCandidate(2),
                SystemVersion::LinuxServer64,
                true,
            ),
            "http://localhost/subdir/1.2.3-rc2/Godot_v1.2.3-rc2_mono_linux_server_64.zip",
        );
    }

    #[test]
    fn test_get_official_export_templates_url_for_version() {
        let root = "http://localhost/subdir/";
        let check = |(v, k, m), expected| {
            assert_eq!(
                Downloader::get_official_export_templates_url_for_version(
                    GodotVersion::new(v, k, m),
                    root
                ),
                expected
            );
        };

        check(
            ("1.2.3", GodotVersionKind::Stable, false),
            "http://localhost/subdir/1.2.3-stable/Godot_v1.2.3-stable_export_templates.tpz",
        );

        check(
            ("1.2.3", GodotVersionKind::Beta(1), false),
            "http://localhost/subdir/1.2.3-beta1/Godot_v1.2.3-beta1_export_templates.tpz",
        );

        check(
            ("1.2.3", GodotVersionKind::ReleaseCandidate(2), false),
            "http://localhost/subdir/1.2.3-rc2/Godot_v1.2.3-rc2_export_templates.tpz",
        );

        check(
            ("1.2.3", GodotVersionKind::ReleaseCandidate(2), true),
            "http://localhost/subdir/1.2.3-rc2/Godot_v1.2.3-rc2_mono_export_templates.tpz",
        );
    }
}
