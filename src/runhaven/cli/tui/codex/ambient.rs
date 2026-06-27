//! Ambient image placement and terminal overlay rendering.
//!
//! Derived from openai/codex:
//! - `codex-rs/tui/src/pets/ambient.rs`
//! - `codex-rs/tui/src/pets/mod.rs`
//!
//! RunHaven changes: this module is asset-agnostic so the same Codex placement
//! and overlay machinery can render both the RunHaven logo and the Cubby pet.

use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;
use ratatui::layout::Rect;

use super::image_protocol;
use super::image_protocol::ImageProtocol;

const PET_TARGET_HEIGHT_PX: u16 = 75;
const PET_COMPOSER_GAP_PX: u16 = 10;
const TERMINAL_ROW_HEIGHT_PX: u16 = 15;
const TERMINAL_CELL_ASPECT: f64 = 0.52;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct AmbientImageDraw {
    pub(crate) frame: PathBuf,
    pub(crate) protocol: ImageProtocol,
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) clear_top_y: u16,
    pub(crate) columns: u16,
    pub(crate) rows: u16,
    pub(crate) height_px: u16,
    pub(crate) sixel_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct AmbientImageSize {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
    pub(crate) height_px: u16,
}

pub(crate) fn ambient_pet_image_size(frame_width: u32, frame_height: u32) -> AmbientImageSize {
    let rows = (f64::from(PET_TARGET_HEIGHT_PX) / f64::from(TERMINAL_ROW_HEIGHT_PX))
        .round()
        .max(1.0) as u16;
    let aspect = f64::from(frame_height) / f64::from(frame_width) * TERMINAL_CELL_ASPECT;
    let columns = (f64::from(rows) / aspect).round() as u16;
    AmbientImageSize {
        columns: columns.max(1),
        rows,
        height_px: PET_TARGET_HEIGHT_PX,
    }
}

/// Build an image draw request anchored above the composer.
///
/// This preserves Codex's ambient pet placement contract: callers provide the
/// available pane and the bottom boundary to avoid, and this function decides
/// the target image size, right edge, vertical gap, and clear area.
pub(crate) fn ambient_pet_draw_request(
    frame: PathBuf,
    protocol: ImageProtocol,
    sixel_dir: PathBuf,
    area: Rect,
    composer_bottom_y: u16,
    frame_width: u32,
    frame_height: u32,
) -> Option<AmbientImageDraw> {
    let size = ambient_pet_image_size(frame_width, frame_height);
    let cell_area = ambient_pet_cell_area(area, composer_bottom_y, frame_width, frame_height)?;

    Some(AmbientImageDraw {
        frame,
        protocol,
        x: cell_area.x,
        y: cell_area.y,
        clear_top_y: area.y,
        columns: size.columns,
        rows: size.rows,
        height_px: size.height_px,
        sixel_dir,
    })
}

pub(crate) fn ambient_pet_cell_area(
    area: Rect,
    composer_bottom_y: u16,
    frame_width: u32,
    frame_height: u32,
) -> Option<Rect> {
    let size = ambient_pet_image_size(frame_width, frame_height);
    let sprite_bottom_y = composer_bottom_y.saturating_sub(composer_gap_rows());
    if sprite_bottom_y < area.y.saturating_add(size.rows) || area.width < size.columns {
        return None;
    }

    let x = area.x + area.width.saturating_sub(size.columns);
    let y = sprite_bottom_y.saturating_sub(size.rows);
    Some(Rect {
        x,
        y,
        width: size.columns,
        height: size.rows,
    })
}

/// Build an image draw request for a caller-owned fixed cell area.
///
/// Codex's terminal overlay path owns the rendering semantics; RunHaven uses
/// this small adapter only when the product must place a static logo in a
/// banner rectangle that Codex does not have as a domain object.
pub(crate) fn fixed_area_draw_request(
    frame: PathBuf,
    protocol: ImageProtocol,
    sixel_dir: PathBuf,
    area: Rect,
    height_px: u16,
) -> Option<AmbientImageDraw> {
    if area.width == 0 || area.height == 0 {
        return None;
    }
    Some(AmbientImageDraw {
        frame,
        protocol,
        x: area.x,
        y: area.y,
        clear_top_y: area.y,
        columns: area.width,
        rows: area.height,
        height_px,
        sixel_dir,
    })
}

fn composer_gap_rows() -> u16 {
    ((f64::from(PET_COMPOSER_GAP_PX) / f64::from(TERMINAL_ROW_HEIGHT_PX)).round() as u16).max(1)
}

#[derive(Debug)]
pub(crate) enum AmbientImageRenderError {
    Terminal(std::io::Error),
    Asset(anyhow::Error),
}

impl std::fmt::Display for AmbientImageRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Terminal(err) => write!(f, "terminal image write failed: {err}"),
            Self::Asset(err) => write!(f, "ambient image asset unavailable: {err}"),
        }
    }
}

impl std::error::Error for AmbientImageRenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Terminal(err) => Some(err),
            Self::Asset(err) => Some(err.as_ref()),
        }
    }
}

impl From<std::io::Error> for AmbientImageRenderError {
    fn from(err: std::io::Error) -> Self {
        Self::Terminal(err)
    }
}

#[derive(Debug, Default)]
pub(crate) struct AmbientImageRenderState {
    last_sixel_clear_area: Option<SixelClearArea>,
    last_protocol: Option<ImageProtocol>,
}

pub(crate) fn render_ambient_image(
    writer: &mut impl Write,
    state: &mut AmbientImageRenderState,
    image_id: u32,
    request: Option<AmbientImageDraw>,
) -> std::result::Result<(), AmbientImageRenderError> {
    use ratatui::crossterm::cursor::MoveTo;
    use ratatui::crossterm::cursor::RestorePosition;
    use ratatui::crossterm::cursor::SavePosition;
    use ratatui::crossterm::queue;

    let Some(request) = request else {
        if state.last_protocol.take().is_some_and(is_kitty_protocol) {
            write!(writer, "{}", image_protocol::kitty_delete_image(image_id))?;
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
        write!(writer, "{}", image_protocol::kitty_delete_image(image_id))?;
    }
    state.last_protocol = Some(request.protocol);

    let payload = match request.protocol {
        ImageProtocol::Kitty => AmbientImagePayload::Text(
            image_protocol::kitty_transmit_png_with_id(
                &request.frame,
                request.columns,
                request.rows,
                Some(image_id),
            )
            .map_err(AmbientImageRenderError::Asset)?,
        ),
        ImageProtocol::KittyLocalFile => AmbientImagePayload::Text(
            image_protocol::kitty_transmit_png_file_with_id(
                &request.frame,
                request.columns,
                request.rows,
                Some(image_id),
            )
            .map_err(AmbientImageRenderError::Asset)?,
        ),
        ImageProtocol::Sixel => {
            let path =
                image_protocol::sixel_frame(&request.frame, &request.sixel_dir, request.height_px)
                    .map_err(AmbientImageRenderError::Asset)?;
            let sixel = std::fs::read(&path)
                .with_context(|| format!("read {}", path.display()))
                .map_err(AmbientImageRenderError::Asset)?;
            AmbientImagePayload::Bytes(sixel)
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
        AmbientImagePayload::Text(payload) => write!(writer, "{payload}")?,
        AmbientImagePayload::Bytes(payload) => writer.write_all(&payload)?,
    }
    queue!(writer, RestorePosition)?;
    writer.flush()?;
    Ok(())
}

enum AmbientImagePayload {
    Text(String),
    Bytes(Vec<u8>),
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

impl From<&AmbientImageDraw> for SixelClearArea {
    fn from(request: &AmbientImageDraw) -> Self {
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
    fn ambient_pet_size_matches_codex_target_height() {
        assert_eq!(
            ambient_pet_image_size(192, 208),
            AmbientImageSize {
                columns: 9,
                rows: 5,
                height_px: 75,
            }
        );
    }

    #[test]
    fn ambient_pet_draw_request_anchors_above_bottom_boundary() {
        let request = ambient_pet_draw_request(
            PathBuf::from("frame.png"),
            ImageProtocol::Kitty,
            PathBuf::new(),
            Rect::new(2, 4, 40, 20),
            22,
            192,
            208,
        )
        .unwrap();

        assert_eq!(request.x, 33);
        assert_eq!(request.y, 16);
        assert_eq!(request.clear_top_y, 4);
        assert_eq!(request.columns, 9);
        assert_eq!(request.rows, 5);
    }

    #[test]
    fn ambient_pet_draw_request_rejects_tight_area() {
        assert!(
            ambient_pet_draw_request(
                PathBuf::from("frame.png"),
                ImageProtocol::Kitty,
                PathBuf::new(),
                Rect::new(0, 0, 8, 10),
                10,
                192,
                208,
            )
            .is_none()
        );
    }

    #[test]
    fn ambient_image_restores_cursor_after_drawing() {
        let dir = tempfile::tempdir().unwrap();
        let frame = dir.path().join("frame.png");
        std::fs::write(&frame, b"png").unwrap();
        let request = AmbientImageDraw {
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
        let mut state = AmbientImageRenderState::default();

        render_ambient_image(&mut output, &mut state, 0xC0DE, Some(request)).unwrap();

        let output = String::from_utf8(output).unwrap();
        let save = output.find("\x1b7").expect("saves cursor position");
        let move_to = output.find("\x1b[4;3H").expect("moves to image position");
        let image = output.find("cG5n").expect("writes image payload");
        let restore = output.find("\x1b8").expect("restores cursor position");
        assert!(save < move_to);
        assert!(move_to < image);
        assert!(image < restore);
    }

    #[test]
    fn kitty_image_clear_deletes_without_moving_cursor() {
        let dir = tempfile::tempdir().unwrap();
        let frame = dir.path().join("frame.png");
        std::fs::write(&frame, b"png").unwrap();
        let request = AmbientImageDraw {
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
        let mut state = AmbientImageRenderState::default();

        render_ambient_image(&mut output, &mut state, 0xC0DE, Some(request)).unwrap();
        output.clear();
        render_ambient_image(&mut output, &mut state, 0xC0DE, None).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("Ga=d,d=I,i=49374,q=2;"));
        assert!(!output.contains("\x1b7"));
        assert!(!output.contains("\x1b["));
        assert!(!output.contains("\x1b8"));
    }

    #[test]
    fn sixel_image_clears_cell_area_before_redrawing() {
        let dir = tempfile::tempdir().unwrap();
        let frame = dir.path().join("frame.png");
        std::fs::write(&frame, b"png").unwrap();
        let sixel_dir = dir.path().join("sixel");
        std::fs::create_dir(&sixel_dir).unwrap();
        let sixel_frame = sixel_dir.join("frame_h75_v2.six");
        std::fs::write(&sixel_frame, b"fake-sixel").unwrap();
        let request = AmbientImageDraw {
            frame,
            protocol: ImageProtocol::Sixel,
            x: 2,
            y: 3,
            clear_top_y: 1,
            columns: 4,
            rows: 2,
            height_px: 75,
            sixel_dir,
        };
        let mut output = Vec::new();
        let mut state = AmbientImageRenderState::default();

        render_ambient_image(&mut output, &mut state, 0xC0DE, Some(request)).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("\x1b[2;3H    \x1b[3;3H    \x1b[4;3H    \x1b[5;3H    \x1b[4;3H"));
        assert!(output.contains("fake-sixel"));
        assert!(output.contains("\x1b8"));
    }

    #[test]
    fn missing_frame_is_an_asset_error() {
        let dir = tempfile::tempdir().unwrap();
        let request = AmbientImageDraw {
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
        let mut state = AmbientImageRenderState::default();

        let err = render_ambient_image(&mut output, &mut state, 0xC0DE, Some(request)).unwrap_err();

        assert!(matches!(err, AmbientImageRenderError::Asset(_)));
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
        let mut state = AmbientImageRenderState::default();
        let request = AmbientImageDraw {
            frame: PathBuf::from("frame.png"),
            protocol: ImageProtocol::Kitty,
            x: 2,
            y: 3,
            clear_top_y: 3,
            columns: 4,
            rows: 5,
            height_px: 75,
            sixel_dir: PathBuf::new(),
        };

        let err = render_ambient_image(&mut writer, &mut state, 0xC0DE, Some(request)).unwrap_err();

        assert!(matches!(err, AmbientImageRenderError::Terminal(_)));
        assert!(err.source().is_some());
    }
}
