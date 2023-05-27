//! Pixel Maps (Patches, Flats, Fonts)

use crate::utils::*;
use crate::*;
use bytes::Bytes;

/// Trait which provides color mapping at runtime (u8 -> RGB).
pub trait ColorMapper {
    /// Map a byte value to a color.
    fn byte2rgb(&self, color: u8) -> RGB;
}

/// Pixel map structure.
/// Can be used for Patches, Flats or Fonts.
#[derive(Clone)]
pub struct PixMap {
    width: u16,
    height: u16,
    kind: PixMapKind,
    data: Bytes,
}

impl PixMap {
    pub fn new_empty() -> Self {
        Self::new_placeholder(0, 0)
    }

    pub fn new_placeholder(width: usize, height: usize) -> Self {
        Self {
            width: width as u16,
            height: height as u16,
            kind: PixMapKind::PlaceHolder,
            data: Bytes::new(),
        }
    }

    pub fn from_flat(flat_bytes: &Bytes) -> Self {
        let data = flat_bytes.clone();
        let height = (data.len() >> 6) as u16;
        Self {
            width: 64,
            height,
            kind: PixMapKind::Flat,
            data,
        }
    }

    pub fn from_patch(patch_bytes: &Bytes) -> Self {
        let data = patch_bytes.clone();
        let width = buf_to_u16(&data[0..=1]);
        let height = buf_to_u16(&data[2..=3]);
        Self {
            width,
            height,
            kind: PixMapKind::Patch,
            data,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    #[inline]
    pub fn x_offset(&self) -> i32 {
        match self.kind {
            PixMapKind::Patch => -buf_to_i16(&self.data[4..6]) as i32,
            _ => 0,
        }
    }

    #[inline]
    pub fn y_offset(&self) -> i32 {
        match self.kind {
            PixMapKind::Patch => -buf_to_i16(&self.data[6..8]) as i32,
            _ => 0,
        }
    }

    pub fn paint(&self, x: i32, y: i32, painter: &mut dyn Painter, mapper: &dyn ColorMapper) {
        if self.width > 0 && self.height > 0 {
            match self.kind {
                PixMapKind::Flat => self.paint_flat(x, y, painter, mapper),
                PixMapKind::Patch => self.paint_patch(x, y, painter, mapper),
                PixMapKind::PlaceHolder => self.paint_pink(x, y, painter),
            }
        }
    }

    fn paint_pink(&self, x: i32, y: i32, painter: &mut dyn Painter) {
        for dy in 0..self.height as i32 {
            for dx in 0..self.width as i32 {
                painter.draw_pixel(x + dx, y + dy, RGB::from(255, 0, 255));
            }
        }
    }

    fn paint_flat(&self, x: i32, y: i32, painter: &mut dyn Painter, mapper: &dyn ColorMapper) {
        let mut idx = 0;
        for dy in 0..self.height as i32 {
            for dx in 0..self.width as i32 {
                let pixcode = self.data[idx];
                idx += 1;
                let color = mapper.byte2rgb(pixcode);
                painter.draw_pixel(x + dx, y + dy, color);
            }
        }
    }

    #[inline]
    fn paint_patch(&self, x: i32, y: i32, painter: &mut dyn Painter, mapper: &dyn ColorMapper) {
        self.paint_patch_customized(
            x,
            y,
            painter,
            mapper,
            self.x_offset(),
            self.y_offset(),
            self.width as i32,
            self.height as i32,
            false,
        );
    }

    fn paint_patch_customized(
        &self,
        x: i32,
        y: i32,
        painter: &mut dyn Painter,
        mapper: &dyn ColorMapper,
        x_offs: i32,
        y_offs: i32,
        w: i32,
        h: i32,
        clip: bool,
    ) {
        let mut ofs_idx = 8;
        for dx in 0..self.width as i32 {
            // find the column index
            let mut col_idx = buf_to_u32(&self.data[ofs_idx..ofs_idx + 4]) as usize;
            ofs_idx += 4;
            // optimization: skip column in clip mode, if outside view port
            let xx = dx + x_offs;
            if clip && (xx < 0 || xx >= w) {
                continue;
            }
            loop {
                let dy = self.data[col_idx] as i32;
                if dy == 0xFF {
                    break;
                }
                let len = self.data[col_idx + 1] as i32;
                for i in 0..len {
                    let yy = dy + i + y_offs;
                    if clip && (yy < 0 || yy >= h) {
                        continue;
                    }
                    let pixcode = self.data[col_idx + 3 + (i as usize)];
                    let color = mapper.byte2rgb(pixcode);
                    painter.draw_pixel(x + xx, y + yy, color);
                }
                col_idx += 4 + (len as usize);
            }
        }
    }
}

//----------------------

/// Texture = a collection of Patches.
pub struct Texture {
    width: u16,
    height: u16,
    patches: Vec<TexturePatch>,
}

impl Texture {
    pub fn new(width: u16, height: u16, patch_cnt: usize) -> Texture {
        Texture {
            width,
            height,
            patches: Vec::with_capacity(patch_cnt),
        }
    }

    pub fn add_patch(&mut self, patch_bytes: &Bytes, x_orig: i16, y_orig: i16) {
        let tex_patch = TexturePatch {
            pixmap: PixMap::from_patch(patch_bytes),
            x_orig,
            y_orig,
        };
        self.patches.push(tex_patch);
    }

    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn paint(&self, x: i32, y: i32, painter: &mut dyn Painter, mapper: &dyn ColorMapper) {
        if self.width > 0 && self.height > 0 {
            for patch in &self.patches {
                patch.pixmap.paint_patch_customized(
                    x,
                    y,
                    painter,
                    mapper,
                    patch.x_orig as i32,
                    patch.y_orig as i32,
                    self.width as i32,
                    self.height as i32,
                    true,
                );
            }
        }
    }
}

//----------------------
// Internal stuff

/// Internal enum for the various kinds of pixel maps.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PixMapKind {
    Patch,
    Flat,
    PlaceHolder,
}

struct TexturePatch {
    pixmap: PixMap,
    x_orig: i16,
    y_orig: i16,
}
