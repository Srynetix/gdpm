//! Versions.

use std::fmt::Display;

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

impl Display for GodotVersionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::ReleaseCandidate(n) => write!(f, "rc{}", n),
            Self::Alpha(n) => write!(f, "alpha{}", n),
            Self::Beta(n) => write!(f, "beta{}", n),
        }
    }
}

impl From<&str> for GodotVersionKind {
    fn from(s: &str) -> Self {
        if s == "stable" {
            Self::Stable
        } else if s.starts_with("rc") {
            let number = s.chars().skip(2).collect::<String>().parse().unwrap();
            Self::ReleaseCandidate(number)
        } else if s.starts_with("alpha") {
            let number = s.chars().skip(5).collect::<String>().parse().unwrap();
            Self::Alpha(number)
        } else if s.starts_with("beta") {
            let number = s.chars().skip(4).collect::<String>().parse().unwrap();
            Self::Beta(number)
        } else {
            panic!("Unsupported version kind: {}", s);
        }
    }
}
