use crate::installer::RunMode;
use crate::manifest::Catalog;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DetectedBoard {
    pub id: Option<String>,
    pub name: String,
    pub compatible: Vec<String>,
    pub source: BoardSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum BoardSource {
    DeviceTree,
    Mock,
    Unknown,
}

pub fn detect_board(mode: RunMode, catalog: &Catalog) -> DetectedBoard {
    match mode {
        RunMode::Mock => mock_board(catalog),
        RunMode::Live => live_board(catalog),
    }
}

fn mock_board(catalog: &Catalog) -> DetectedBoard {
    let mock_board_id = std::env::var("TOBI_MOCK_BOARD")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "sk-am62p-lp".to_string());
    let device = catalog
        .devices
        .iter()
        .find(|device| device.id == mock_board_id)
        .or_else(|| catalog.devices.first());

    match device {
        Some(device) => DetectedBoard {
            id: Some(device.id.clone()),
            name: device.name.clone(),
            compatible: device.compatible.clone(),
            source: BoardSource::Mock,
        },
        None => DetectedBoard {
            id: None,
            name: "SK-AM62P-LP".to_string(),
            compatible: vec!["ti,am62pxx-evm".to_string()],
            source: BoardSource::Mock,
        },
    }
}

fn live_board(catalog: &Catalog) -> DetectedBoard {
    #[cfg(target_os = "linux")]
    {
        let compatible = read_compatible();
        let matched = compatible
            .as_ref()
            .and_then(|compatible| match_catalog_device(catalog, compatible));
        let model = read_model();

        if let Some(device) = matched {
            return DetectedBoard {
                id: Some(device.id.clone()),
                name: device.name.clone(),
                compatible: compatible.unwrap_or_else(|| device.compatible.clone()),
                source: BoardSource::DeviceTree,
            };
        }

        return DetectedBoard {
            id: None,
            name: model.unwrap_or_else(|| "Unknown Sitara board".to_string()),
            compatible: compatible.unwrap_or_default(),
            source: BoardSource::Unknown,
        };
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = catalog;
        DetectedBoard {
            id: None,
            name: "Unknown board".to_string(),
            compatible: Vec::new(),
            source: BoardSource::Unknown,
        }
    }
}

#[cfg(target_os = "linux")]
fn read_compatible() -> Option<Vec<String>> {
    let bytes = std::fs::read("/proc/device-tree/compatible").ok()?;
    let compatible = bytes
        .split(|byte| *byte == 0)
        .filter_map(|chunk| std::str::from_utf8(chunk).ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    (!compatible.is_empty()).then_some(compatible)
}

#[cfg(target_os = "linux")]
fn read_model() -> Option<String> {
    std::fs::read("/proc/device-tree/model")
        .ok()
        .and_then(|bytes| {
            String::from_utf8(
                bytes
                    .into_iter()
                    .take_while(|byte| *byte != 0)
                    .collect::<Vec<_>>(),
            )
            .ok()
        })
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
}

fn match_catalog_device<'a>(
    catalog: &'a Catalog,
    compatible: &[String],
) -> Option<&'a crate::manifest::DeviceEntry> {
    compatible.iter().find_map(|detected| {
        catalog
            .devices
            .iter()
            .find(|device| device.compatible.iter().any(|entry| entry == detected))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::DeviceEntry;

    #[test]
    fn match_catalog_device_prefers_detected_compatible_order() {
        let catalog = Catalog {
            schema_version: 1,
            generated_at: None,
            devices: vec![
                DeviceEntry {
                    id: "sk-am64b".to_string(),
                    name: "SK-AM64B".to_string(),
                    compatible: vec!["ti,am642-sk".to_string(), "ti,am642".to_string()],
                },
                DeviceEntry {
                    id: "tmds64evm".to_string(),
                    name: "TMDS64EVM".to_string(),
                    compatible: vec!["ti,am642-evm".to_string(), "ti,am642".to_string()],
                },
            ],
            images: Vec::new(),
        };

        let matched = match_catalog_device(
            &catalog,
            &["ti,am642-evm".to_string(), "ti,am642".to_string()],
        )
        .expect("matched board");

        assert_eq!(matched.id, "tmds64evm");
    }
}
