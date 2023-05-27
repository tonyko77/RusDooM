//! Handler for WAD graphics + flats

use crate::{
    pixmap::{PixMap, Texture},
    utils::*,
};
use bytes::Bytes;
use std::collections::HashMap;

pub struct Graphics {
    patches: HashMap<u64, Bytes>,
    flats: HashMap<u64, Bytes>,
    pnames: Bytes,
    textures: HashMap<u64, Bytes>,
}

impl Graphics {
    pub fn new() -> Self {
        Graphics {
            patches: HashMap::new(),
            flats: HashMap::new(),
            pnames: Bytes::new(),
            textures: HashMap::new(),
        }
    }

    pub fn add_patch(&mut self, name: &str, lump: &Bytes) {
        let key = hash_lump_name(name.as_bytes());
        self.patches.insert(key, lump.clone());
    }

    pub fn add_flat(&mut self, name: &str, lump: &Bytes) {
        let key = hash_lump_name(name.as_bytes());
        self.flats.insert(key, lump.clone());
    }

    pub fn set_patch_names(&mut self, patches: &Bytes) -> Result<(), String> {
        if patches.len() <= 4 {
            return Err(String::from("PNAMES lump size too small"));
        }
        let cnt = buf_to_u32(&patches[0..4]) as usize;
        if patches.len() < (4 + cnt * 8) {
            return Err(String::from("PNAMES lump size too small"));
        }
        // OK
        self.pnames = patches.clone();
        Ok(())
    }

    pub fn add_textures(&mut self, bytes: &Bytes) -> Result<(), String> {
        let len = bytes.len();
        if len <= 8 {
            return Err(format!("TEXTUREx lump size too small: {len}"));
        }
        // number of textures
        let cnt = buf_to_u32(bytes) as usize;
        if len <= 4 + 4 * cnt {
            return Err(format!("TEXTUREx lump size too small: {len}"));
        }
        // extract bytes for each texture
        for t in 0..cnt {
            let offs = buf_to_u32(&bytes[4 + 4 * t..]) as usize;
            if len <= (offs + 28) {
                return Err(format!("TEXTUREx entry #{t} out of bounds: len={len} < ofs={offs}"));
            }
            let key = hash_lump_name(&bytes[offs..offs + 8]);
            let patch_count = buf_to_u16(&bytes[offs + 20..]) as usize;
            let tex_len = 22 + 10 * patch_count;
            if len < (offs + tex_len) {
                return Err(format!("TEXTUREx entry #{t} out of bounds: len={len} < ofs={offs}"));
            }
            let tex_bytes = bytes.slice(offs..offs + tex_len);
            self.textures.insert(key, tex_bytes);
        }

        Ok(())
    }

    pub fn get_patch(&self, key: u64) -> Option<PixMap> {
        self.patches.get(&key).map(|bytes| PixMap::from_patch(&bytes))
    }

    pub fn get_flat(&self, key: u64) -> Option<PixMap> {
        self.flats.get(&key).map(|bytes| PixMap::from_flat(&bytes))
    }

    pub fn get_texture(&self, key: u64) -> Option<Texture> {
        // get texture
        let tex_bytes = self.textures.get(&key)?;
        let width = buf_to_u16(&tex_bytes[12..14]);
        let height = buf_to_u16(&tex_bytes[14..16]);
        let patch_cnt = buf_to_u16(&tex_bytes[20..22]) as usize;
        let mut texture = Texture::new(width, height, patch_cnt);
        // get all patches for this texture
        for idx in 0..patch_cnt {
            let pofs = 22 + 10 * idx;
            let x_orig = buf_to_i16(&tex_bytes[(pofs + 0)..(pofs + 2)]);
            let y_orig = buf_to_i16(&tex_bytes[(pofs + 2)..(pofs + 4)]);
            let patch_idx = buf_to_u16(&tex_bytes[(pofs + 4)..(pofs + 6)]) as usize;
            let name = std::str::from_utf8(&self.pnames[(patch_idx * 8 + 4)..(patch_idx * 8 + 12)]).unwrap();
            let patch_key = hash_lump_name(&self.pnames[(patch_idx * 8 + 4)..(patch_idx * 8 + 12)]);
            let patch_bytes = self
                .patches
                .get(&patch_key)
                .expect(format!("PATCH bytes not found: {name}").as_str());
            texture.add_patch(patch_bytes, x_orig, y_orig);
        }
        Some(texture)
    }
}
