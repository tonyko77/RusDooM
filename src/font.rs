//! Handling and drawing fonts

use crate::pixmap::*;
use crate::utils::*;
use crate::*;
use bytes::Bytes;

pub struct Font {
    font: Vec<PixMap>,
    grayscale: Box<[u8; 256]>,
}

impl Font {
    pub fn new() -> Self {
        // prepare a pseudo-grayscale palette
        let mut grayscale = Box::new([0; 256]);
        for i in 0..256 {
            grayscale[i] = i as u8;
        }

        Font {
            font: vec![PixMap::new_empty(); 64],
            grayscale,
        }
    }

    pub fn add_font_lump(&mut self, name: &str, bytes: &Bytes) {
        if name.len() > 6 {
            // extract the code from the lump name
            let code = match &name[0..5] {
                "STCFN" => atoi(&name[5..]).unwrap_or(9999),
                "FONTA" => atoi(&name[5..]).unwrap_or(9999) + 32,
                _ => 9999,
            };
            // map the code to a index between 0..=63
            let idx = match code {
                33..=95 => code - 33,
                121 | 124 => 63,
                _ => 9999,
            } as usize;
            if idx <= 63 {
                self.font[idx] = PixMap::from_patch(bytes);
            }
        }
    }

    pub fn compute_grayscale(&mut self, playpal: &Bytes) {
        assert!(playpal.len() >= 768);
        for i in 0..=255 {
            let r = playpal[i * 3 + 0];
            let g = playpal[i * 3 + 1];
            let b = playpal[i * 3 + 2];
            // HACK: just pick the max between R, G and B levels
            // (fonts are usually the same color, so it works for DOOM's red-ish font)
            self.grayscale[i] = r.max(g.max(b));
        }
    }

    pub fn is_complete(&self) -> bool {
        (0..=57).all(|i| !self.font[i].is_empty())
    }

    pub fn draw_text(&self, x: i32, y: i32, text: &str, color: RGB, painter: &mut dyn Painter) {
        const SPACE_WIDTH: i32 = 6;
        let mapper = FontColorMapper(color, self.grayscale.as_ref());
        let mut dx = 0;
        for byte in text.bytes() {
            if byte <= 32 {
                dx += SPACE_WIDTH;
            } else {
                let idx = match byte {
                    33..=95 => (byte - 33) as usize,
                    96 => 6,
                    97..=122 => (byte - 65) as usize,
                    123 => 27,
                    124 => 63,
                    125 => 29,
                    126 => 61,
                    _ => 0,
                };
                let char_pixmap = &self.font[idx];
                if !char_pixmap.is_empty() {
                    char_pixmap.paint(x + dx, y, painter, &mapper);
                    dx += char_pixmap.width() as i32;
                }
            }
        }
    }
}

//---------------

/// Internal color mapper, for painting fonts
struct FontColorMapper<'a>(RGB, &'a [u8]);

impl<'a> ColorMapper for FontColorMapper<'a> {
    fn byte2rgb(&self, color: u8) -> RGB {
        let gray = self.1[color as usize] as u32;
        let r = (self.0.r as u32) * gray / 255;
        let g = (self.0.g as u32) * gray / 255;
        let b = (self.0.b as u32) * gray / 255;
        RGB::from(r as u8, g as u8, b as u8)
    }
}
