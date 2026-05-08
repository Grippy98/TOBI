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
    let kind = if metadata.is_file() {
        TargetKind::File
    } else {
        infer_kind(path)
    };

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
        size_bytes: if metadata.is_file() {
            Some(metadata.len())
        } else {
            None
        },
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
        let removable = fs::read_to_string(sys_path.join("removable"))
            .ok()
            .map(|value| value.trim() == "1")
            .unwrap_or(false);
        let size_bytes = fs::read_to_string(sys_path.join("size"))
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .map(|sectors| sectors.saturating_mul(512));
        let path = PathBuf::from(format!("/dev/{name}"));
        let kind = infer_linux_kind(&name, removable);
        let partitions = linux_partitions(&sys_path);

        targets.push(InstallTarget {
            id: name.clone(),
            name: linux_device_model(&sys_path).unwrap_or_else(|| name.clone()),
            path,
            size_bytes,
            kind,
            removable,
            partitions,
            warning: Some("live target; writing requires --allow-write".to_string()),
        });
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
fn linux_device_model(sys_path: &Path) -> Option<String> {
    fs::read_to_string(sys_path.join("device/model"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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

fn infer_linux_kind(name: &str, removable: bool) -> TargetKind {
    if name.starts_with("mmcblk") {
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
        assert_eq!(infer_linux_kind("mmcblk0", true), TargetKind::Sd);
        assert_eq!(infer_linux_kind("mmcblk1", false), TargetKind::Emmc);
    }
}
