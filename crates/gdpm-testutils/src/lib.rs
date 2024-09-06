use std::alloc::System;

use gdpm_io::IoAdapter;
use gdpm_types::version::{GodotVersion, SystemVersion};

struct GodotPackageBuilder;

impl GodotPackageBuilder {}

struct GodotTemplatePackageBuilder<I: IoAdapter> {
    io_adapter: I,
}

impl<I: IoAdapter> GodotTemplatePackageBuilder<I> {
    pub fn build(&self, version: &GodotVersion) {}
}

pub fn get_engine_archive_name_from_version(version: &GodotVersion) -> String {
    let mut builder = String::from("Godot_v");
    builder.push_str(&version.version());
    builder.push('-');
    builder.push_str(&version.kind().to_string());
    builder.push('_');

    let package_comp = match (version.system(), version.mono()) {
        (SystemVersion::Win32, true) => "mono_win32.zip",
        (SystemVersion::Win32, false) => "win32.exe.zip",
        (SystemVersion::Win64, true) => "mono_win64.zip",
        (SystemVersion::Win64, false) => "win64.exe.zip",
        (SystemVersion::WinArm64, true) => "mono_windows_arm64.zip",
        (SystemVersion::WinArm64, false) => "windows_arm64.exe.zip",
        (SystemVersion::MacOS, true) => "mono_macos.universal.zip",
        (SystemVersion::MacOS, false) => "macos.universal.zip",
        (SystemVersion::LinuxX32, true) => "mono_linux_x86_32.zip",
        (SystemVersion::LinuxX32, false) => "linux.x86_32.zip",
        (SystemVersion::LinuxX64, true) => "mono_linux_x86_64.zip",
        (SystemVersion::LinuxX64, false) => "linux.x86_64.zip",
        (SystemVersion::LinuxArm32, true) => "mono_linux_arm32.zip",
        (SystemVersion::LinuxArm32, false) => "linux.arm32.zip",
        (SystemVersion::LinuxArm64, true) => "mono_linux_arm64.zip",
        (SystemVersion::LinuxArm64, false) => "linux.arm64.zip",
    };

    format!("{}{}", builder, package_comp)
}

pub fn get_template_archive_name_from_version(version: &GodotVersion) -> String {
    let mut builder = String::from("Godot_v");
    builder.push_str(&version.version());
    builder.push('-');
    builder.push_str(&version.kind().to_string());

    if version.mono() {
        builder.push_str("_mono");
    }

    format!("{}_export_templates.tgz", builder)
}

#[cfg(test)]
mod tests {
    use gdpm_types::version::GodotVersion;

    #[test]
    fn test_get_engine_archive_name_from_version() {
        fn check(version: &str, filename: &str) {
            let gd_version = version.parse::<GodotVersion>().unwrap();
            assert_eq!(
                super::get_engine_archive_name_from_version(&gd_version),
                filename
            );
        }

        check(
            "4.3.mono.win32",
            "Godot_v4.3-stable_mono_win32.zip",
        );

        check(
            "4.3.win32",
            "Godot_v4.3-stable_win32.exe.zip",
        );

        check(
            "4.2.1.win64",
            "Godot_v4.2.1-stable_win64.exe.zip"
        );

        check(
            "4.3.beta1.mono.win64",
            "Godot_v4.3-beta1_mono_win64.zip"
        );

        check(
            "4.3.macos",
            "Godot_v4.3-stable_macos.universal.zip"
        );
    }
}
