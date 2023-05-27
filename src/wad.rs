//! WAD loader and parser.
//! See [DIYDoom, Notes001](https://github.com/amroibrahim/DIYDoom/tree/master/DIYDOOM/Notes001/notes).

use crate::font::Font;
use crate::graphics::Graphics;
use crate::map::*;
use crate::palette::Palette;
use crate::*;
use bytes::{Bytes, BytesMut};
use std::fs::*;
use std::io::Read;

/// Stores all the data (lumps) from a WAD file.
pub struct WadData {
    maps: Vec<MapData>,
    pal: Palette,
    gfx: Graphics,
    font: Font,
}

impl WadData {
    pub fn load(wad_path: &str, is_iwad: bool) -> Result<WadData, String> {
        // read WAD file bytes
        let mut wad_bytes: BytesMut;
        {
            let mut file = File::open(wad_path).map_err(|e| e.to_string())?;
            let len = file.metadata().map_err(|e| e.to_string())?.len() as usize;
            wad_bytes = BytesMut::zeroed(len);
            file.read_exact(&mut wad_bytes).map_err(|e| e.to_string())?;
        }
        let wad_bytes = wad_bytes.freeze();

        // check the WAD header
        if wad_bytes.len() <= 16 {
            return Err(format!("WAD file {wad_path} is too small"));
        }
        let wad_kind_str = std::str::from_utf8(&wad_bytes[0..4]).map_err(|_| String::from("Invalid WAD header"))?;
        let expected_kind_str = if is_iwad { "IWAD" } else { "PWAD" };
        if expected_kind_str.ne(wad_kind_str) {
            return Err(format!(
                "Invalid WAD type: expected {expected_kind_str}, was {wad_kind_str}"
            ));
        }

        let mut wad = WadData {
            maps: Vec::new(),
            pal: Palette::new(),
            gfx: Graphics::new(),
            font: Font::new(),
        };
        wad.parse_wad_lumps(wad_bytes)?;
        wad.validate_collected_data()?;
        Ok(wad)
    }

    #[inline]
    pub fn palette(&self) -> &Palette {
        &self.pal
    }

    #[inline]
    pub fn map_count(&self) -> usize {
        self.maps.len()
    }

    #[inline]
    pub fn map_name(&self, idx: usize) -> &str {
        // TODO panic-safe error handling ?!?
        assert!(idx < self.maps.len());
        &self.maps[idx].name()
    }

    #[inline]
    pub fn map(&self, idx: usize) -> &MapData {
        // TODO panic-safe error handling ?!?
        assert!(idx < self.maps.len());
        &self.maps[idx]
    }

    #[inline]
    pub fn font(&self) -> &Font {
        &self.font
    }

    #[inline]
    pub fn graphics(&self) -> &Graphics {
        &self.gfx
    }

    //-----------------

    fn parse_wad_lumps(&mut self, wad_bytes: Bytes) -> Result<(), String> {
        let lump_count = utils::buf_to_u32(&wad_bytes[4..8]) as usize;
        let dir_offset = utils::buf_to_u32(&wad_bytes[8..12]) as usize;
        let wad_len = wad_bytes.len();

        let mut is_flats = false;
        let mut currently_parsing_map: Option<MapData> = None;

        // parse each lump
        for lump_idx in 0..lump_count {
            let offs = dir_offset + 16 * lump_idx;
            let lump_start = utils::buf_to_u32(&wad_bytes[(offs + 0)..(offs + 4)]) as usize;
            let lump_size = utils::buf_to_u32(&wad_bytes[(offs + 4)..(offs + 8)]) as usize;
            let lump_name = extract_lump_name(&wad_bytes[(offs + 8)..(offs + 16)], lump_idx)?.to_string();
            let lump_end = lump_start + lump_size;
            if lump_end >= wad_len {
                return Err(format!("Lump {lump_name} too big: its end goes beyond the WAD"));
            }
            let lump_bytes = wad_bytes.slice(lump_start..lump_end);

            // parse map lumps
            if currently_parsing_map.is_some() {
                let mut map = currently_parsing_map.unwrap();
                let still_parsing = map.add_lump(&lump_name, &lump_bytes);
                if still_parsing {
                    currently_parsing_map = Some(map);
                    continue;
                }
                // finished parsing one map
                currently_parsing_map = None;
                if !map.is_complete() {
                    return Err(format!("Incomplete map in WAD: {}", map.name()));
                }
                self.maps.push(map);
            }
            if is_map_name(&lump_name) {
                // starting to parse new map
                currently_parsing_map = Some(MapData::new(&lump_name));
                continue;
            }

            // parse other lump types
            match lump_name.as_str() {
                "PLAYPAL" => {
                    self.pal.init_palettes(&lump_bytes);
                    self.font.compute_grayscale(&lump_bytes);
                }
                "COLORMAP" => self.pal.init_colormaps(&lump_bytes),
                "PNAMES" => self.gfx.set_patch_names(&lump_bytes)?,
                "F_START" => is_flats = true,
                "F_END" => is_flats = false,
                _ => {
                    if is_texture_name(&lump_name) {
                        self.gfx.add_textures(&lump_bytes)?;
                    } else if (lump_bytes.len() > 0) && is_flats {
                        self.gfx.add_flat(&lump_name, &lump_bytes);
                    } else if quick_check_if_lump_is_graphic(&lump_bytes) {
                        self.gfx.add_patch(&lump_name, &lump_bytes);
                        if is_font_name(&lump_name) {
                            self.font.add_font_lump(&lump_name, &lump_bytes);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_collected_data(&self) -> Result<(), String> {
        if !self.pal.is_initialized() {
            Err(String::from("PLAYPAL or COLORMAP lump not found in WAD"))
        } else if self.maps.len() == 0 {
            Err(String::from("Maps not found in WAD"))
        } else if !self.font.is_complete() {
            Err(String::from("Fonts not found in WAD"))
        } else {
            Ok(())
        }
    }
}

//-----------------------------
//  Internal utils

fn extract_lump_name(name_bytes: &[u8], idx: usize) -> Result<&str, String> {
    // dismiss all null bytes at the name's end
    let mut idx_end = 0;
    for ch in name_bytes {
        if *ch == 0 {
            break;
        } else if *ch <= 32 || *ch >= 127 {
            return Err(format!("Invalid lump name at index {idx}"));
        } else {
            idx_end += 1;
        }
    }
    // all ok
    std::str::from_utf8(&name_bytes[0..idx_end]).map_err(|e| e.to_string())
}

#[inline]
fn is_map_name(name: &str) -> bool {
    const E: u8 = 'E' as u8;
    const M: u8 = 'M' as u8;
    const A: u8 = 'A' as u8;
    const P: u8 = 'P' as u8;

    let b = name.as_bytes();
    if b.len() == 4 {
        b[0] == E && is_ascii_digit(b[1]) && b[2] == M && is_ascii_digit(b[3])
    } else if b.len() == 5 {
        b[0] == M && b[1] == A && b[2] == P && is_ascii_digit(b[3]) && is_ascii_digit(b[4])
    } else {
        false
    }
}

#[inline]
fn is_texture_name(name: &str) -> bool {
    name.len() == 8 && &name[0..7] == "TEXTURE" && is_ascii_digit(name.as_bytes()[7])
}

#[inline]
fn is_font_name(name: &str) -> bool {
    name.len() >= 7 && {
        let n5 = &name[0..5];
        (n5 == "STCFN") || (n5 == "FONTA")
    }
}

#[inline]
fn is_ascii_digit(byte: u8) -> bool {
    byte >= ('0' as u8) && byte <= ('9' as u8)
}

// TODO improve check, to make sure nothing goes out of bounds !!?
fn quick_check_if_lump_is_graphic(bytes: &[u8]) -> bool {
    let len = bytes.len();
    if len < 12 {
        return false;
    }

    // check that, for each column, its offset fits in the patch
    let mut max_idx = 0;
    let width = utils::buf_to_u16(&bytes[0..=1]) as usize;
    let height = utils::buf_to_u16(&bytes[2..=3]) as usize;
    if width == 0 || height == 0 || len < (8 + 4 * width) {
        return false;
    }
    for col in 0..width {
        let col_ofs = utils::buf_to_u32(&bytes[8 + 4 * col..]) as usize;
        max_idx = max_idx.max(col_ofs);
        if len <= max_idx {
            return false;
        }
    }

    // check the column with the maximum offset
    loop {
        // if we went past the end of the lump bytes => NOT ok
        if max_idx >= len {
            return false;
        }
        // if we reached the end of column safely => we're ok
        if bytes[max_idx] == 0xFF {
            return true;
        }
        // skip the post
        if (max_idx + 3) >= len {
            return false;
        }
        let post_len = bytes[max_idx + 1] as usize;
        max_idx += post_len + 4;
    }
}
