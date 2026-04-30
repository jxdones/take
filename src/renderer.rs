use crate::{box_drawing::draw_box_char, player::Frame};
use anyhow::Result;
use fontdue::{Font, FontSettings, Metrics};
use image::{Rgb, RgbImage};
use std::collections::HashMap;
use vt100::Color;

const FONT_SIZE: f32 = 14.0;
const COLS: u16 = 80;
const ROWS: u16 = 24;
const PADDING: u32 = 16;
const DEFAULT_BG: (u8, u8, u8) = (0, 0, 0);

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

    let width = COLS as u32 * cell_width + 2 * PADDING;
    let height = ROWS as u32 * cell_height + 2 * PADDING;

    let mut file = std::fs::File::create(output)?;
    let mut encoder = gif::Encoder::new(&mut file, width as u16, height as u16, &[])?;
    encoder.set_repeat(gif::Repeat::Infinite)?;

    let (bg_r, bg_g, bg_b) = DEFAULT_BG;
    for frame in frames {
        let mut img = image::ImageBuffer::from_pixel(width, height, Rgb([bg_r, bg_g, bg_b]));
        for row in 0..ROWS {
            for col in 0..COLS {
                if let Some(cell) = frame.screen.cell(row, col) {
                    let bgcolor = cell.bgcolor();
                    let fgcolor = cell.fgcolor();

                    let contents = cell.contents();
                    let is_cursor =
                        frame.screen.cursor_position() == (row, col) && !frame.screen.hide_cursor();
                    let ch = if is_cursor {
                        '_'
                    } else {
                        contents.chars().next().unwrap_or(' ')
                    };
                    let (glyph_metrics, bitmap) = glyph_cache
                        .entry(ch)
                        .or_insert_with(|| font.rasterize(ch, FONT_SIZE));

                    // pixel coordinates of the top-left corner of the cell
                    let x = col as u32 * cell_width + PADDING;
                    let y = row as u32 * cell_height + PADDING;

                    let (bg_r, bg_g, bg_b) = color_to_rgb(bgcolor, DEFAULT_BG);
                    for py in y..y + cell_height {
                        for px in x..x + cell_width {
                            img.put_pixel(px, py, Rgb([bg_r, bg_g, bg_b]));
                        }
                    }

                    let (fg_r, fg_g, fg_b) = color_to_rgb(fgcolor, (255, 255, 255));
                    let baseline = y as i32 + ascent;

                    if let Some(strokes) = crate::box_drawing::strokes_for(ch) {
                        draw_box_char(
                            &strokes,
                            x,
                            y,
                            cell_width,
                            cell_height,
                            (fg_r, fg_g, fg_b),
                            &mut img,
                        );
                    } else {
                        for gy in 0..glyph_metrics.height {
                            for gx in 0..glyph_metrics.width {
                                // xmin/ymin offset the bitmap relative to the baseline and left edge of the cell
                                let px = x as i32 + glyph_metrics.xmin + gx as i32;
                                let py =
                                    baseline - glyph_metrics.ymin - glyph_metrics.height as i32
                                        + gy as i32;

                                // skip pixels that fall outside the image (e.g. tall Nerd Font icons)
                                if px < 0 || px >= width as i32 || py < 0 || py >= height as i32 {
                                    continue;
                                }

                                // coverage: how much of this pixel the letter's shape occupies (0=none, 255=fully inside).
                                // edge pixels get a partial value, which we use to blend fg/bg for smooth curves.
                                let coverage = bitmap[gy * glyph_metrics.width + gx];
                                if coverage > 0 {
                                    let alpha = coverage as f32 / 255.0;
                                    let r =
                                        (fg_r as f32 * alpha + bg_r as f32 * (1.0 - alpha)) as u8;
                                    let g =
                                        (fg_g as f32 * alpha + bg_g as f32 * (1.0 - alpha)) as u8;
                                    let b =
                                        (fg_b as f32 * alpha + bg_b as f32 * (1.0 - alpha)) as u8;
                                    img.put_pixel(px as u32, py as u32, Rgb([r, g, b]));
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut gif_frame = indexed_frame(&img, width as u16, height as u16);
        gif_frame.delay = (frame.duration.as_millis() / 10).max(2) as u16;
        encoder.write_frame(&gif_frame)?;
    }

    Ok(())
}

/// Encodes an `RgbImage` as a GIF frame using a direct indexed palette, avoiding NeuQuant dithering.
/// Falls back to NeuQuant if the frame contains more than 256 unique colors.
fn indexed_frame(img: &RgbImage, width: u16, height: u16) -> gif::Frame<'static> {
    let mut color_to_idx: std::collections::HashMap<[u8; 3], u8> = std::collections::HashMap::new();
    let mut palette: Vec<u8> = Vec::new();

    let mut pixels: Vec<u8> = Vec::with_capacity((width as usize) * (height as usize));
    for pixel in img.pixels() {
        let rgb = pixel.0;
        let next_idx = palette.len() / 3;
        if next_idx >= 256 && !color_to_idx.contains_key(&rgb) {
            return gif::Frame::from_rgb(width, height, img.as_raw());
        }
        let idx = if let Some(&i) = color_to_idx.get(&rgb) {
            i
        } else {
            let i = next_idx as u8;
            color_to_idx.insert(rgb, i);
            palette.extend_from_slice(&rgb);
            i
        };
        pixels.push(idx);
    }

    // GIF palette length must be a power of two, padded to at least 2 entries.
    let palette_size = (palette.len() / 3).next_power_of_two().max(2);
    palette.resize(palette_size * 3, 0);

    gif::Frame {
        width,
        height,
        palette: Some(palette),
        buffer: std::borrow::Cow::Owned(pixels),
        ..Default::default()
    }
}

// Converts a vt100 terminal color to an RGB tuple, falling back to `default` for the default color.
fn color_to_rgb(color: Color, default: (u8, u8, u8)) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Idx(n) => match n {
            0 => (0, 0, 0),
            1 => (128, 0, 0),
            2 => (0, 128, 0),
            3 => (128, 128, 0),
            4 => (0, 0, 128),
            5 => (128, 0, 128),
            6 => (0, 128, 128),
            7 => (192, 192, 192),
            8 => (128, 128, 128),
            9 => (255, 0, 0),
            10 => (0, 255, 0),
            11 => (255, 255, 0),
            12 => (0, 0, 255),
            13 => (255, 0, 255),
            14 => (0, 255, 255),
            15 => (255, 255, 255),
            _ => default,
        },
        Color::Default => default,
    }
}

// TODO Add unit tests for both export_gif and color_to_rgb function
