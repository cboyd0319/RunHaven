//! The RunHaven mascot, "Cubby": a friendly glass container cube with a tiny
//! agent spark inside, drawn as half-block pixel art so it renders in any
//! terminal without image protocols. Branding only; it shares no data plumbing
//! with the functional screens (see docs/plans/tui-architecture.md).
//!
//! The pixel grids live in `sprites.rs` (generated from the half-block renders
//! in docs/assets/terminal-mascot/, xterm-256 indexed, two pixel rows per cell).
//! `hero_for_area` picks the largest sprite whose banner fits the rows and
//! columns available above the agent list.

mod sprites;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

/// One Cubby hero at a fixed pixel size. `pixels` is a row-major grid of
/// xterm-256 color indices, where `0` means transparent; `pixel_rows` is even
/// so two pixel rows pair into one terminal cell.
pub(super) struct HeroSprite {
    pub width: u16,
    pub pixel_rows: u16,
    pub pixels: &'static [u8],
}

impl HeroSprite {
    /// Width in terminal cells (one cell per pixel column).
    pub fn cell_width(&self) -> u16 {
        self.width
    }

    /// Height in terminal cells (two pixel rows per cell).
    pub fn cell_height(&self) -> u16 {
        self.pixel_rows / 2
    }

    fn pixel(&self, row: u16, col: u16) -> Option<Color> {
        if row >= self.pixel_rows || col >= self.width {
            return None;
        }
        let index = self.pixels[(row * self.width + col) as usize];
        (index != 0).then_some(Color::Indexed(index))
    }

    /// The sprite as half-block lines: the foreground colors the top pixel of a
    /// cell, the background colors the bottom.
    #[allow(dead_code)]
    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines_with_color(true)
    }

    /// Render the same sprite without terminal colors for `NO_COLOR` and line
    /// capture paths. Shape remains visible, but color-dependent detail is gone.
    pub fn lines_with_color(&self, color_enabled: bool) -> Vec<Line<'static>> {
        let mut lines = Vec::with_capacity((self.pixel_rows / 2) as usize);
        let mut row = 0;
        while row + 1 < self.pixel_rows {
            let mut spans = Vec::with_capacity(self.width as usize);
            for col in 0..self.width {
                let (symbol, style) = match (
                    self.pixel(row, col),
                    self.pixel(row + 1, col),
                    color_enabled,
                ) {
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
            row += 2;
        }
        lines
    }
}

/// The largest hero whose banner height and width fit the available area.
/// Returns `None` when the terminal is too short or narrow for a useful mascot.
pub(super) fn hero_for_area(
    available_rows: u16,
    available_columns: u16,
) -> Option<&'static HeroSprite> {
    sprites::HEROES
        .iter()
        .find(|hero| hero.cell_height() <= available_rows && hero.cell_width() <= available_columns)
        .or_else(|| {
            sprites::HEROES.last().filter(|hero| {
                hero.cell_height() <= available_rows && hero.cell_width() <= available_columns
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprites_are_well_formed() {
        assert!(!sprites::HEROES.is_empty());
        for hero in sprites::HEROES {
            assert_eq!(hero.pixel_rows % 2, 0, "pixel rows must pair into cells");
            assert_eq!(
                hero.pixels.len(),
                (hero.width * hero.pixel_rows) as usize,
                "{}x{} grid length mismatch",
                hero.width,
                hero.pixel_rows
            );
        }
    }

    #[test]
    fn heroes_are_ordered_largest_first() {
        let heights: Vec<u16> = sprites::HEROES.iter().map(|h| h.cell_height()).collect();
        let mut sorted = heights.clone();
        sorted.sort_by(|a, b| b.cmp(a));
        assert_eq!(heights, sorted, "HEROES must be ordered largest-first");
    }

    #[test]
    fn banner_picks_largest_that_fits() {
        let big = sprites::HEROES.first().unwrap();
        let small = sprites::HEROES.last().unwrap();
        assert_eq!(
            hero_for_area(1000, 1000).unwrap().cell_height(),
            big.cell_height()
        );
        assert_eq!(
            hero_for_area(small.cell_height(), small.cell_width())
                .unwrap()
                .cell_height(),
            small.cell_height()
        );
        assert!(hero_for_area(0, 1000).is_none());
        assert!(hero_for_area(1000, 0).is_none());
    }

    #[test]
    fn lines_match_cell_dimensions() {
        for hero in sprites::HEROES {
            let lines = hero.lines();
            assert_eq!(lines.len() as u16, hero.cell_height());
            for line in &lines {
                assert_eq!(line.spans.len() as u16, hero.cell_width());
            }
        }
    }

    #[test]
    fn no_color_lines_keep_shape_without_color() {
        for hero in sprites::HEROES {
            let lines = hero.lines_with_color(false);
            let non_space = lines
                .iter()
                .flat_map(|line| &line.spans)
                .filter(|span| span.content.as_ref() != " ")
                .count();
            assert!(non_space > 0);
            for span in lines.iter().flat_map(|line| &line.spans) {
                assert_eq!(span.style.fg, None);
                assert_eq!(span.style.bg, None);
            }
        }
    }
}
