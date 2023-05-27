//! Structs for the various items found in a map.

use crate::{angle::Angle, utils::*};
use std::ops::{Add, Sub};

// Lump item sizes
pub const THING_SIZE: usize = 10;
pub const LINEDEF_SIZE: usize = 14;
pub const SIDEDEF_SIZE: usize = 30;
pub const VERTEX_SIZE: usize = 4;
pub const SEG_SIZE: usize = 12;
pub const SSECTOR_SIZE: usize = 4;
pub const NODE_SIZE: usize = 28;
pub const SECTOR_SIZE: usize = 26;

/// A Vertex is a point in the 2D top-view space of a level map.<br/>
/// **Note:** the Y axis goes *upwards* (towards North), like in a normal xOy system,
/// and not like on screen, where the Y axis goes downwards.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Vertex {
    pub x: i32,
    pub y: i32,
}

impl Vertex {
    #[inline]
    pub fn from_lump(lump: &[u8], idx: usize) -> Self {
        let bytes = checked_slice(lump, idx, VERTEX_SIZE);
        Vertex {
            x: buf_to_i16(&bytes[0..2]) as i32,
            y: buf_to_i16(&bytes[2..4]) as i32,
        }
    }

    #[inline]
    pub fn scale(&self, mul: i32, div: i32) -> Self {
        Self {
            x: self.x * mul / div,
            y: self.y * mul / div,
        }
    }

    #[inline]
    pub fn fscale(&self, mul: f64) -> Self {
        Self {
            x: ((self.x as f64) * mul) as i32,
            y: ((self.y as f64) * mul) as i32,
        }
    }

    #[inline]
    pub fn polar_translate(&self, dist: f64, angle: Angle) -> Self {
        let (s, c) = angle.rad().sin_cos();
        Self {
            x: self.x + ((dist * c) as i32),
            y: self.y + ((dist * s) as i32),
        }
    }
}

impl Add for Vertex {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vertex {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

//----------------------------

pub struct LineDef {
    pub v1: Vertex,
    pub v2: Vertex,
    pub flags: u16,
    pub special_type: u16,
    pub sector_tag: u16,
    pub right_side_idx: u16,
    pub left_side_idx: u16,
}

impl LineDef {
    pub fn from_lump(lump: &[u8], idx: usize, vertex_lump: &[u8]) -> Self {
        let bytes = checked_slice(lump, idx, LINEDEF_SIZE);
        let v1_idx = buf_to_u16(&bytes[0..2]) as usize;
        let v2_idx = buf_to_u16(&bytes[2..4]) as usize;
        Self {
            v1: Vertex::from_lump(vertex_lump, v1_idx),
            v2: Vertex::from_lump(vertex_lump, v2_idx),
            flags: buf_to_u16(&bytes[4..6]),
            special_type: buf_to_u16(&bytes[6..8]),
            sector_tag: buf_to_u16(&bytes[8..10]),
            right_side_idx: buf_to_u16(&bytes[10..12]),
            left_side_idx: buf_to_u16(&bytes[12..14]),
        }
    }
}

//----------------------------

pub struct SideDef {
    pub x_offset: i16,
    pub y_offset: i16,
    pub upper_texture_key: u64,
    pub lower_texture_key: u64,
    pub middle_texture_key: u64,
    pub sector_idx: u16,
}

impl SideDef {
    pub fn from_lump(lump: &[u8], idx: usize) -> Self {
        let bytes = checked_slice(lump, idx, SIDEDEF_SIZE);
        Self {
            x_offset: buf_to_i16(&bytes[0..2]),
            y_offset: buf_to_i16(&bytes[2..4]),
            upper_texture_key: hash_lump_name(&bytes[4..12]),
            lower_texture_key: hash_lump_name(&bytes[12..20]),
            middle_texture_key: hash_lump_name(&bytes[20..28]),
            sector_idx: buf_to_u16(&bytes[28..30]),
        }
    }
}

//----------------------------

pub struct Sector {
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_flat_key: u64,
    pub ceiling_flat_key: u64,
    pub light_level: u16,
    pub special_type: u16,
    pub tag_nr: u16,
}

impl Sector {
    pub fn from_lump(lump: &[u8], idx: usize) -> Self {
        let bytes = checked_slice(lump, idx, SECTOR_SIZE);
        Self {
            floor_height: buf_to_i16(&bytes[0..2]),
            ceiling_height: buf_to_i16(&bytes[2..4]),
            floor_flat_key: hash_lump_name(&bytes[4..12]),
            ceiling_flat_key: hash_lump_name(&bytes[12..20]),
            light_level: buf_to_u16(&bytes[20..22]),
            special_type: buf_to_u16(&bytes[22..24]),
            tag_nr: buf_to_u16(&bytes[24..26]),
        }
    }
}

//----------------------------

pub struct BspNode {
    vect_orig: Vertex,
    vect_dir: Vertex,
    right_child: u16,
    left_child: u16,
}

impl BspNode {
    pub fn from_lump(lump: &[u8], idx: usize) -> Self {
        let bytes = checked_slice(lump, idx, NODE_SIZE);
        let vect = buf_to_i16_vect(&bytes[0..24]);
        Self {
            vect_orig: Vertex {
                x: vect[0] as i32,
                y: vect[1] as i32,
            },
            vect_dir: Vertex {
                x: vect[2] as i32,
                y: vect[3] as i32,
            },
            right_child: buf_to_u16(&bytes[24..26]),
            left_child: buf_to_u16(&bytes[26..28]),
        }
    }

    /// Returns the indices of the children of this node, based on the position of a point:
    /// * if the point is on the *left* side => returns *(left_child_idx, right_child_idx)*
    /// * if the point is on the *right* side => returns *(right_child_idx, left_child_idx)*
    #[inline]
    pub fn child_indices_based_on_point_pos(&self, point: Vertex) -> (u16, u16) {
        let pvect = point - self.vect_orig;
        let cross_product_dir = pvect.x * self.vect_dir.y - pvect.y * self.vect_dir.x;
        if cross_product_dir <= 0 {
            // vertex is on the left side
            (self.left_child, self.right_child)
        } else {
            // vertex is on the right side
            (self.right_child, self.left_child)
        }
    }
}

//----------------------------

pub struct Seg {
    pub start: Vertex,
    pub end: Vertex,
    pub angle: Angle,
    // TODO use linedef idx to mark "Seen" walls, when rendering automap
    pub linedef_idx: u16,
    pub direction_same: bool,
    pub offset: i16,
}

impl Seg {
    pub fn from_lump(lump: &[u8], idx: usize, vertex_lump: &[u8]) -> Self {
        let bytes = checked_slice(lump, idx, SEG_SIZE);
        let start_idx = buf_to_u16(&bytes[0..2]) as usize;
        let end_idx = buf_to_u16(&bytes[2..4]) as usize;
        let seg_angle = buf_to_u16(&bytes[4..6]);
        let angle = Angle::from_segment_angle(seg_angle);
        Self {
            start: Vertex::from_lump(vertex_lump, start_idx),
            end: Vertex::from_lump(vertex_lump, end_idx),
            angle,
            linedef_idx: buf_to_u16(&bytes[6..8]),
            direction_same: 0 == buf_to_u16(&bytes[8..10]),
            offset: buf_to_i16(&bytes[10..12]),
        }
    }
}
