//!  Various utilities

//------------------------------
//  Misc utility functions

#[inline]
pub fn buf_to_u16(buf: &[u8]) -> u16 {
    assert!(buf.len() >= 2);
    (buf[0] as u16) | ((buf[1] as u16) << 8)
}

#[inline]
pub fn buf_to_i16(buf: &[u8]) -> i16 {
    buf_to_u16(buf) as i16
}

#[inline]
pub fn buf_to_u32(buf: &[u8]) -> u32 {
    assert!(buf.len() >= 4);
    (buf[0] as u32) | ((buf[1] as u32) << 8) | ((buf[2] as u32) << 16) | ((buf[3] as u32) << 24)
}

pub fn buf_to_i16_vect(buf: &[u8]) -> Vec<i16> {
    let cnt = buf.len() >> 1;
    let mut vct = Vec::with_capacity(cnt);
    for i in 0..cnt {
        vct.push(buf_to_i16(&buf[i * 2..]));
    }
    vct
}

#[inline]
pub fn checked_slice(buf: &[u8], idx: usize, item_size: usize) -> &[u8] {
    let start = idx * item_size;
    let end = start + item_size;
    assert!(end <= buf.len());
    &buf[start..end]
}

/// Convert a lump name into a 64 bit integer, for easier use as key in a hashmap.
/// Since lumps should only use digits, upper case letters and a few simbols
/// => they fall into the range 32-95 (0x20-0x5F)
/// => it is safe to pick only the lower 6 bits of each ASCII character (byte).
pub fn hash_lump_name(name: &[u8]) -> u64 {
    const A: u8 = 'a' as u8;
    const Z: u8 = 'z' as u8;
    let mut key = 0_u64;
    for b in name {
        let bb = match *b {
            A..=Z => *b - 32,
            0 => {
                return key;
            }
            _ => *b,
        };
        key = (key << 8) | (bb as u64);
    }
    key
}

pub fn atoi(s: &str) -> Option<u32> {
    let mut num = 0_u32;
    for b in s.bytes() {
        if b < ('0' as u8) || b > ('9' as u8) {
            return None;
        }
        num = num * 10 + (b as u32) - ('0' as u32);
    }
    Some(num)
}
