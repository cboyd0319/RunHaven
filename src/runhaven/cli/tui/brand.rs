//! RunHaven logo rendering for the Home header.
//!
//! The Home header uses the project logo from `docs/assets/logo.png`. Cubby
//! stays the animated pet; this module only handles the static brand mark.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use ratatui::layout::Rect;
use ratatui::text::Line;

use super::codex::ambient;
use super::codex::ambient::AmbientImageDraw;
use super::codex::image_protocol::ImageProtocol;
use super::pet::{PetSize, half_block_lines};

const LOGO_BYTES: &[u8] = include_bytes!("../../../../docs/assets/logo.png");
pub(crate) const LOGO_IMAGE_ID: u32 = 0x52554c;
const TERMINAL_ROW_HEIGHT_PX: u16 = 15;
const TERMINAL_CELL_ASPECT: f64 = 0.52;
const MIN_LOGO_ROWS: u16 = 5;
const MAX_LOGO_ROWS: u16 = 14;

#[derive(Debug)]
pub(crate) struct BrandLogo {
    path: PathBuf,
    half_blocks: HashMap<LogoHalfBlockKey, Vec<Line<'static>>>,
}

impl BrandLogo {
    pub(crate) fn load() -> Result<Self> {
        let path = materialize_logo()?;
        Ok(Self {
            path,
            half_blocks: HashMap::new(),
        })
    }

    pub(crate) fn size_for_area(&self, max_rows: u16, max_columns: u16) -> Option<PetSize> {
        (MIN_LOGO_ROWS..=max_rows.min(MAX_LOGO_ROWS))
            .rev()
            .find_map(|rows| {
                let columns = logo_columns_for_rows(rows);
                (columns <= max_columns).then_some(PetSize { columns, rows })
            })
    }

    pub(crate) fn lines(
        &mut self,
        size: PetSize,
        color_enabled: bool,
    ) -> Result<Vec<Line<'static>>> {
        let key = LogoHalfBlockKey {
            rows: size.rows,
            color_enabled,
        };
        if let Some(lines) = self.half_blocks.get(&key) {
            return Ok(lines.clone());
        }
        let lines = half_block_lines(&self.path, size, color_enabled)?;
        self.half_blocks.insert(key, lines.clone());
        Ok(lines)
    }

    pub(crate) fn draw_request(
        &self,
        area: Rect,
        protocol: ImageProtocol,
    ) -> Option<AmbientImageDraw> {
        ambient::fixed_area_draw_request(
            self.path.clone(),
            protocol,
            cache_root().join("sixel"),
            area,
            area.height.saturating_mul(TERMINAL_ROW_HEIGHT_PX),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct LogoHalfBlockKey {
    rows: u16,
    color_enabled: bool,
}

fn logo_columns_for_rows(rows: u16) -> u16 {
    (f64::from(rows) / TERMINAL_CELL_ASPECT).round().max(1.0) as u16
}

fn materialize_logo() -> Result<PathBuf> {
    let path = cache_root().join("logo.png");
    fs::create_dir_all(path.parent().context("logo cache path has no parent")?)
        .with_context(|| format!("create logo cache parent for {}", path.display()))?;
    if fs::read(&path).is_ok_and(|current| current == LOGO_BYTES) {
        return Ok(path);
    }
    fs::write(&path, LOGO_BYTES).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn cache_root() -> PathBuf {
    std::env::temp_dir().join("runhaven-tui-brand-v1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_loads_from_project_asset() {
        let mut logo = BrandLogo::load().unwrap();
        let size = logo.size_for_area(8, 80).unwrap();
        assert_eq!(size.rows, 8);
        assert!(size.columns <= 80);
        assert_eq!(logo.lines(size, true).unwrap().len(), 8);
    }

    #[test]
    fn logo_draw_request_uses_banner_area() {
        let logo = BrandLogo::load().unwrap();
        let request = logo
            .draw_request(Rect::new(2, 3, 16, 8), ImageProtocol::Kitty)
            .unwrap();

        assert_eq!(request.x, 2);
        assert_eq!(request.y, 3);
        assert_eq!(request.columns, 16);
        assert_eq!(request.rows, 8);
    }
}
