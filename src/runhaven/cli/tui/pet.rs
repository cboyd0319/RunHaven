//! Cubby animated pet integration.
//!
//! The structure here follows Codex's `tui/src/pets/ambient.rs` and
//! `tui/src/pets/mod.rs`: a validated Codex pet package is loaded into the
//! vendored model, frames are extracted from the atlas, animation state picks a
//! sprite index, and terminal image protocols are emitted after ratatui draws.
//! RunHaven-specific code only chooses where the pet fits in the launcher.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use image::imageops::FilterType;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use super::codex::animation::current_animation_frame;
use super::codex::frames;
use super::codex::image_protocol;
use super::codex::image_protocol::ImageProtocol;
use super::codex::model::Pet;
use super::theme::TuiSettings;

const CUBBY_PET_JSON: &[u8] = include_bytes!("assets/cubby/pet.json");
const CUBBY_SPRITESHEET: &[u8] = include_bytes!("assets/cubby/spritesheet.webp");
const PET_IMAGE_ID: u32 = 0x525548;
const TERMINAL_ROW_HEIGHT_PX: u16 = 15;
const TERMINAL_CELL_ASPECT: f64 = 0.52;
const MIN_PET_ROWS: u16 = 4;

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

    pub(crate) fn size_for_area(&self, max_rows: u16, max_columns: u16) -> Option<PetSize> {
        (MIN_PET_ROWS..=max_rows).rev().find_map(|rows| {
            let columns = self.columns_for_rows(rows);
            (columns <= max_columns).then_some(PetSize { columns, rows })
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

    pub(crate) fn draw_request(
        &self,
        area: Rect,
        elapsed: Duration,
        animated: bool,
        protocol: ImageProtocol,
    ) -> Option<PetImageDraw> {
        if area.width == 0 || area.height == 0 {
            return None;
        }
        Some(PetImageDraw {
            frame: self
                .frame_path_for_sprite_index(self.current_idle_sprite_index(elapsed, animated))?,
            protocol,
            x: area.x,
            y: area.y,
            clear_top_y: area.y,
            columns: area.width,
            rows: area.height,
            height_px: area.height.saturating_mul(TERMINAL_ROW_HEIGHT_PX),
            sixel_dir: self.sixel_dir.clone(),
        })
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

    fn columns_for_rows(&self, rows: u16) -> u16 {
        let aspect = f64::from(self.pet.frame_height) / f64::from(self.pet.frame_width)
            * TERMINAL_CELL_ASPECT;
        (f64::from(rows) / aspect).round().max(1.0) as u16
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PetSize {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PetImageDraw {
    frame: PathBuf,
    protocol: ImageProtocol,
    x: u16,
    y: u16,
    clear_top_y: u16,
    columns: u16,
    rows: u16,
    height_px: u16,
    sixel_dir: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct HalfBlockKey {
    sprite_index: usize,
    rows: u16,
    color_enabled: bool,
}

#[cfg(not(test))]
pub(crate) fn detect_image_protocol(settings: TuiSettings) -> Option<ImageProtocol> {
    if settings.pet_enabled && settings.color_enabled && !settings.line_mode {
        image_protocol::detect_pet_image_support().protocol()
    } else {
        None
    }
}

#[cfg(test)]
pub(crate) fn detect_image_protocol(_settings: TuiSettings) -> Option<ImageProtocol> {
    None
}

#[derive(Debug)]
pub(crate) enum PetImageRenderError {
    Terminal(std::io::Error),
    Asset(anyhow::Error),
}

impl std::fmt::Display for PetImageRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Terminal(err) => write!(f, "terminal image write failed: {err}"),
            Self::Asset(err) => write!(f, "pet image asset unavailable: {err}"),
        }
    }
}

impl std::error::Error for PetImageRenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Terminal(err) => Some(err),
            Self::Asset(err) => Some(err.as_ref()),
        }
    }
}

impl From<std::io::Error> for PetImageRenderError {
    fn from(err: std::io::Error) -> Self {
        Self::Terminal(err)
    }
}

#[derive(Debug, Default)]
pub(crate) struct PetImageRenderState {
    last_sixel_clear_area: Option<SixelClearArea>,
    last_protocol: Option<ImageProtocol>,
}

pub(crate) fn render_pet_image(
    writer: &mut impl Write,
    state: &mut PetImageRenderState,
    request: Option<PetImageDraw>,
) -> std::result::Result<(), PetImageRenderError> {
    use ratatui::crossterm::cursor::MoveTo;
    use ratatui::crossterm::cursor::RestorePosition;
    use ratatui::crossterm::cursor::SavePosition;
    use ratatui::crossterm::queue;

    let Some(request) = request else {
        if state.last_protocol.take().is_some_and(is_kitty_protocol) {
            write!(
                writer,
                "{}",
                image_protocol::kitty_delete_image(PET_IMAGE_ID)
            )?;
        }
        if let Some(area) = state.last_sixel_clear_area.take() {
            queue!(writer, SavePosition)?;
            clear_sixel_area(writer, area)?;
            queue!(writer, RestorePosition)?;
        }
        writer.flush()?;
        return Ok(());
    };

    if state.last_protocol.take().is_some_and(is_kitty_protocol)
        || is_kitty_protocol(request.protocol)
    {
        write!(
            writer,
            "{}",
            image_protocol::kitty_delete_image(PET_IMAGE_ID)
        )?;
    }
    state.last_protocol = Some(request.protocol);

    let payload = match request.protocol {
        ImageProtocol::Kitty => PetImagePayload::Text(
            image_protocol::kitty_transmit_png_with_id(
                &request.frame,
                request.columns,
                request.rows,
                Some(PET_IMAGE_ID),
            )
            .map_err(PetImageRenderError::Asset)?,
        ),
        ImageProtocol::KittyLocalFile => PetImagePayload::Text(
            image_protocol::kitty_transmit_png_file_with_id(
                &request.frame,
                request.columns,
                request.rows,
                Some(PET_IMAGE_ID),
            )
            .map_err(PetImageRenderError::Asset)?,
        ),
        ImageProtocol::Sixel => {
            let path =
                image_protocol::sixel_frame(&request.frame, &request.sixel_dir, request.height_px)
                    .map_err(PetImageRenderError::Asset)?;
            let sixel = fs::read(&path)
                .with_context(|| format!("read {}", path.display()))
                .map_err(PetImageRenderError::Asset)?;
            PetImagePayload::Bytes(sixel)
        }
    };

    queue!(writer, SavePosition)?;
    let current_sixel_clear_area = if matches!(request.protocol, ImageProtocol::Sixel) {
        Some(SixelClearArea::from(&request))
    } else {
        None
    };
    if let Some(previous_area) = state.last_sixel_clear_area.take()
        && Some(previous_area) != current_sixel_clear_area
    {
        clear_sixel_area(writer, previous_area)?;
    }
    if let Some(area) = current_sixel_clear_area {
        clear_sixel_area(writer, area)?;
        state.last_sixel_clear_area = Some(area);
    }
    queue!(writer, MoveTo(request.x, request.y))?;
    match payload {
        PetImagePayload::Text(payload) => write!(writer, "{payload}")?,
        PetImagePayload::Bytes(payload) => writer.write_all(&payload)?,
    }
    queue!(writer, RestorePosition)?;
    writer.flush()?;
    Ok(())
}

enum PetImagePayload {
    Text(String),
    Bytes(Vec<u8>),
}

fn half_block_lines(
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

fn is_kitty_protocol(protocol: ImageProtocol) -> bool {
    matches!(
        protocol,
        ImageProtocol::Kitty | ImageProtocol::KittyLocalFile
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SixelClearArea {
    x: u16,
    clear_top_y: u16,
    clear_bottom_y: u16,
    columns: u16,
}

impl From<&PetImageDraw> for SixelClearArea {
    fn from(request: &PetImageDraw) -> Self {
        Self {
            x: request.x,
            clear_top_y: request.clear_top_y,
            clear_bottom_y: request.y.saturating_add(request.rows),
            columns: request.columns,
        }
    }
}

fn clear_sixel_area(writer: &mut impl Write, area: SixelClearArea) -> std::io::Result<()> {
    use ratatui::crossterm::cursor::MoveTo;
    use ratatui::crossterm::queue;

    let blank = " ".repeat(area.columns.into());
    for row in area.clear_top_y..area.clear_bottom_y {
        queue!(writer, MoveTo(area.x, row))?;
        write!(writer, "{blank}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;
    use std::io;

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
    fn size_for_area_respects_terminal_columns() {
        let pet = CubbyPet::load().unwrap();

        let size = pet.size_for_area(12, 80).unwrap();

        assert_eq!(size.rows, 12);
        assert!(size.columns <= 80);
        assert!(pet.size_for_area(12, 1).is_none());
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
    fn draw_request_uses_current_idle_frame_and_banner_area() {
        let pet = CubbyPet::load().unwrap();
        let request = pet
            .draw_request(
                Rect::new(2, 3, 18, 10),
                Duration::from_millis(1680),
                true,
                ImageProtocol::Kitty,
            )
            .unwrap();

        assert_eq!(request.x, 2);
        assert_eq!(request.y, 3);
        assert_eq!(request.columns, 18);
        assert_eq!(request.rows, 10);
        assert!(request.frame.ends_with("frame_001.png"));
    }

    #[test]
    fn pet_image_restores_cursor_after_drawing() {
        let dir = tempfile::tempdir().unwrap();
        let frame = dir.path().join("frame.png");
        fs::write(&frame, b"png").unwrap();
        let request = PetImageDraw {
            frame,
            protocol: ImageProtocol::Kitty,
            x: 2,
            y: 3,
            clear_top_y: 3,
            columns: 4,
            rows: 5,
            height_px: 75,
            sixel_dir: PathBuf::new(),
        };
        let mut output = Vec::new();
        let mut state = PetImageRenderState::default();

        render_pet_image(&mut output, &mut state, Some(request)).unwrap();

        let output = String::from_utf8(output).unwrap();
        let save = output.find("\x1b7").expect("saves cursor position");
        let move_to = output.find("\x1b[4;3H").expect("moves to pet position");
        let image = output.find("cG5n").expect("writes image payload");
        let restore = output.find("\x1b8").expect("restores cursor position");
        assert!(save < move_to);
        assert!(move_to < image);
        assert!(image < restore);
    }

    #[test]
    fn kitty_pet_image_clear_deletes_without_moving_cursor() {
        let mut output = Vec::new();
        let mut state = PetImageRenderState {
            last_protocol: Some(ImageProtocol::Kitty),
            ..Default::default()
        };

        render_pet_image(&mut output, &mut state, None).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("Ga=d,d=I,i=5395784,q=2;"));
        assert!(!output.contains("\x1b7"));
        assert!(!output.contains("\x1b["));
        assert!(!output.contains("\x1b8"));
    }

    #[test]
    fn sixel_pet_image_clear_erases_last_drawn_area() {
        let mut output = Vec::new();
        let mut state = PetImageRenderState {
            last_sixel_clear_area: Some(SixelClearArea {
                x: 2,
                clear_top_y: 1,
                clear_bottom_y: 5,
                columns: 4,
            }),
            ..Default::default()
        };

        render_pet_image(&mut output, &mut state, None).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("\x1b7"));
        assert!(output.contains("\x1b[2;3H    \x1b[3;3H    \x1b[4;3H    \x1b[5;3H    "));
        assert!(output.contains("\x1b8"));
    }

    #[test]
    fn missing_frame_is_an_asset_error() {
        let dir = tempfile::tempdir().unwrap();
        let request = PetImageDraw {
            frame: dir.path().join("missing.png"),
            protocol: ImageProtocol::Kitty,
            x: 2,
            y: 3,
            clear_top_y: 3,
            columns: 4,
            rows: 5,
            height_px: 75,
            sixel_dir: PathBuf::new(),
        };
        let mut output = Vec::new();
        let mut state = PetImageRenderState::default();

        let err = render_pet_image(&mut output, &mut state, Some(request)).unwrap_err();

        assert!(matches!(err, PetImageRenderError::Asset(_)));
        assert!(err.source().is_some());
    }

    #[test]
    fn writer_failure_is_a_terminal_error() {
        struct FailingWriter;

        impl io::Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "test writer failed",
                ))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut writer = FailingWriter;
        let mut state = PetImageRenderState {
            last_protocol: Some(ImageProtocol::Kitty),
            ..Default::default()
        };

        let err = render_pet_image(&mut writer, &mut state, None).unwrap_err();

        assert!(matches!(err, PetImageRenderError::Terminal(_)));
        assert!(err.source().is_some());
    }
}
