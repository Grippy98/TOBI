use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

use crate::installer::RunMode;

#[derive(Clone, Debug)]
pub struct InstallTarget {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: Option<u64>,
    pub kind: TargetKind,
    pub removable: bool,
    pub partitions: Vec<TargetPartition>,
    pub warning: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TargetPartition {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetKind {
    Emmc,
    Sd,
    Usb,
    Nvme,
    File,
    Unknown,
}

#[derive(Clone, Copy, Debug)]
pub enum DeviceMode {
    Mock,
    Live,
}

impl From<RunMode> for DeviceMode {
    fn from(value: RunMode) -> Self {
        match value {
            RunMode::Mock => Self::Mock,
            RunMode::Live => Self::Live,
        }
    }
}

pub fn list_devices(mode: DeviceMode, target: Option<&Path>) -> anyhow::Result<Vec<InstallTarget>> {
    match mode {
        DeviceMode::Mock => Ok(mock_targets()),
        DeviceMode::Live => live_targets(target),
    }
}

pub fn format_bytes(bytes: Option<u64>) -> String {
    let Some(bytes) = bytes else {
        return "unknown size".to_string();
    };
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn mock_targets() -> Vec<InstallTarget> {
    vec![
        InstallTarget {
            id: "mock-emmc".to_string(),
            name: "SK-AM62P-LP eMMC".to_string(),
            path: PathBuf::from("/dev/mmcblk0"),
            size_bytes: Some(16 * 1024 * 1024 * 1024),
            kind: TargetKind::Emmc,
            removable: false,
            partitions: vec![
                TargetPartition {
                    name: "boot".to_string(),
                    path: PathBuf::from("/dev/mmcblk0p1"),
                    size_bytes: Some(128 * 1024 * 1024),
                },
                TargetPartition {
                    name: "rootfs".to_string(),
                    path: PathBuf::from("/dev/mmcblk0p2"),
                    size_bytes: Some(15 * 1024 * 1024 * 1024),
                },
            ],
            warning: Some("mock target; no data will be written".to_string()),
        },
        InstallTarget {
            id: "mock-usb".to_string(),
            name: "USB recovery media".to_string(),
            path: PathBuf::from("/dev/sda"),
            size_bytes: Some(32 * 1024 * 1024 * 1024),
            kind: TargetKind::Usb,
            removable: true,
            partitions: Vec::new(),
            warning: Some("mock target; no data will be written".to_string()),
        },
        InstallTarget {
            id: "mock-sd".to_string(),
            name: "SD card media".to_string(),
            path: PathBuf::from("/dev/mmcblk1"),
            size_bytes: Some(64 * 1024 * 1024 * 1024),
            kind: TargetKind::Sd,
            removable: true,
            partitions: vec![TargetPartition {
                name: "boot".to_string(),
                path: PathBuf::from("/dev/mmcblk1p1"),
                size_bytes: Some(256 * 1024 * 1024),
            }],
            warning: Some("mock target; no data will be written".to_string()),
        },
    ]
}

fn live_targets(target: Option<&Path>) -> anyhow::Result<Vec<InstallTarget>> {
    if let Some(path) = target {
        return target_from_path(path).map(|target| vec![target]);
    }

    #[cfg(target_os = "linux")]
    {
        scan_linux_block_devices()
    }

    #[cfg(not(target_os = "linux"))]
    {
        bail!(
            "live block device scanning is only implemented on Linux; pass --target to test with a file"
        )
    }
}

fn target_from_path(path: &Path) -> anyhow::Result<InstallTarget> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("target {} is not accessible", path.display()))?;
    if metadata.is_file() {
        return Ok(InstallTarget {
            id: path.display().to_string(),
            name: "File target".to_string(),
            path: path.to_path_buf(),
            size_bytes: Some(metadata.len()),
            kind: TargetKind::File,
            removable: false,
            partitions: Vec::new(),
            warning: Some("live target; writing requires --allow-write".to_string()),
        });
    }

    #[cfg(target_os = "linux")]
    if let Some(target) = linux_target_from_path(path) {
        return Ok(target);
    }

    let kind = infer_kind(path);

    Ok(InstallTarget {
        id: path.display().to_string(),
        name: match kind {
            TargetKind::File => "File target".to_string(),
            TargetKind::Emmc => "eMMC target".to_string(),
            TargetKind::Sd => "SD target".to_string(),
            TargetKind::Usb => "USB/storage target".to_string(),
            TargetKind::Nvme => "NVMe target".to_string(),
            TargetKind::Unknown => "Block target".to_string(),
        },
        path: path.to_path_buf(),
        size_bytes: None,
        kind,
        removable: false,
        partitions: Vec::new(),
        warning: Some("live target; writing requires --allow-write".to_string()),
    })
}

#[cfg(target_os = "linux")]
fn scan_linux_block_devices() -> anyhow::Result<Vec<InstallTarget>> {
    let mut targets = Vec::new();
    for entry in fs::read_dir("/sys/block").context("failed to read /sys/block")? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_visible_root_block_device(&name) {
            continue;
        }

        let sys_path = entry.path();
        targets.push(linux_target_from_sys_block(
            &name,
            PathBuf::from(format!("/dev/{name}")),
            &sys_path,
        ));
    }

    if targets.is_empty() {
        bail!("no writable block devices were found");
    }
    targets.sort_by(|a, b| {
        target_kind_rank(a.kind)
            .cmp(&target_kind_rank(b.kind))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(targets)
}

#[cfg(target_os = "linux")]
fn linux_target_from_path(path: &Path) -> Option<InstallTarget> {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let name = path.file_name()?.to_str()?.to_string();
    if !is_visible_root_block_device(&name) {
        return None;
    }

    let sys_path = PathBuf::from("/sys/block").join(&name);
    if !sys_path.exists() {
        return None;
    }

    Some(linux_target_from_sys_block(&name, path, &sys_path))
}

#[cfg(target_os = "linux")]
fn linux_target_from_sys_block(name: &str, path: PathBuf, sys_path: &Path) -> InstallTarget {
    let sys_removable = linux_removable(sys_path);
    let mmc_type = linux_mmc_type(sys_path);
    let kind = infer_linux_kind(name, sys_removable, mmc_type.as_deref());
    let removable = sys_removable || kind == TargetKind::Sd;

    InstallTarget {
        id: name.to_string(),
        name: linux_device_label(sys_path, name, kind),
        path,
        size_bytes: linux_block_size(sys_path),
        kind,
        removable,
        partitions: linux_partitions(sys_path),
        warning: Some("live target; writing requires --allow-write".to_string()),
    }
}

#[cfg(target_os = "linux")]
fn linux_read_trimmed(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(target_os = "linux")]
fn linux_removable(sys_path: &Path) -> bool {
    linux_read_trimmed(sys_path.join("removable"))
        .map(|value| value == "1")
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn linux_block_size(sys_path: &Path) -> Option<u64> {
    linux_read_trimmed(sys_path.join("size"))
        .and_then(|value| value.parse::<u64>().ok())
        .map(|sectors| sectors.saturating_mul(512))
}

#[cfg(target_os = "linux")]
fn linux_mmc_type(sys_path: &Path) -> Option<String> {
    linux_read_trimmed(sys_path.join("device/type"))
        .or_else(|| {
            fs::read_to_string(sys_path.join("device/uevent"))
                .ok()?
                .lines()
                .find_map(|line| line.strip_prefix("MMC_TYPE=").map(str::to_string))
        })
        .map(|value| value.to_ascii_uppercase())
}

#[cfg(target_os = "linux")]
fn linux_device_model(sys_path: &Path) -> Option<String> {
    linux_read_trimmed(sys_path.join("device/model"))
}

#[cfg(target_os = "linux")]
fn linux_mmc_name(sys_path: &Path) -> Option<String> {
    linux_read_trimmed(sys_path.join("device/name"))
}

#[cfg(target_os = "linux")]
fn linux_device_label(sys_path: &Path, fallback_name: &str, kind: TargetKind) -> String {
    if let Some(model) = linux_device_model(sys_path) {
        return model;
    }

    let kind_label = match kind {
        TargetKind::Emmc => "eMMC",
        TargetKind::Sd => "SD card",
        TargetKind::Usb => "USB/storage",
        TargetKind::Nvme => "NVMe",
        TargetKind::File => "file",
        TargetKind::Unknown => "storage",
    };

    if matches!(kind, TargetKind::Emmc | TargetKind::Sd) {
        if let Some(card_name) = linux_mmc_name(sys_path) {
            return format!("{kind_label} {card_name}");
        }
    }

    format!("{kind_label} {fallback_name}")
}

#[cfg(target_os = "linux")]
fn linux_partitions(sys_path: &Path) -> Vec<TargetPartition> {
    let Ok(entries) = fs::read_dir(sys_path) else {
        return Vec::new();
    };

    let mut partitions = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            if !entry.path().join("partition").exists() {
                return None;
            }

            let name = entry.file_name().to_string_lossy().into_owned();
            let size_bytes = fs::read_to_string(entry.path().join("size"))
                .ok()
                .and_then(|value| value.trim().parse::<u64>().ok())
                .map(|sectors| sectors.saturating_mul(512));

            Some(TargetPartition {
                name: name.clone(),
                path: PathBuf::from(format!("/dev/{name}")),
                size_bytes,
            })
        })
        .collect::<Vec<_>>();
    partitions.sort_by(|a, b| a.name.cmp(&b.name));
    partitions
}

#[cfg(any(target_os = "linux", test))]
fn is_visible_root_block_device(name: &str) -> bool {
    if name.starts_with("loop")
        || name.starts_with("ram")
        || name.starts_with("zram")
        || name.starts_with("mtdblock")
        || name.starts_with("dm-")
        || name.starts_with("sr")
    {
        return false;
    }

    !(name.starts_with("mmcblk") && (name.contains("boot") || name.contains("rpmb")))
}

fn infer_kind(path: &Path) -> TargetKind {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if name.starts_with("mmcblk") {
        TargetKind::Emmc
    } else if name.starts_with("sd") {
        TargetKind::Usb
    } else if name.starts_with("nvme") {
        TargetKind::Nvme
    } else {
        TargetKind::Unknown
    }
}

#[cfg(any(target_os = "linux", test))]
fn infer_linux_kind(name: &str, removable: bool, mmc_type: Option<&str>) -> TargetKind {
    if name.starts_with("mmcblk") {
        match mmc_type.map(str::to_ascii_uppercase).as_deref() {
            Some("SD" | "SDIO") => return TargetKind::Sd,
            Some("MMC") => return TargetKind::Emmc,
            _ => {}
        }
        if removable {
            TargetKind::Sd
        } else {
            TargetKind::Emmc
        }
    } else if name.starts_with("sd") {
        TargetKind::Usb
    } else if name.starts_with("nvme") {
        TargetKind::Nvme
    } else {
        TargetKind::Unknown
    }
}

#[cfg(target_os = "linux")]
fn target_kind_rank(kind: TargetKind) -> u8 {
    match kind {
        TargetKind::Emmc => 0,
        TargetKind::Sd => 1,
        TargetKind::Usb => 2,
        TargetKind::Nvme => 3,
        TargetKind::File => 4,
        TargetKind::Unknown => 9,
    }
}

impl TargetKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Emmc => "eMMC",
            Self::Sd => "SD",
            Self::Usb => "USB",
            Self::Nvme => "NVMe",
            Self::File => "file",
            Self::Unknown => "storage",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_binary_sizes() {
        assert_eq!(format_bytes(Some(16 * 1024 * 1024 * 1024)), "16.0 GiB");
        assert_eq!(format_bytes(None), "unknown size");
    }

    #[test]
    fn mock_mode_lists_emmc() {
        let targets = list_devices(DeviceMode::Mock, None).expect("mock targets");
        assert!(targets.iter().any(|target| target.kind == TargetKind::Emmc));
    }

    #[test]
    fn hides_partition_like_block_roots() {
        assert!(is_visible_root_block_device("mmcblk1"));
        assert!(is_visible_root_block_device("sda"));
        assert!(!is_visible_root_block_device("mmcblk0boot0"));
        assert!(!is_visible_root_block_device("mmcblk0rpmb"));
        assert!(!is_visible_root_block_device("mtdblock4"));
        assert!(!is_visible_root_block_device("sr0"));
    }

    #[test]
    fn classifies_removable_mmc_as_sd() {
        assert_eq!(infer_linux_kind("mmcblk0", true, None), TargetKind::Sd);
        assert_eq!(infer_linux_kind("mmcblk1", false, None), TargetKind::Emmc);
    }

    #[test]
    fn classifies_mmc_by_card_type_before_numbering() {
        assert_eq!(
            infer_linux_kind("mmcblk0", false, Some("SD")),
            TargetKind::Sd
        );
        assert_eq!(
            infer_linux_kind("mmcblk1", true, Some("MMC")),
            TargetKind::Emmc
        );
    }
}
