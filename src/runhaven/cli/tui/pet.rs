//! Cubby animated pet integration.
//!
//! The structure here follows Codex's `tui/src/pets/ambient.rs` and
//! `tui/src/pets/mod.rs`: a validated Codex pet package is loaded into the
//! vendored model, frames are extracted from the atlas, animation state picks a
//! sprite index, and terminal image protocols are emitted after ratatui draws.
//! RunHaven-specific code only supplies the available launcher rectangle and
//! the Cubby asset package.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use image::imageops::FilterType;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use super::codex::ambient;
use super::codex::ambient::AmbientImageDraw;
use super::codex::animation::current_animation_frame;
use super::codex::frames;
#[cfg(not(test))]
use super::codex::image_protocol;
use super::codex::image_protocol::ImageProtocol;
use super::codex::model::Pet;
use super::theme::TuiSettings;

const CUBBY_PET_JSON: &[u8] = include_bytes!("assets/cubby/pet.json");
const CUBBY_SPRITESHEET: &[u8] = include_bytes!("assets/cubby/spritesheet.webp");
pub(crate) const PET_IMAGE_ID: u32 = 0x525550;

#[derive(Debug)]
pub(crate) struct CubbyPet {
    pet: Pet,
    frames: Vec<PathBuf>,
    sixel_dir: PathBuf,
    half_blocks: HashMap<HalfBlockKey, Vec<Line<'static>>>,
}

impl CubbyPet {
    pub(crate) fn load() -> Result<Self> {
        let asset_dir = materialize_assets()?;
        let pet = Pet::load_with_codex_home(
            asset_dir
                .to_str()
                .context("Cubby pet asset path is not valid UTF-8")?,
            None,
        )
        .context("load Cubby pet")?;
        let cache_dir = cache_root()
            .join("frame-cache")
            .join(&pet.id)
            .join(pet.frame_cache_key()?);
        let frame_dir = cache_dir.join("frames");
        let sixel_dir = cache_dir.join("sixel");
        let frames = frames::prepare_png_frames(&pet, &frame_dir)?;

        Ok(Self {
            pet,
            frames,
            sixel_dir,
            half_blocks: HashMap::new(),
        })
    }

    pub(crate) fn idle_lines(
        &mut self,
        size: PetSize,
        color_enabled: bool,
        elapsed: Duration,
        animated: bool,
    ) -> Result<Vec<Line<'static>>> {
        let sprite_index = self.current_idle_sprite_index(elapsed, animated);
        let key = HalfBlockKey {
            sprite_index,
            rows: size.rows,
            color_enabled,
        };
        if let Some(lines) = self.half_blocks.get(&key) {
            return Ok(lines.clone());
        }

        let frame = self
            .frame_path_for_sprite_index(sprite_index)
            .context("Cubby pet has no frame for current animation")?;
        let lines = half_block_lines(&frame, size, color_enabled)?;
        self.half_blocks.insert(key, lines.clone());
        Ok(lines)
    }

    pub(crate) fn ambient_size(&self) -> PetSize {
        let size = ambient::ambient_pet_image_size(self.pet.frame_width, self.pet.frame_height);
        PetSize {
            columns: size.columns,
            rows: size.rows,
        }
    }

    pub(crate) fn ambient_lines(
        &mut self,
        color_enabled: bool,
        elapsed: Duration,
        animated: bool,
    ) -> Result<Vec<Line<'static>>> {
        self.idle_lines(self.ambient_size(), color_enabled, elapsed, animated)
    }

    pub(crate) fn ambient_area(&self, area: Rect, composer_bottom_y: u16) -> Option<Rect> {
        ambient::ambient_pet_cell_area(
            area,
            composer_bottom_y,
            self.pet.frame_width,
            self.pet.frame_height,
        )
    }

    pub(crate) fn ambient_draw_request(
        &self,
        area: Rect,
        composer_bottom_y: u16,
        elapsed: Duration,
        animated: bool,
        protocol: ImageProtocol,
    ) -> Option<AmbientImageDraw> {
        ambient::ambient_pet_draw_request(
            self.frame_path_for_sprite_index(self.current_idle_sprite_index(elapsed, animated))?,
            protocol,
            self.sixel_dir.clone(),
            area,
            composer_bottom_y,
            self.pet.frame_width,
            self.pet.frame_height,
        )
    }

    pub(crate) fn current_idle_sprite_index(&self, elapsed: Duration, animated: bool) -> usize {
        let Some(animation) = self.pet.animations.get("idle") else {
            return 0;
        };
        if !animated {
            return animation
                .frames
                .first()
                .map_or(0, |frame| frame.sprite_index);
        }
        current_animation_frame(animation, elapsed).map_or(0, |frame| frame.sprite_index)
    }

    fn frame_path_for_sprite_index(&self, sprite_index: usize) -> Option<PathBuf> {
        self.frames
            .get(sprite_index.min(self.frames.len().saturating_sub(1)))
            .cloned()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PetSize {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct HalfBlockKey {
    sprite_index: usize,
    rows: u16,
    color_enabled: bool,
}

#[cfg(not(test))]
pub(crate) fn detect_image_protocol(settings: TuiSettings) -> Option<ImageProtocol> {
    if settings.color_enabled && !settings.line_mode {
        image_protocol::detect_pet_image_support().protocol()
    } else {
        None
    }
}

#[cfg(test)]
pub(crate) fn detect_image_protocol(_settings: TuiSettings) -> Option<ImageProtocol> {
    None
}

pub(crate) fn half_block_lines(
    frame: &Path,
    size: PetSize,
    color_enabled: bool,
) -> Result<Vec<Line<'static>>> {
    let image = image::open(frame)
        .with_context(|| format!("read {}", frame.display()))?
        .resize_exact(
            u32::from(size.columns),
            u32::from(size.rows.saturating_mul(2)),
            FilterType::Lanczos3,
        )
        .to_rgba8();

    let mut lines = Vec::with_capacity(size.rows as usize);
    for row in 0..size.rows {
        let mut spans = Vec::with_capacity(size.columns as usize);
        for col in 0..size.columns {
            let top = pixel_color(*image.get_pixel(u32::from(col), u32::from(row * 2)));
            let bottom = pixel_color(*image.get_pixel(u32::from(col), u32::from(row * 2 + 1)));
            let (symbol, style) = match (top, bottom, color_enabled) {
                (None, None, _) => (" ", Style::default()),
                (Some(_), None, false) => ("\u{2580}", Style::default()),
                (None, Some(_), false) => ("\u{2584}", Style::default()),
                (Some(_), Some(_), false) => ("\u{2588}", Style::default()),
                (Some(top), None, true) => ("\u{2580}", Style::default().fg(top)),
                (None, Some(bottom), true) => ("\u{2584}", Style::default().fg(bottom)),
                (Some(top), Some(bottom), true) => {
                    ("\u{2580}", Style::default().fg(top).bg(bottom))
                }
            };
            spans.push(Span::styled(symbol, style));
        }
        lines.push(Line::from(spans));
    }
    Ok(lines)
}

fn pixel_color(pixel: image::Rgba<u8>) -> Option<Color> {
    let [red, green, blue, alpha] = pixel.0;
    (alpha >= 16).then_some(Color::Rgb(red, green, blue))
}

fn materialize_assets() -> Result<PathBuf> {
    let dir = cache_root().join("assets").join("cubby");
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    write_if_changed(&dir.join("pet.json"), CUBBY_PET_JSON)?;
    write_if_changed(&dir.join("spritesheet.webp"), CUBBY_SPRITESHEET)?;
    Ok(dir)
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<()> {
    if fs::read(path).is_ok_and(|current| current == bytes) {
        return Ok(());
    }
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

fn cache_root() -> PathBuf {
    std::env::temp_dir().join("runhaven-tui-cubby-pet-v1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubby_pet_loads_codex_package_contract() {
        let pet = CubbyPet::load().unwrap();

        assert_eq!(pet.pet.id, "cubby");
        assert_eq!(pet.pet.frame_width, 192);
        assert_eq!(pet.pet.frame_height, 208);
        assert_eq!(pet.pet.columns, 8);
        assert_eq!(pet.pet.rows, 9);
        assert_eq!(pet.frames.len(), 72);
        assert!(pet.pet.animations.contains_key("idle"));
    }

    #[test]
    fn idle_animation_uses_codex_source_timing() {
        let pet = CubbyPet::load().unwrap();

        assert_eq!(
            pet.current_idle_sprite_index(Duration::from_millis(0), true),
            0
        );
        assert_eq!(
            pet.current_idle_sprite_index(Duration::from_millis(1680), true),
            1
        );
        assert_eq!(
            pet.current_idle_sprite_index(Duration::from_millis(1680), false),
            0
        );
    }

    #[test]
    fn half_block_lines_render_without_color_for_no_color_mode() {
        let mut pet = CubbyPet::load().unwrap();

        let lines = pet
            .idle_lines(
                PetSize {
                    columns: 8,
                    rows: 6,
                },
                false,
                Duration::ZERO,
                true,
            )
            .unwrap();

        assert_eq!(lines.len(), 6);
        assert!(lines.iter().flat_map(|line| &line.spans).any(|span| {
            let content = span.content.as_ref();
            content == "\u{2580}" || content == "\u{2584}" || content == "\u{2588}"
        }));
        for span in lines.iter().flat_map(|line| &line.spans) {
            assert_eq!(span.style.fg, None);
            assert_eq!(span.style.bg, None);
        }
    }

    #[test]
    fn ambient_size_uses_codex_target_height() {
        let pet = CubbyPet::load().unwrap();
        let size = pet.ambient_size();

        assert_eq!(size.rows, 5);
        assert_eq!(size.columns, 9);
    }

    #[test]
    fn ambient_draw_request_uses_codex_anchor_contract() {
        let pet = CubbyPet::load().unwrap();
        let request = pet
            .ambient_draw_request(
                Rect::new(2, 4, 40, 20),
                22,
                Duration::from_millis(1680),
                true,
                ImageProtocol::Kitty,
            )
            .unwrap();

        assert_eq!(request.x, 33);
        assert_eq!(request.y, 16);
        assert_eq!(request.clear_top_y, 4);
        assert_eq!(request.columns, 9);
        assert_eq!(request.rows, 5);
        assert!(request.frame.ends_with("frame_001.png"));
    }
}
