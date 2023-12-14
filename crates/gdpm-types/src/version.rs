//! Versions.

use std::{fmt::Display, str::FromStr};

use gdsettings_parser::GdValue;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wrong version: {0}")]
    WrongVersion(String),

    #[error("Wrong version kind: {0}")]
    WrongVersionKind(String),
}

/// System version.
#[derive(Debug, Clone)]
pub enum SystemVersion {
    /// Windows 32-bit
    Win32,
    /// Windows 64-bit
    Win64,
    /// Linux 32-bit
    X1132,
    /// Linux 64-bit
    X1164,
    /// MacOS
    OSX,
    /// Server (Linux 64-bit)
    LinuxServer64,
    /// Headless (Linux 64-bit)
    LinuxHeadless64,
}

/// Godot version.
#[derive(Debug, Clone, PartialEq)]
pub struct GodotVersion {
    version: String,
    kind: GodotVersionKind,
    mono: bool,
}

/// Godot version kind.
#[derive(Clone, Debug, PartialEq)]
pub enum GodotVersionKind {
    /// Stable release.
    Stable,
    /// Release candidate.
    ReleaseCandidate(u16),
    /// Alpha release.
    Alpha(u16),
    /// Beta release.
    Beta(u16),
    /// Custom release.
    Custom(String),
}

impl SystemVersion {
    /// Determine system kind.
    pub fn determine_system_kind() -> SystemVersion {
        if cfg!(target_os = "windows") {
            if cfg!(target_arch = "x86") {
                SystemVersion::Win32
            } else if cfg!(target_arch = "x86_64") {
                SystemVersion::Win64
            } else {
                panic!("Unsupported Windows architecture.")
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "x86") {
                SystemVersion::X1132
            } else if cfg!(target_arch = "x86_64") {
                SystemVersion::X1164
            } else {
                panic!("Unsupported Linux architecture.")
            }
        } else if cfg!(target_os = "macos") {
            SystemVersion::OSX
        } else {
            panic!("Unsupported OS.")
        }
    }

    /// Check if system is linux-based.
    pub fn is_linux(&self) -> bool {
        matches!(self, SystemVersion::X1132 | SystemVersion::X1164)
    }

    /// Get executable extension for current system.
    pub fn get_extension(&self) -> &'static str {
        match self {
            SystemVersion::Win32 | SystemVersion::Win64 => "exe",
            SystemVersion::X1132 | SystemVersion::X1164 => "x11",
            SystemVersion::LinuxHeadless64 => "headless.x11",
            SystemVersion::LinuxServer64 => "server.x11",
            SystemVersion::OSX => "osx",
        }
    }

    /// Get archive basename for current system.
    pub fn get_archive_basename(&self, with_mono: bool) -> &'static str {
        match self {
            SystemVersion::Win32 => {
                if with_mono {
                    "mono_win32"
                } else {
                    "win32.exe"
                }
            }
            SystemVersion::Win64 => {
                if with_mono {
                    "mono_win64"
                } else {
                    "win64.exe"
                }
            }
            SystemVersion::X1132 => {
                if with_mono {
                    "mono_x11_32"
                } else {
                    "x11.32"
                }
            }
            SystemVersion::X1164 => {
                if with_mono {
                    "mono_x11_64"
                } else {
                    "x11.64"
                }
            }
            SystemVersion::OSX => {
                if with_mono {
                    "mono_osx.64"
                } else {
                    "osx.universal"
                }
            }
            SystemVersion::LinuxServer64 => {
                if with_mono {
                    "mono_linux_server_64"
                } else {
                    "linux_server.64"
                }
            }
            SystemVersion::LinuxHeadless64 => {
                if with_mono {
                    "mono_linux_headless_64"
                } else {
                    "linux_server.64"
                }
            }
        }
    }
}

impl Display for SystemVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Win32 => write!(f, "win32"),
            Self::Win64 => write!(f, "win64"),
            Self::X1132 => write!(f, "linux32"),
            Self::X1164 => write!(f, "linux64"),
            Self::OSX => write!(f, "osx"),
            Self::LinuxServer64 => write!(f, "linuxserver64"),
            Self::LinuxHeadless64 => write!(f, "linuxheadless64"),
        }
    }
}

impl GodotVersion {
    /// Creates a new Godot version.
    pub fn new(version: &str, kind: GodotVersionKind, mono: bool) -> Self {
        Self {
            version: version.to_owned(),
            kind,
            mono,
        }
    }

    /// Get version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get kind.
    pub fn kind(&self) -> &GodotVersionKind {
        &self.kind
    }

    /// Is mono enabled?
    pub fn mono(&self) -> bool {
        self.mono
    }

    /// To GdValue.
    pub fn to_gdvalue(&self) -> GdValue {
        GdValue::Object(vec![
            ("version".into(), GdValue::String(self.version.clone())),
            ("kind".into(), GdValue::String(self.kind.to_string())),
            ("mono".into(), GdValue::Boolean(self.mono)),
        ])
    }

    /// From GdValue.
    pub fn from_gdvalue(value: GdValue) -> Option<Self> {
        if let Some(map) = value.to_object() {
            let version = map
                .get("version")
                .and_then(|x| x.to_str())
                .unwrap_or_else(|| String::from("unknown"));
            let kind: GodotVersionKind = map
                .get("kind")
                .and_then(|x| x.to_str())
                .map(|x| GodotVersionKind::from_str(&x).unwrap())
                .unwrap_or(GodotVersionKind::Stable);
            let mono = map.get("mono").and_then(|x| x.to_bool()).unwrap_or(false);

            Some(Self {
                version,
                kind,
                mono,
            })
        } else {
            None
        }
    }
}

impl Display for GodotVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)?;

        if self.kind != GodotVersionKind::Stable {
            write!(f, ".{}", self.kind)?;
        }

        if self.mono {
            write!(f, ".mono")?;
        }

        Ok(())
    }
}

impl FromStr for GodotVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts: Vec<_> = s.split('.').collect();
        let mut kind = GodotVersionKind::Stable;
        let mut mono = false;

        if let Some(&"mono") = parts.last() {
            mono = true;
            parts.pop();
        }

        if let Some(&x) = parts.last() {
            if x.parse::<u16>().is_err() {
                kind = GodotVersionKind::from_str(x)?;
                parts.pop();
            }
        }

        let version = parts.join(".");

        Ok(Self {
            version,
            kind,
            mono,
        })
    }
}

impl Display for GodotVersionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::ReleaseCandidate(n) if *n == 0 => write!(f, "rc"),
            Self::ReleaseCandidate(n) => write!(f, "rc{}", n),
            Self::Alpha(n) if *n == 0 => write!(f, "alpha"),
            Self::Alpha(n) => write!(f, "alpha{}", n),
            Self::Beta(n) if *n == 0 => write!(f, "beta"),
            Self::Beta(n) => write!(f, "beta{}", n),
            Self::Custom(c) => write!(f, "custom.{c}"),
        }
    }
}

impl FromStr for GodotVersionKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "stable" {
            Ok(Self::Stable)
        } else if s.starts_with("rc") {
            let number = s.chars().skip(2).collect::<String>().parse().unwrap_or(0);
            Ok(Self::ReleaseCandidate(number))
        } else if s.starts_with("alpha") {
            let number = s.chars().skip(5).collect::<String>().parse().unwrap_or(0);
            Ok(Self::Alpha(number))
        } else if s.starts_with("beta") {
            let number = s.chars().skip(4).collect::<String>().parse().unwrap_or(0);
            Ok(Self::Beta(number))
        } else if s.starts_with("custom.") {
            let custom_string = s.chars().skip(7).collect::<String>();
            Ok(Self::Custom(custom_string))
        } else {
            Err(Error::WrongVersionKind(s.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_kind() {
        assert_eq!(
            GodotVersionKind::from_str("rc").unwrap(),
            GodotVersionKind::ReleaseCandidate(0)
        );
        assert_eq!(
            GodotVersionKind::from_str("rc9").unwrap(),
            GodotVersionKind::ReleaseCandidate(9)
        );
        assert_eq!(
            GodotVersionKind::from_str("alpha").unwrap(),
            GodotVersionKind::Alpha(0)
        );
        assert_eq!(
            GodotVersionKind::from_str("alpha2").unwrap(),
            GodotVersionKind::Alpha(2)
        );
        assert_eq!(
            GodotVersionKind::from_str("beta").unwrap(),
            GodotVersionKind::Beta(0)
        );
        assert_eq!(
            GodotVersionKind::from_str("beta1").unwrap(),
            GodotVersionKind::Beta(1)
        );
        assert_eq!(
            GodotVersionKind::from_str("stable").unwrap(),
            GodotVersionKind::Stable
        );
    }

    #[test]
    fn test_parse_version() {
        assert_eq!(
            GodotVersion::from_str("3.0").unwrap(),
            GodotVersion::new("3.0", GodotVersionKind::Stable, false)
        );
        assert_eq!(
            GodotVersion::from_str("3.0.mono").unwrap(),
            GodotVersion::new("3.0", GodotVersionKind::Stable, true)
        );
        assert_eq!(
            GodotVersion::from_str("3.0.alpha1").unwrap(),
            GodotVersion::new("3.0", GodotVersionKind::Alpha(1), false)
        );
        assert_eq!(
            GodotVersion::from_str("3.1.beta2.mono").unwrap(),
            GodotVersion::new("3.1", GodotVersionKind::Beta(2), true)
        );
        assert_eq!(
            GodotVersion::from_str("3.1.2").unwrap(),
            GodotVersion::new("3.1.2", GodotVersionKind::Stable, false)
        );
        assert_eq!(
            GodotVersion::from_str("3.1.2.mono").unwrap(),
            GodotVersion::new("3.1.2", GodotVersionKind::Stable, true)
        );
    }
}
