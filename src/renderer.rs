use crate::player::Frame;
use anyhow::Result;
use fontdue::{Font, FontSettings, Metrics};
use image::{Rgb, RgbImage};
use std::collections::HashMap;
use vt100::Color;

const FONT_SIZE: f32 = 14.0;
const COLS: u16 = 80;
const ROWS: u16 = 24;

// Renders each terminal frame as an image and encodes them into an animated GIF file.
pub fn export_gif(frames: &[Frame], output: &str) -> Result<()> {
    let font_bytes: &[u8] = include_bytes!("../assets/JetBrainsMonoNerdFont-Regular.ttf");
    let font = Font::from_bytes(font_bytes, FontSettings::default()).unwrap();

    let line_metrics = font.horizontal_line_metrics(FONT_SIZE).unwrap();
    let cell_height = line_metrics.new_line_size as u32;
    let ascent = line_metrics.ascent as i32;

    let mut glyph_cache: HashMap<char, (Metrics, Vec<u8>)> = HashMap::new();

    // 'M' is the conventional reference character for measuring monospace cell width.
    let (m_metrics, _) = font.rasterize('M', FONT_SIZE);
    let cell_width = m_metrics.advance_width as u32;

    let width = COLS as u32 * cell_width;
    let height = ROWS as u32 * cell_height;

    let mut file = std::fs::File::create(output)?;
    let mut encoder = gif::Encoder::new(&mut file, width as u16, height as u16, &[])?;
    encoder.set_repeat(gif::Repeat::Infinite)?;

    for frame in frames {
        let mut img = RgbImage::new(width, height);
        for row in 0..ROWS {
            for col in 0..COLS {
                if let Some(cell) = frame.screen.cell(row, col) {
                    let bgcolor = cell.bgcolor();
                    let fgcolor = cell.fgcolor();

                    let contents = cell.contents();
                    let is_cursor = frame.screen.cursor_position() == (row, col)
                        && !frame.screen.hide_cursor();
                    let ch = if is_cursor {
                        '_'
                    } else {
                        contents.chars().next().unwrap_or(' ')
                    };
                    let (glyph_metrics, bitmap) = glyph_cache
                        .entry(ch)
                        .or_insert_with(|| font.rasterize(ch, FONT_SIZE));

                    // pixel coordinates of the top-left corner of the cell
                    let x = col as u32 * cell_width;
                    let y = row as u32 * cell_height;

                    let (bg_r, bg_g, bg_b) = color_to_rgb(bgcolor, (0, 0, 0));
                    for py in y..y + cell_height {
                        for px in x..x + cell_width {
                            img.put_pixel(px, py, Rgb([bg_r, bg_g, bg_b]));
                        }
                    }

                    let (fg_r, fg_g, fg_b) = color_to_rgb(fgcolor, (255, 255, 255));
                    let baseline = y as i32 + ascent;

                    for gy in 0..glyph_metrics.height {
                        for gx in 0..glyph_metrics.width {
                            // xmin/ymin offset the bitmap relative to the baseline and left edge of the cell
                            let px = x as i32 + glyph_metrics.xmin + gx as i32;
                            let py = baseline - glyph_metrics.ymin - glyph_metrics.height as i32
                                + gy as i32;

                            // skip pixels that fall outside the image (e.g. tall Nerd Font icons)
                            if px < 0 || px >= width as i32 || py < 0 || py >= height as i32 {
                                continue;
                            }

                            // coverage: how much of this pixel the letter's shape occupies (0=none, 255=fully inside).
                            // edge pixels get a partial value, which we use to blend fg/bg for smooth curves.
                            let coverage = bitmap[gy * glyph_metrics.width + gx];
                            if coverage > 0 {
                                let alpha = coverage as f32 / 255.0; // convert to 0.0–1.0 blend weight
                                let r = (fg_r as f32 * alpha + bg_r as f32 * (1.0 - alpha)) as u8;
                                let g = (fg_g as f32 * alpha + bg_g as f32 * (1.0 - alpha)) as u8;
                                let b = (fg_b as f32 * alpha + bg_b as f32 * (1.0 - alpha)) as u8;
                                img.put_pixel(px as u32, py as u32, Rgb([r, g, b]));
                            }
                        }
                    }
                }
            }
        }

        let mut gif_frame = gif::Frame::from_rgb(width as u16, height as u16, img.as_raw());
        gif_frame.delay = (frame.duration.as_millis() / 10).max(2) as u16;
        encoder.write_frame(&gif_frame)?;
    }

    Ok(())
}

// Converts a vt100 terminal color to an RGB tuple, falling back to `default` for the default color.
fn color_to_rgb(color: Color, default: (u8, u8, u8)) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Idx(_n) => todo!(),
        Color::Default => default,
    }
}

// TODO Add unit tests for both export_gif and color_to_rgb function
