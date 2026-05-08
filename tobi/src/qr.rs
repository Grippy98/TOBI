use qrcode::render::unicode;
use qrcode::{EcLevel, QrCode};

pub fn render_qr(data: &str, max_width: u16, max_height: u16) -> Option<Vec<String>> {
    let code = QrCode::with_error_correction_level(data.as_bytes(), EcLevel::M).ok()?;
    let rendered = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .quiet_zone(true)
        .build();

    let lines = rendered
        .lines()
        .map(str::to_string)
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();

    if lines.is_empty() {
        return None;
    }

    let width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    if width > usize::from(max_width) || lines.len() > usize::from(max_height) {
        return None;
    }

    Some(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_short_url_when_it_fits() {
        let lines = render_qr("https://www.ti.com/sitara", 80, 40).expect("qr code");
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|line| line.chars().count() <= 80));
    }

    #[test]
    fn returns_none_when_viewport_is_too_small() {
        assert!(render_qr("https://www.ti.com/sitara", 8, 4).is_none());
    }
}
