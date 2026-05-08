use std::collections::BTreeSet;
use std::process::Command;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use crate::board::DetectedBoard;
use crate::custom_image::{
    CustomImage, custom_placeholder, is_custom_placeholder, scan_custom_images,
};
use crate::device::{DeviceMode, InstallTarget, TargetKind, list_devices};
use crate::installer::{InstallEvent, InstallRequest, RunMode, start_install};
use crate::manifest::{self, Catalog, ImageEntry};
use crate::memory::{MemoryCheck, check_image_memory};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    ImageSelect,
    CustomImageSelect,
    TargetSelect,
    Confirm,
    Installing,
    Complete,
    Error,
    ProxyConfig,
}

#[derive(Clone, Debug)]
pub struct ProgressState {
    pub phase: String,
    pub current: u64,
    pub total: Option<u64>,
    pub source_current: Option<u64>,
    pub source_total: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct SystemStatus {
    pub time: String,
    pub ip: String,
}

#[derive(Clone, Debug)]
pub struct RunnerGame {
    tick: u64,
    score: u64,
    runner_y: i16,
    velocity: i16,
    obstacles: Vec<Obstacle>,
    crashed: bool,
}

#[derive(Clone, Debug)]
pub struct Obstacle {
    pub x: i16,
    pub width: i16,
    pub kind: ObstacleKind,
}

#[derive(Clone, Copy, Debug)]
pub enum ObstacleKind {
    Cactus,
    Rock,
}

pub struct App {
    catalog: Catalog,
    board: DetectedBoard,
    devices: Vec<InstallTarget>,
    custom_images: Vec<CustomImage>,
    run_mode: RunMode,
    allow_write: bool,
    manifest_source: String,
    proxy_url: Option<String>,
    proxy_input: String,
    screen: Screen,
    image_index: usize,
    custom_image_index: usize,
    target_index: usize,
    expanded_targets: BTreeSet<String>,
    active_image: Option<ImageEntry>,
    active_image_is_custom: bool,
    progress: Option<ProgressState>,
    install_started_at: Option<Instant>,
    status: String,
    warning: Option<String>,
    install_rx: Option<Receiver<InstallEvent>>,
    runner: RunnerGame,
    system_status: SystemStatus,
    system_status_refreshed_at: Option<Instant>,
}

impl App {
    pub fn new(
        mut catalog: Catalog,
        board: DetectedBoard,
        devices: Vec<InstallTarget>,
        run_mode: RunMode,
        allow_write: bool,
        manifest_source: String,
        proxy_url: Option<String>,
        warning: Option<String>,
    ) -> Self {
        catalog.images.push(custom_placeholder());
        sort_images_for_display(&mut catalog.images);
        let image_index = catalog
            .images
            .iter()
            .position(|image| image.recommended)
            .unwrap_or(0);

        Self {
            catalog,
            board,
            devices,
            custom_images: Vec::new(),
            run_mode,
            allow_write,
            manifest_source,
            proxy_input: proxy_url.clone().unwrap_or_default(),
            proxy_url,
            screen: Screen::ImageSelect,
            image_index,
            custom_image_index: 0,
            target_index: 0,
            expanded_targets: BTreeSet::new(),
            active_image: None,
            active_image_is_custom: false,
            progress: None,
            install_started_at: None,
            status: "Choose an operating system image.".to_string(),
            warning,
            install_rx: None,
            runner: RunnerGame::default(),
            system_status: read_system_status(),
            system_status_refreshed_at: Some(Instant::now()),
        }
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn board(&self) -> &DetectedBoard {
        &self.board
    }

    pub fn devices(&self) -> &[InstallTarget] {
        &self.devices
    }

    pub fn custom_images(&self) -> &[CustomImage] {
        &self.custom_images
    }

    pub fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn image_index(&self) -> usize {
        self.image_index
    }

    pub fn custom_image_index(&self) -> usize {
        self.custom_image_index
    }

    pub fn target_index(&self) -> usize {
        self.target_index
    }

    pub fn selected_catalog_image(&self) -> Option<&ImageEntry> {
        self.catalog.images.get(self.image_index)
    }

    pub fn selected_image(&self) -> Option<&ImageEntry> {
        if matches!(
            self.screen,
            Screen::TargetSelect
                | Screen::Confirm
                | Screen::Installing
                | Screen::Complete
                | Screen::Error
        ) {
            self.active_image
                .as_ref()
                .or_else(|| self.catalog.images.get(self.image_index))
        } else {
            self.catalog.images.get(self.image_index)
        }
    }

    pub fn selected_custom_image(&self) -> Option<&CustomImage> {
        self.custom_images.get(self.custom_image_index)
    }

    pub fn selected_target(&self) -> Option<&InstallTarget> {
        self.devices.get(self.target_index)
    }

    pub fn is_target_expanded(&self, target_id: &str) -> bool {
        self.expanded_targets.contains(target_id)
    }

    pub fn progress(&self) -> Option<&ProgressState> {
        self.progress.as_ref()
    }

    pub fn install_elapsed(&self) -> Option<Duration> {
        self.install_started_at.map(|started| started.elapsed())
    }

    pub fn install_rate_bytes_per_second(&self) -> Option<u64> {
        let elapsed = self.install_elapsed()?.as_secs().max(1);
        let current = self.progress.as_ref()?.current;
        Some(current / elapsed)
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn warning(&self) -> Option<&str> {
        self.warning.as_deref()
    }

    pub fn has_warning(&self) -> bool {
        self.warning.is_some() && self.screen != Screen::ProxyConfig
    }

    pub fn proxy_input(&self) -> &str {
        &self.proxy_input
    }

    pub fn memory_check(&self) -> Option<MemoryCheck> {
        self.selected_image().map(check_image_memory)
    }

    pub fn runner(&self) -> &RunnerGame {
        &self.runner
    }

    pub fn system_status(&self) -> &SystemStatus {
        &self.system_status
    }

    pub fn can_quit(&self) -> bool {
        self.screen != Screen::Installing
    }

    pub fn next(&mut self) {
        match self.screen {
            Screen::ImageSelect => {
                self.image_index = next_index(self.image_index, self.catalog.images.len());
            }
            Screen::CustomImageSelect => {
                self.custom_image_index =
                    next_index(self.custom_image_index, self.custom_images.len());
            }
            Screen::ProxyConfig => {}
            Screen::TargetSelect => {
                self.target_index = next_index(self.target_index, self.devices.len());
            }
            _ => {}
        }
    }

    pub fn previous(&mut self) {
        match self.screen {
            Screen::ImageSelect => {
                self.image_index = previous_index(self.image_index, self.catalog.images.len());
            }
            Screen::CustomImageSelect => {
                self.custom_image_index =
                    previous_index(self.custom_image_index, self.custom_images.len());
            }
            Screen::ProxyConfig => {}
            Screen::TargetSelect => {
                self.target_index = previous_index(self.target_index, self.devices.len());
            }
            _ => {}
        }
    }

    pub fn expand_target(&mut self) {
        if self.screen != Screen::TargetSelect {
            return;
        }
        let Some(target) = self.selected_target() else {
            return;
        };
        if !target.partitions.is_empty() {
            let target_id = target.id.clone();
            let target_name = target.name.clone();
            self.expanded_targets.insert(target_id);
            self.status = format!("Showing partitions for {target_name}.");
        }
    }

    pub fn collapse_target(&mut self) {
        if self.screen != Screen::TargetSelect {
            return;
        }
        let Some(target) = self.selected_target() else {
            return;
        };
        let target_id = target.id.clone();
        let target_name = target.name.clone();
        if self.expanded_targets.remove(&target_id) {
            self.status = format!("Hiding partitions for {target_name}.");
        }
    }

    pub fn back(&mut self) {
        match self.screen {
            Screen::CustomImageSelect => {
                self.screen = Screen::ImageSelect;
                self.status = "Choose an operating system image.".to_string();
            }
            Screen::TargetSelect => {
                if self.active_image_is_custom {
                    self.screen = Screen::CustomImageSelect;
                    self.status = "Choose a custom image from attached media.".to_string();
                } else {
                    self.screen = Screen::ImageSelect;
                    self.status = "Choose an operating system image.".to_string();
                }
            }
            Screen::Confirm => {
                self.screen = Screen::TargetSelect;
                self.status = "Choose target media.".to_string();
            }
            Screen::Complete | Screen::Error => {
                self.screen = Screen::ImageSelect;
                self.active_image = None;
                self.active_image_is_custom = false;
                self.progress = None;
                self.install_started_at = None;
                self.status = "Choose an operating system image.".to_string();
            }
            Screen::ProxyConfig => self.cancel_proxy_config(),
            _ => {}
        }
    }

    pub fn activate_selected(&mut self) {
        match self.screen {
            Screen::ImageSelect => {
                let Some(image) = self.selected_catalog_image().cloned() else {
                    self.status = "No image selected.".to_string();
                    return;
                };
                if is_custom_placeholder(&image) {
                    self.refresh_custom_images();
                    self.screen = Screen::CustomImageSelect;
                    self.status = "Choose a custom image from attached media.".to_string();
                } else {
                    self.active_image = Some(image);
                    self.active_image_is_custom = false;
                    self.screen = Screen::TargetSelect;
                    self.status = "Choose target media.".to_string();
                }
            }
            Screen::CustomImageSelect => {
                let Some(custom_image) = self.selected_custom_image().cloned() else {
                    self.status =
                        "No custom images found. Attach media and press r to rescan.".to_string();
                    return;
                };
                self.active_image = Some(custom_image.into_image_entry());
                self.active_image_is_custom = true;
                self.screen = Screen::TargetSelect;
                self.status = "Choose target media.".to_string();
            }
            Screen::TargetSelect => {
                self.screen = Screen::Confirm;
                self.status = "Confirm destructive write.".to_string();
            }
            Screen::Confirm => self.start_install(),
            Screen::Complete | Screen::Error => self.back(),
            Screen::ProxyConfig => self.apply_proxy_config(),
            _ => {}
        }
    }

    pub fn dismiss_warning(&mut self) {
        self.warning = None;
    }

    pub fn start_proxy_config(&mut self) {
        self.warning = None;
        self.proxy_input = self.proxy_url.clone().unwrap_or_default();
        self.screen = Screen::ProxyConfig;
        self.status = "Enter proxy URL and press Enter to retry the online catalog.".to_string();
    }

    pub fn cancel_proxy_config(&mut self) {
        self.screen = Screen::ImageSelect;
        self.status = "Choose an operating system image.".to_string();
    }

    pub fn proxy_push(&mut self, ch: char) {
        if !ch.is_control() {
            self.proxy_input.push(ch);
        }
    }

    pub fn proxy_backspace(&mut self) {
        self.proxy_input.pop();
    }

    pub fn apply_proxy_config(&mut self) {
        let proxy = self.proxy_input.trim().to_string();
        let proxy = (!proxy.is_empty()).then_some(proxy);

        match manifest::load_catalog_with_proxy(&self.manifest_source, proxy.as_deref()) {
            Ok(catalog) => {
                self.proxy_url = proxy;
                self.replace_catalog(catalog);
                self.screen = Screen::ImageSelect;
                self.status = "Online OS catalog loaded.".to_string();
                self.warning = None;
            }
            Err(error) => {
                self.warning = Some(format!(
                    "Could not load the online catalog with this proxy. You can adjust the proxy and try again, or press Esc to continue with custom local images.\n\n{error:#}"
                ));
            }
        }
    }

    pub fn refresh_devices(&mut self) {
        if self.screen == Screen::CustomImageSelect {
            self.refresh_custom_images();
            return;
        }

        match list_devices(DeviceMode::from(self.run_mode), None) {
            Ok(devices) => {
                self.devices = devices;
                self.target_index = self.target_index.min(self.devices.len().saturating_sub(1));
                let target_ids = self
                    .devices
                    .iter()
                    .map(|target| target.id.clone())
                    .collect::<BTreeSet<_>>();
                self.expanded_targets
                    .retain(|target_id| target_ids.contains(target_id));
                self.status = "Storage targets refreshed.".to_string();
            }
            Err(error) => {
                self.status = format!("Could not refresh targets: {error:#}");
            }
        }
    }

    pub fn refresh_custom_images(&mut self) {
        match scan_custom_images() {
            Ok(images) => {
                self.custom_images = images;
                self.custom_image_index = self
                    .custom_image_index
                    .min(self.custom_images.len().saturating_sub(1));
                self.status = if self.custom_images.is_empty() {
                    "No custom images found. Attach media and press r to rescan.".to_string()
                } else {
                    format!("Found {} custom image(s).", self.custom_images.len())
                };
            }
            Err(error) => {
                self.status = format!("Could not scan attached media: {error:#}");
            }
        }
    }

    pub fn poll_install_events(&mut self) {
        let Some(rx) = self.install_rx.take() else {
            return;
        };

        let mut keep_rx = true;
        while let Ok(event) = rx.try_recv() {
            match event {
                InstallEvent::Phase(phase) => {
                    self.status = phase;
                }
                InstallEvent::Progress {
                    phase,
                    current,
                    total,
                    source_current,
                    source_total,
                } => {
                    self.status = phase.clone();
                    self.progress = Some(ProgressState {
                        phase,
                        current,
                        total,
                        source_current,
                        source_total,
                    });
                }
                InstallEvent::Complete(message) => {
                    self.status = message;
                    self.screen = Screen::Complete;
                    keep_rx = false;
                }
                InstallEvent::Failed(message) => {
                    self.status = message;
                    self.screen = Screen::Error;
                    keep_rx = false;
                }
            }
        }

        if keep_rx {
            self.install_rx = Some(rx);
        }
    }

    pub fn tick_runner(&mut self) {
        if self.screen == Screen::Installing {
            self.runner.tick();
        }
    }

    pub fn refresh_system_status_if_due(&mut self) {
        let now = Instant::now();
        let should_refresh = self
            .system_status_refreshed_at
            .map(|last| now.duration_since(last) >= Duration::from_secs(1))
            .unwrap_or(true);
        if should_refresh {
            self.system_status = read_system_status();
            self.system_status_refreshed_at = Some(now);
        }
    }

    pub fn runner_jump_or_restart(&mut self) {
        if self.screen == Screen::Installing {
            self.runner.jump_or_restart();
        }
    }

    fn start_install(&mut self) {
        let Some(image) = self.selected_image().cloned() else {
            self.status = "No image selected.".to_string();
            return;
        };
        let Some(target) = self.selected_target().cloned() else {
            self.status = "No target selected.".to_string();
            return;
        };

        let memory = check_image_memory(&image);
        if memory.enough == Some(false) {
            self.screen = Screen::Error;
            self.status = format!(
                "Not enough available RAM for the installer working set. {}. TOBI streams images, so the full image does not need to fit in RAM.",
                memory.summary()
            );
            return;
        }

        self.screen = Screen::Installing;
        self.runner = RunnerGame::default();
        self.install_started_at = Some(Instant::now());
        self.status = "Starting install.".to_string();
        self.progress = Some(ProgressState {
            phase: "Starting".to_string(),
            current: 0,
            total: image.extract_size,
            source_current: Some(0),
            source_total: image.image_download_size,
        });
        let reboot_after_install =
            self.run_mode == RunMode::Live && target.kind != TargetKind::File;
        self.install_rx = Some(start_install(InstallRequest {
            image,
            target,
            run_mode: self.run_mode,
            allow_write: self.allow_write,
            proxy_url: self.proxy_url.clone(),
            reboot_after_install,
        }));
    }

    fn replace_catalog(&mut self, mut catalog: Catalog) {
        catalog.images.push(custom_placeholder());
        sort_images_for_display(&mut catalog.images);
        self.catalog = catalog;
        self.image_index = self
            .catalog
            .images
            .iter()
            .position(|image| image.recommended)
            .unwrap_or(0);
        self.active_image = None;
        self.active_image_is_custom = false;
    }
}

impl Default for RunnerGame {
    fn default() -> Self {
        Self {
            tick: 0,
            score: 0,
            runner_y: 0,
            velocity: 0,
            obstacles: Vec::new(),
            crashed: false,
        }
    }
}

impl RunnerGame {
    pub fn score(&self) -> u64 {
        self.score
    }

    pub fn runner_y(&self) -> i16 {
        self.runner_y
    }

    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }

    pub fn crashed(&self) -> bool {
        self.crashed
    }

    fn tick(&mut self) {
        if self.crashed {
            return;
        }

        self.tick = self.tick.saturating_add(1);
        self.score = self.score.saturating_add(1);

        if self.runner_y > 0 || self.velocity > 0 {
            self.runner_y += self.velocity;
            self.velocity -= 1;
            if self.runner_y <= 0 {
                self.runner_y = 0;
                self.velocity = 0;
            }
        }

        for obstacle in &mut self.obstacles {
            obstacle.x -= 1;
        }
        self.obstacles
            .retain(|obstacle| obstacle.x + obstacle.width > 0);

        if self.should_spawn_obstacle() {
            let kind = if (self.tick / 29).is_multiple_of(2) {
                ObstacleKind::Cactus
            } else {
                ObstacleKind::Rock
            };
            self.obstacles.push(Obstacle {
                x: 140,
                width: 2 + ((self.tick / 37) % 2) as i16,
                kind,
            });
        }

        self.crashed = self.obstacles.iter().any(|obstacle| {
            obstacle.x <= 5 && obstacle.x + obstacle.width >= 4 && self.runner_y == 0
        });
    }

    fn jump_or_restart(&mut self) {
        if self.crashed {
            *self = Self::default();
            return;
        }

        if self.runner_y == 0 {
            self.velocity = 5;
        }
    }

    fn should_spawn_obstacle(&self) -> bool {
        if self.tick < 8 {
            return false;
        }
        let interval = 28_u64.saturating_sub((self.score / 180).min(10));
        self.tick % interval == 0
    }
}

fn next_index(current: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (current + 1) % len }
}

fn previous_index(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (current + len - 1) % len
    }
}

fn sort_images_for_display(images: &mut [ImageEntry]) {
    images.sort_by(|a, b| {
        category_rank(a.category_label())
            .cmp(&category_rank(b.category_label()))
            .then_with(|| a.category_label().cmp(b.category_label()))
            .then_with(|| b.recommended.cmp(&a.recommended))
            .then_with(|| a.name.cmp(&b.name))
    });
}

fn category_rank(category: &str) -> u8 {
    match category {
        "Yocto" => 0,
        "Debian" => 1,
        "Virtualization" => 2,
        "Custom" => 99,
        _ => 50,
    }
}

fn read_system_status() -> SystemStatus {
    SystemStatus {
        time: command_output("date", &["-u", "+%Y-%m-%d %H:%M:%S UTC"])
            .unwrap_or_else(|| "time unavailable".to_string()),
        ip: read_primary_ipv4().unwrap_or_else(|| "no IPv4".to_string()),
    }
}

fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn read_primary_ipv4() -> Option<String> {
    let output = Command::new("ip")
        .args(["-4", "-o", "addr", "show", "scope", "global"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    text.lines().find_map(|line| {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let iface = fields.get(1)?.trim_end_matches(':');
        let addr = fields.get(3)?.split('/').next()?;
        Some(format!("{iface} {addr}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{BoardSource, DetectedBoard};
    use crate::device::{InstallTarget, TargetKind};
    use crate::manifest::{Catalog, ImageEntry, ImageFormat};

    #[test]
    fn enter_advances_from_image_to_target() {
        let mut app = App::new(
            catalog(),
            board(),
            targets(),
            RunMode::Mock,
            false,
            "sample/catalog.json".to_string(),
            None,
            None,
        );
        app.activate_selected();
        assert_eq!(app.screen(), Screen::TargetSelect);
    }

    #[test]
    fn recommended_image_is_selected_first() {
        let app = App::new(
            catalog(),
            board(),
            targets(),
            RunMode::Mock,
            false,
            "sample/catalog.json".to_string(),
            None,
            None,
        );
        assert_eq!(app.selected_image().expect("image").name, "Image");
    }

    #[test]
    fn custom_placeholder_is_added() {
        let app = App::new(
            catalog(),
            board(),
            targets(),
            RunMode::Mock,
            false,
            "sample/catalog.json".to_string(),
            None,
            None,
        );
        assert!(app.catalog().images.iter().any(is_custom_placeholder));
    }

    fn catalog() -> Catalog {
        Catalog {
            schema_version: 1,
            generated_at: None,
            devices: vec![],
            images: vec![ImageEntry {
                id: "image".to_string(),
                name: "Image".to_string(),
                description: "Test image".to_string(),
                devices: vec![],
                category: Some("Yocto".to_string()),
                recommended: true,
                version: "1".to_string(),
                release_date: "2026-05-07".to_string(),
                channel: "dev".to_string(),
                url: "mock://image".to_string(),
                format: ImageFormat::Raw,
                image_download_sha256: None,
                extract_sha256: None,
                image_download_size: Some(1),
                extract_size: Some(1),
                bmap_url: None,
                signature_url: None,
            }],
        }
    }

    fn board() -> DetectedBoard {
        DetectedBoard {
            id: Some("sk-am62p-lp".to_string()),
            name: "SK-AM62P-LP".to_string(),
            compatible: vec!["ti,am62pxx-evm".to_string()],
            source: BoardSource::Mock,
        }
    }

    fn targets() -> Vec<InstallTarget> {
        vec![InstallTarget {
            id: "target".to_string(),
            name: "Target".to_string(),
            path: "/dev/mmcblk0".into(),
            size_bytes: Some(1),
            kind: TargetKind::Emmc,
            removable: false,
            partitions: Vec::new(),
            warning: None,
        }]
    }
}
