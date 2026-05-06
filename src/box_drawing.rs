use image::RgbImage;

/// Thickness of a single box-drawing stroke arm.
pub enum Weight {
    None,
    Light,
    Heavy,
    Double,
}

/// The four arms of a box-drawing character (top, bottom, left, right).
pub struct Strokes {
    pub top: Weight,
    pub bottom: Weight,
    pub left: Weight,
    pub right: Weight,
}

/// Maps a Unicode box-drawing character to its stroke description, or `None` if not supported.
pub fn strokes_for(ch: char) -> Option<Strokes> {
    use Weight::*;
    macro_rules! s {
        ($t:ident, $b:ident, $l:ident, $r:ident) => {
            Some(Strokes {
                top: $t,
                bottom: $b,
                left: $l,
                right: $r,
            })
        };
    }

    match ch {
        // Straight lines
        '─' => s!(None, None, Light, Light),
        '━' => s!(None, None, Heavy, Heavy),
        '│' => s!(Light, Light, None, None),
        '┃' => s!(Heavy, Heavy, None, None),

        // Corners down-right
        '┌' => s!(None, Light, None, Light),
        '┍' => s!(None, Light, None, Heavy),
        '┎' => s!(None, Heavy, None, Light),
        '┏' => s!(None, Heavy, None, Heavy),

        // Corners down-left
        '┐' => s!(None, Light, Light, None),
        '┑' => s!(None, Light, Heavy, None),
        '┒' => s!(None, Heavy, Light, None),
        '┓' => s!(None, Heavy, Heavy, None),

        // Corners up-right
        '└' => s!(Light, None, None, Light),
        '┕' => s!(Light, None, None, Heavy),
        '┖' => s!(Heavy, None, None, Light),
        '┗' => s!(Heavy, None, None, Heavy),

        // Corners up-left
        '┘' => s!(Light, None, Light, None),
        '┙' => s!(Light, None, Heavy, None),
        '┚' => s!(Heavy, None, Light, None),
        '┛' => s!(Heavy, None, Heavy, None),

        // T-junctions right
        '├' => s!(Light, Light, None, Light),
        '┝' => s!(Light, Light, None, Heavy),
        '┞' => s!(Heavy, Light, None, Light),
        '┟' => s!(Light, Heavy, None, Light),
        '┠' => s!(Heavy, Heavy, None, Light),
        '┡' => s!(Heavy, Light, None, Heavy),
        '┢' => s!(Light, Heavy, None, Heavy),
        '┣' => s!(Heavy, Heavy, None, Heavy),

        // T-junctions left
        '┤' => s!(Light, Light, Light, None),
        '┥' => s!(Light, Light, Heavy, None),
        '┦' => s!(Heavy, Light, Light, None),
        '┧' => s!(Light, Heavy, Light, None),
        '┨' => s!(Heavy, Heavy, Light, None),
        '┩' => s!(Heavy, Light, Heavy, None),
        '┪' => s!(Light, Heavy, Heavy, None),
        '┫' => s!(Heavy, Heavy, Heavy, None),

        // T-junctions down
        '┬' => s!(None, Light, Light, Light),
        '┭' => s!(None, Light, Heavy, Light),
        '┮' => s!(None, Light, Light, Heavy),
        '┯' => s!(None, Light, Heavy, Heavy),
        '┰' => s!(None, Heavy, Light, Light),
        '┱' => s!(None, Heavy, Heavy, Light),
        '┲' => s!(None, Heavy, Light, Heavy),
        '┳' => s!(None, Heavy, Heavy, Heavy),

        // T-junctions up
        '┴' => s!(Light, None, Light, Light),
        '┵' => s!(Light, None, Heavy, Light),
        '┶' => s!(Light, None, Light, Heavy),
        '┷' => s!(Light, None, Heavy, Heavy),
        '┸' => s!(Heavy, None, Light, Light),
        '┹' => s!(Heavy, None, Heavy, Light),
        '┺' => s!(Heavy, None, Light, Heavy),
        '┻' => s!(Heavy, None, Heavy, Heavy),

        // Crosses
        '┼' => s!(Light, Light, Light, Light),
        '┽' => s!(Light, Light, Heavy, Light),
        '┾' => s!(Light, Light, Light, Heavy),
        '┿' => s!(Light, Light, Heavy, Heavy),
        '╀' => s!(Heavy, Light, Light, Light),
        '╁' => s!(Light, Heavy, Light, Light),
        '╂' => s!(Heavy, Heavy, Light, Light),
        '╃' => s!(Heavy, Light, Heavy, Light),
        '╄' => s!(Heavy, Light, Light, Heavy),
        '╅' => s!(Light, Heavy, Heavy, Light),
        '╆' => s!(Light, Heavy, Light, Heavy),
        '╇' => s!(Light, Heavy, Heavy, Heavy),
        '╈' => s!(Heavy, Light, Heavy, Heavy),
        '╉' => s!(Heavy, Heavy, Heavy, Light),
        '╊' => s!(Heavy, Heavy, Light, Heavy),
        '╋' => s!(Heavy, Heavy, Heavy, Heavy),

        // Double lines
        '═' => s!(None, None, Double, Double),
        '║' => s!(Double, Double, None, None),

        // Double corners down-right
        '╒' => s!(None, Light, None, Double),
        '╓' => s!(None, Double, None, Light),
        '╔' => s!(None, Double, None, Double),

        // Double corners down-left
        '╕' => s!(None, Light, Double, None),
        '╖' => s!(None, Double, Light, None),
        '╗' => s!(None, Double, Double, None),

        // Double corners up-right
        '╘' => s!(Light, None, None, Double),
        '╙' => s!(Double, None, None, Light),
        '╚' => s!(Double, None, None, Double),

        // Double corners up-left
        '╛' => s!(Light, None, Double, None),
        '╜' => s!(Double, None, Light, None),
        '╝' => s!(Double, None, Double, None),

        // Double T-junctions right
        '╞' => s!(Light, Light, None, Double),
        '╟' => s!(Double, Double, None, Light),
        '╠' => s!(Double, Double, None, Double),

        // Double T-junctions left
        '╡' => s!(Light, Light, Double, None),
        '╢' => s!(Double, Double, Light, None),
        '╣' => s!(Double, Double, Double, None),

        // Double T-junctions down
        '╤' => s!(None, Light, Double, Double),
        '╥' => s!(None, Double, Light, Light),
        '╦' => s!(None, Double, Double, Double),

        // Double T-junctions up
        '╧' => s!(Light, None, Double, Double),
        '╨' => s!(Double, None, Light, Light),
        '╩' => s!(Double, None, Double, Double),

        // Double crosses
        '╪' => s!(Light, Light, Double, Double),
        '╫' => s!(Double, Double, Light, Light),
        '╬' => s!(Double, Double, Double, Double),

        _ => Option::None,
    }
}

/// Draws a box-drawing character geometrically as filled rectangles, bypassing font rasterization.
pub fn draw_box_char(
    strokes: &Strokes,
    x: u32,
    y: u32,
    cell_width: u32,
    cell_height: u32,
    color: (u8, u8, u8),
    img: &mut RgbImage,
) {
    let cx = x + cell_width / 2;
    let cy = y + cell_height / 2;
    let pixel = image::Rgb([color.0, color.1, color.2]);

    let mut fill = |x0: u32, y0: u32, x1: u32, y1: u32| {
        for py in y0..y1 {
            for px in x0..x1 {
                img.put_pixel(px, py, pixel);
            }
        }
    };

    match strokes.right {
        Weight::Light => fill(cx, cy, x + cell_width, cy + 1),
        Weight::Heavy => fill(cx, cy - 1, x + cell_width, cy + 1),
        Weight::Double => {
            fill(cx, cy - 1, x + cell_width, cy);
            fill(cx, cy + 1, x + cell_width, cy + 2);
        }
        Weight::None => {}
    }

    match strokes.left {
        Weight::Light => fill(x, cy, cx, cy + 1),
        Weight::Heavy => fill(x, cy - 1, cx, cy + 1),
        Weight::Double => {
            fill(x, cy - 1, cx, cy);
            fill(x, cy + 1, cx, cy + 2);
        }
        Weight::None => {}
    }

    match strokes.top {
        Weight::Light => fill(cx, y, cx + 1, cy),
        Weight::Heavy => fill(cx - 1, y, cx + 1, cy),
        Weight::Double => {
            fill(cx - 1, y, cx, cy);
            fill(cx + 1, y, cx + 2, cy);
        }
        Weight::None => {}
    }

    match strokes.bottom {
        Weight::Light => fill(cx, cy, cx + 1, y + cell_height),
        Weight::Heavy => fill(cx - 1, cy, cx + 1, y + cell_height),
        Weight::Double => {
            fill(cx - 1, cy, cx, y + cell_height);
            fill(cx + 1, cy, cx + 2, y + cell_height);
        }
        Weight::None => {}
    }
}

// TODO Add unit tests for strokes_for and draw_box_char
