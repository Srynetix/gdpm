//! Versions.

use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use gdsettings_parser::GdValue;
use slugify::slugify;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wrong version: {0}")]
    WrongVersion(String),

    #[error("Wrong version kind: {0}")]
    WrongVersionKind(String),
}

/// System version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemVersion {
    /// Windows 32-bit
    Win32,
    /// Windows 64-bit
    Win64,
    /// Windows ARM 64-bit
    WinArm64,
    /// Linux x86 32-bit
    LinuxX32,
    /// Linux x86 64-bit
    LinuxX64,
    /// Linux ARM 32-bit
    LinuxArm32,
    /// Linux ARM 64-bit
    LinuxArm64,
    /// MacOS
    MacOS,
}

/// Godot version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GodotVersion {
    version: String,
    kind: GodotVersionKind,
    system: SystemVersion,
    mono: bool,
}

/// Godot version kind.
#[derive(Clone, Debug, PartialEq, Eq)]
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
                SystemVersion::LinuxX32
            } else if cfg!(target_arch = "x86_64") {
                SystemVersion::LinuxX64
            } else if cfg!(target_arch = "arm") {
                SystemVersion::LinuxArm32
            } else if cfg!(target_arch = "aarch64") {
                SystemVersion::LinuxArm64
            } else {
                panic!("Unsupported Linux architecture.")
            }
        } else if cfg!(target_os = "macos") {
            SystemVersion::MacOS
        } else {
            panic!("Unsupported OS.")
        }
    }

    /// Check if system is linux-based.
    pub fn is_linux(&self) -> bool {
        matches!(
            self,
            SystemVersion::LinuxArm32
                | SystemVersion::LinuxArm64
                | SystemVersion::LinuxX32
                | SystemVersion::LinuxX64
        )
    }

    /// Get executable extension for current system.
    pub fn get_extension(&self) -> &'static str {
        match self {
            SystemVersion::Win32 | SystemVersion::Win64 | SystemVersion::WinArm64 => "exe",
            SystemVersion::LinuxX32 => "x86_32",
            SystemVersion::LinuxX64 => "x86_64",
            SystemVersion::LinuxArm32 => "arm32",
            SystemVersion::LinuxArm64 => "arm64",
            SystemVersion::MacOS => "app",
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
            SystemVersion::WinArm64 => {
                if with_mono {
                    "mono_windows_arm64"
                } else {
                    "windows_arm64.exe"
                }
            }
            SystemVersion::LinuxX32 => {
                if with_mono {
                    "mono_linux_x86_32"
                } else {
                    "linux.x86_32"
                }
            }
            SystemVersion::LinuxX64 => {
                if with_mono {
                    "mono_linux_x86_64"
                } else {
                    "linux.x86_64"
                }
            }
            SystemVersion::MacOS => {
                if with_mono {
                    "mono_macos.64"
                } else {
                    "macos.universal"
                }
            }
            SystemVersion::LinuxArm32 => {
                if with_mono {
                    "mono_linux_arm32"
                } else {
                    "linux.arm32"
                }
            }
            SystemVersion::LinuxArm64 => {
                if with_mono {
                    "mono_linux_arm64"
                } else {
                    "linux.arm64"
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
            Self::WinArm64 => write!(f, "win-arm64"),
            Self::LinuxX32 => write!(f, "linux-x32"),
            Self::LinuxX64 => write!(f, "linux-x64"),
            Self::MacOS => write!(f, "macos"),
            Self::LinuxArm32 => write!(f, "linux-arm32"),
            Self::LinuxArm64 => write!(f, "linux-arm64"),
        }
    }
}

impl FromStr for SystemVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "win32" => Ok(SystemVersion::Win32),
            "win64" => Ok(SystemVersion::Win64),
            "win-arm64" => Ok(SystemVersion::WinArm64),
            "linux-x32" => Ok(SystemVersion::LinuxX32),
            "linux-x64" => Ok(SystemVersion::LinuxX64),
            "macos" => Ok(SystemVersion::MacOS),
            "linux-arm32" => Ok(SystemVersion::LinuxArm32),
            "linux-arm64" => Ok(SystemVersion::LinuxArm64),
            _ => Err(Error::WrongVersion(format!("Unknown version {}", s))),
        }
    }
}

impl GodotVersion {
    /// Creates a new Godot version.
    pub fn new(version: &str, kind: GodotVersionKind, system: SystemVersion, mono: bool) -> Self {
        Self {
            version: version.to_owned(),
            kind,
            system,
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

    /// Get version slug.
    pub fn slug(&self) -> String {
        slugify!(&self.to_string())
    }

    /// Get system version.
    pub fn system(&self) -> SystemVersion {
        self.system
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
            let system: SystemVersion = map
                .get("system")
                .and_then(|x| x.to_str())
                .map(|x| x.parse().unwrap())
                .unwrap_or_else(|| SystemVersion::determine_system_kind());
            let mono = map.get("mono").and_then(|x| x.to_bool()).unwrap_or(false);

            Some(Self {
                version,
                kind,
                system,
                mono,
            })
        } else {
            None
        }
    }

    /// Get export template name
    pub fn get_export_template_name(&self) -> String {
        let mut output = String::new();
        output.write_str(&self.version).unwrap();
        output.write_fmt(format_args!(".{}", self.kind)).unwrap();

        if self.mono {
            output.write_str(".mono").unwrap();
        }

        output
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

/*
Syntax

4.3.mono
4.3.mono.win64
4.1.2.mono.win64
4.2.beta1.mono.win64
*/

impl FromStr for GodotVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('.').collect();
        let mut kind = GodotVersionKind::Stable;
        let mut mono = false;
        let mut version_number: Vec<&str> = vec![];
        let mut system = SystemVersion::determine_system_kind();
        let mut version_number_done = false;

        for part in parts {
            // Try to parse a version
            if part.parse::<u16>().is_ok() {
                if version_number_done {
                    return Err(Error::WrongVersion(s.to_string()));
                }

                version_number.push(part);
                continue;
            }

            // We finished reading numbers
            if !version_number_done {
                version_number_done = true;
            }

            // Now that's a string
            if part == "mono" {
                mono = true;
                continue;
            }

            // Try to parse a kind
            if let Ok(kind_) = part.parse::<GodotVersionKind>() {
                kind = kind_;
                continue;
            }

            // Try to parse a system version
            if let Ok(version) = part.parse::<SystemVersion>() {
                system = version;
                continue;
            }

            // That's another thing, error out
            return Err(Error::WrongVersion(s.to_string()));
        }

        Ok(Self {
            version: version_number.join("."),
            kind,
            system,
            mono,
        })
    }
}

impl TryFrom<&str> for GodotVersion {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
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

impl TryFrom<&str> for GodotVersionKind {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use GodotVersionKind::*;
    use SystemVersion::*;
    use test_case::test_case;

    fn system_auto() -> SystemVersion {
        SystemVersion::determine_system_kind()
    }

    const NO_MONO: bool = false;
    const MONO: bool = true;

    #[test_case("rc", ReleaseCandidate(0))]
    #[test_case("rc9", ReleaseCandidate(9))]
    #[test_case("alpha", Alpha(0))]
    #[test_case("alpha2", Alpha(2))]
    #[test_case("beta", Beta(0))]
    #[test_case("beta1", Beta(1))]
    #[test_case("stable", Stable)]
    fn test_parse_version_kind(version: &str, kind: GodotVersionKind) {
        assert_eq!(version.parse::<GodotVersionKind>().unwrap(), kind);
    }

    #[test_case("3.0", "3.0", Stable, system_auto(), NO_MONO)]
    #[test_case("3.0.mono", "3.0", Stable, system_auto(), MONO)]
    #[test_case("3.0.beta1", "3.0", Beta(1), system_auto(), NO_MONO)]
    #[test_case("3.0.beta1.win32", "3.0", Beta(1), Win32, NO_MONO)]
    #[test_case("3.0.beta1.mono.linux-arm64", "3.0", Beta(1), LinuxArm64, MONO)]
    #[test_case("3.0.rc1.mono.linux-x32", "3.0", ReleaseCandidate(1), LinuxX32, MONO)]
    #[test_case("3.0.mono.macos", "3.0", Stable, MacOS, MONO)]
    fn test_parse_version(version: &str, gd_version: &str, kind: GodotVersionKind, system: SystemVersion, mono: bool) {
        assert_eq!(version.parse::<GodotVersion>().unwrap(), GodotVersion::new(gd_version, kind, system, mono));
    }

    #[test]
    fn test_export_template_name() {
        assert_eq!(
            GodotVersion::from_str("3.1.2")
                .unwrap()
                .get_export_template_name(),
            "3.1.2.stable"
        );
        assert_eq!(
            GodotVersion::from_str("3.1.2.mono")
                .unwrap()
                .get_export_template_name(),
            "3.1.2.stable.mono"
        );
    }
}
