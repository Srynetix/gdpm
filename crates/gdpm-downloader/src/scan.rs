use async_recursion::async_recursion;
use soup::prelude::*;
use tracing::debug;

use crate::{error::DownloadError, DownloadAdapter};
use gdpm_types::version::{GodotVersion, GodotVersionKind};

/// Godot mirror scanner.
pub struct GodotMirrorScanner<'a, D: DownloadAdapter> {
    download_adapter: &'a D,
}

impl<'a, D: DownloadAdapter> GodotMirrorScanner<'a, D> {
    /// Creates a new mirror scanner.
    pub fn new(download_adapter: &'a D) -> Self {
        Self { download_adapter }
    }

    /// Scan versions.
    pub async fn scan(&self, mirror_url: &str) -> Result<Vec<GodotVersion>, DownloadError> {
        let versions = self.scan_version_names(mirror_url).await?;
        let mut output = Vec::new();

        for version in versions {
            for version in self
                .scan_kinds_for_version(mirror_url, &version, GodotVersionKind::Stable, false)
                .await?
            {
                output.push(version);
            }
        }

        Ok(output)
    }

    #[async_recursion(?Send)]
    async fn scan_kinds_for_version(
        &self,
        mirror_url: &str,
        version: &str,
        kind: GodotVersionKind,
        has_mono: bool,
    ) -> Result<Vec<GodotVersion>, DownloadError> {
        let suffix = match &kind {
            GodotVersionKind::Stable => String::new(),
            other => format!("{}/", other),
        };
        let target_url = if has_mono {
            format!("{mirror_url}{version}/{suffix}mono/")
        } else {
            format!("{mirror_url}{version}/{suffix}")
        };

        debug!("Scanning {target_url} ...");

        let content = self.download_adapter.get_url_contents(&target_url).await?;
        let soup = Soup::new(&content);
        let mut versions = Vec::new();
        let folders: Vec<_> = soup
            .tag("td")
            .class("n")
            .find_all()
            .filter_map(|n| {
                let folders: Vec<_> = n
                    .children()
                    .flat_map(|x| x.tag("a").find_all().collect::<Vec<_>>())
                    .filter_map(|x| {
                        if let Some(x) = x.get("href") {
                            // Yep, it's a folder
                            if x.ends_with('/') {
                                return Some(x.strip_suffix('/').unwrap().to_string());
                            }
                        }

                        None
                    })
                    .collect();

                if folders.is_empty() {
                    None
                } else {
                    Some(folders)
                }
            })
            .flatten()
            .collect();

        let files = soup
            .tag("td")
            .class("n")
            .find_all()
            .filter_map(|n| {
                let files: Vec<_> = n
                    .children()
                    .flat_map(|x| x.tag("a").find_all().collect::<Vec<_>>())
                    .filter_map(|x| {
                        if let Some(x) = x.get("href") {
                            // Yep, it's a file
                            if !x.ends_with('/') {
                                return Some(x);
                            }
                        }

                        None
                    })
                    .collect();

                if files.is_empty() {
                    None
                } else {
                    Some(files)
                }
            })
            .flatten();

        for folder in folders {
            if folder == "mono" {
                if has_mono {
                    panic!("Well a 'mono' folder should not be present in another 'mono' folder.");
                } else {
                    // Let's fetch mono versions
                    for godot_version in self
                        .scan_kinds_for_version(mirror_url, version, kind.clone(), true)
                        .await?
                    {
                        versions.push(godot_version);
                    }
                }
            } else if folder.starts_with("rc") {
                let rc_num: u16 = folder.strip_prefix("rc").unwrap().parse().unwrap_or(0);
                for godot_version in self
                    .scan_kinds_for_version(
                        mirror_url,
                        version,
                        GodotVersionKind::ReleaseCandidate(rc_num),
                        has_mono,
                    )
                    .await?
                {
                    versions.push(godot_version);
                }
            } else if folder.starts_with("beta") {
                let beta_num: u16 = folder.strip_prefix("beta").unwrap().parse().unwrap_or(0);
                for godot_version in self
                    .scan_kinds_for_version(
                        mirror_url,
                        version,
                        GodotVersionKind::Beta(beta_num),
                        has_mono,
                    )
                    .await?
                {
                    versions.push(godot_version);
                }
            } else if folder.starts_with("alpha") {
                let alpha_num: u16 = folder.strip_prefix("alpha").unwrap().parse().unwrap_or(0);
                for godot_version in self
                    .scan_kinds_for_version(
                        mirror_url,
                        version,
                        GodotVersionKind::Alpha(alpha_num),
                        has_mono,
                    )
                    .await?
                {
                    versions.push(godot_version);
                }
            }
        }

        if files.count() > 0 {
            // If there is something here, push!
            versions.push(GodotVersion::new(version, kind, has_mono));
        }

        Ok(versions)
    }

    async fn scan_version_names(&self, mirror_url: &str) -> Result<Vec<String>, DownloadError> {
        let content = self.download_adapter.get_url_contents(mirror_url).await?;
        let soup = Soup::new(&content);
        let versions = soup
            .tag("td")
            .class("n")
            .find_all()
            .filter_map(|n| {
                let versions: Vec<_> = n
                    .children()
                    .flat_map(|x| x.tag("a").find_all().collect::<Vec<_>>())
                    .filter_map(|x| {
                        if let Some(x) = x.get("href") {
                            let c = x.chars().next().unwrap();
                            if let Some(d) = c.to_digit(10) {
                                // Only fetch numbers > to 3
                                if d >= 3 {
                                    return Some(x.strip_suffix('/').unwrap().to_string());
                                }
                            }
                        }

                        None
                    })
                    .collect();

                if versions.is_empty() {
                    None
                } else {
                    Some(versions)
                }
            })
            .flatten()
            .collect();

        Ok(versions)
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use mockall::predicate;
    use pretty_assertions::assert_eq;

    use super::{GodotMirrorScanner, GodotVersion, GodotVersionKind};
    use crate::MockDownloadAdapter;

    #[tokio::test]
    async fn test_scan() {
        let mut adapter = MockDownloadAdapter::new();
        adapter
            .expect_get_url_contents()
            .with(predicate::eq("https://localhost/"))
            .times(1)
            .returning(|_| {
                async {
                    Ok(indoc::indoc! {r#"
                    <table>
                        <td class="n"><a href="3.0.0/"></a></td>
                    </table>
                "#}
                    .to_string())
                }
                .boxed()
            });

        adapter
            .expect_get_url_contents()
            .with(predicate::eq("https://localhost/3.0.0/"))
            .times(1)
            .returning(|_| {
                async {
                    Ok(indoc::indoc! {r#"
                    <table>
                        <td class="n"><a href="mono/"></a></td>
                        <td class="n"><a href="rc/"></a></td>
                        <td class="n"><a href="alpha1/"></a></td>
                        <td class="n"><a href="beta2/"></a></td>
                        <td class="n"><a href="sample"></a></td>
                    </table>
                "#}
                    .to_string())
                }
                .boxed()
            });

        adapter
            .expect_get_url_contents()
            .with(predicate::eq("https://localhost/3.0.0/mono/"))
            .times(1)
            .returning(|_| {
                async {
                    Ok(indoc::indoc! {r#"
                    <table>
                        <td class="n"><a href="sample"></a></td>
                    </table>
                "#}
                    .to_string())
                }
                .boxed()
            });

        adapter
            .expect_get_url_contents()
            .with(predicate::eq("https://localhost/3.0.0/rc/"))
            .times(1)
            .returning(|_| {
                async {
                    Ok(indoc::indoc! {r#"
                    <table>
                        <td class="n"><a href="sample"></a></td>
                    </table>
                "#}
                    .to_string())
                }
                .boxed()
            });

        adapter
            .expect_get_url_contents()
            .with(predicate::eq("https://localhost/3.0.0/alpha1/"))
            .times(1)
            .returning(|_| {
                async {
                    Ok(indoc::indoc! {r#"
                    <table>
                        <td class="n"><a href="mono/"></a></td>
                        <td class="n"><a href="sample"></a></td>
                    </table>
                "#}
                    .to_string())
                }
                .boxed()
            });

        adapter
            .expect_get_url_contents()
            .with(predicate::eq("https://localhost/3.0.0/alpha1/mono/"))
            .times(1)
            .returning(|_| {
                async {
                    Ok(indoc::indoc! {r#"
                    <table>
                        <td class="n"><a href="sample"></a></td>
                    </table>
                "#}
                    .to_string())
                }
                .boxed()
            });

        adapter
            .expect_get_url_contents()
            .with(predicate::eq("https://localhost/3.0.0/beta2/"))
            .times(1)
            .returning(|_| {
                async {
                    Ok(indoc::indoc! {r#"
                    <table>
                        <td class="n"><a href="sample"></a></td>
                    </table>
                "#}
                    .to_string())
                }
                .boxed()
            });

        let scanner = GodotMirrorScanner::new(&adapter);
        let versions = scanner.scan("https://localhost/").await.unwrap();
        assert_eq!(
            versions,
            vec![
                GodotVersion::new("3.0.0", GodotVersionKind::Stable, true),
                GodotVersion::new("3.0.0", GodotVersionKind::ReleaseCandidate(0), false),
                GodotVersion::new("3.0.0", GodotVersionKind::Alpha(1), true),
                GodotVersion::new("3.0.0", GodotVersionKind::Alpha(1), false),
                GodotVersion::new("3.0.0", GodotVersionKind::Beta(2), false),
                GodotVersion::new("3.0.0", GodotVersionKind::Stable, false),
            ]
        )
    }
}
