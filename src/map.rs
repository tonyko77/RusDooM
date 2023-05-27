//! Store maps from the WAD, just as collections of lump bytes.
//! Used as immutable storage, from which to build each level map when it becomes active.

use crate::{map_items::*, things::Thing, utils::*};
use bytes::Bytes;

// Indexes for various MapData lumps
const IDX_THINGS: usize = 0;
const IDX_LINEDEFS: usize = 1;
const IDX_SIDEDEFS: usize = 2;
const IDX_VERTEXES: usize = 3;
const IDX_SEGS: usize = 4;
const IDX_SSECTORS: usize = 5;
const IDX_NODES: usize = 6;
const IDX_SECTORS: usize = 7;
const IDX_REJECT: usize = 8;
const IDX_BLOCKMAP: usize = 9;
const LUMP_CNT: usize = 10;

pub struct MapData {
    name: String,
    lumps: Box<[Bytes; LUMP_CNT]>,
    bound_min: Vertex,
    bound_max: Vertex,
}

impl MapData {
    pub fn new(name: &str) -> Self {
        let lumps: Box<[Bytes; LUMP_CNT]> = Box::new(Default::default());
        Self {
            name: name.to_string(),
            lumps,
            bound_min: Default::default(),
            bound_max: Default::default(),
        }
    }

    #[inline]
    pub fn is_complete(&self) -> bool {
        self.lumps.iter().all(|b| b.len() > 0)
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.lumps[IDX_VERTEXES].len() / VERTEX_SIZE
    }

    #[inline]
    pub fn vertex(&self, idx: usize) -> Vertex {
        Vertex::from_lump(&self.lumps[IDX_VERTEXES], idx)
    }

    #[inline]
    pub fn thing_count(&self) -> usize {
        self.lumps[IDX_THINGS].len() / THING_SIZE
    }

    #[inline]
    pub fn thing(&self, idx: usize) -> Thing {
        let bytes = checked_slice(&self.lumps[IDX_THINGS], idx, THING_SIZE);
        Thing::from(bytes)
    }

    #[inline]
    pub fn linedef_count(&self) -> usize {
        self.lumps[IDX_LINEDEFS].len() / LINEDEF_SIZE
    }

    #[inline]
    pub fn linedef(&self, idx: usize) -> LineDef {
        LineDef::from_lump(&self.lumps[IDX_LINEDEFS], idx, &self.lumps[IDX_VERTEXES])
    }

    #[inline]
    pub fn sidedef(&self, idx: usize) -> SideDef {
        SideDef::from_lump(&self.lumps[IDX_SIDEDEFS], idx)
    }

    #[inline]
    pub fn sector(&self, idx: usize) -> Sector {
        Sector::from_lump(&self.lumps[IDX_SECTORS], idx)
    }

    #[inline]
    pub fn root_bsp_node_idx(&self) -> u16 {
        ((self.lumps[IDX_NODES].len() / NODE_SIZE) - 1) as u16
    }

    #[inline]
    pub fn bsp_node(&self, idx: usize) -> BspNode {
        BspNode::from_lump(&self.lumps[IDX_NODES], idx)
    }

    #[inline]
    pub fn seg_count(&self) -> usize {
        self.lumps[IDX_SEGS].len() / SEG_SIZE
    }

    pub fn sub_sector(&self, idx: usize) -> Vec<Seg> {
        // from SSECTORS, extract the seg count and first seg index
        let bytes = checked_slice(&self.lumps[IDX_SSECTORS], idx, SSECTOR_SIZE);
        let seg_count = buf_to_u16(&bytes[0..2]) as usize;
        let first_seg_idx = buf_to_u16(&bytes[2..4]) as usize;
        // from SEGS, extract each segment
        let mut seg_collector = Vec::with_capacity(seg_count);
        for i in 0..seg_count {
            let idx = first_seg_idx + i;
            let seg = Seg::from_lump(&self.lumps[IDX_SEGS], idx, &self.lumps[IDX_VERTEXES]);
            seg_collector.push(seg);
        }
        seg_collector
    }

    #[inline]
    pub fn min_x(&self) -> i32 {
        self.bound_min.x
    }

    #[inline]
    pub fn min_y(&self) -> i32 {
        self.bound_min.y
    }

    #[inline]
    pub fn max_x(&self) -> i32 {
        self.bound_max.x
    }

    #[inline]
    pub fn max_y(&self) -> i32 {
        self.bound_max.y
    }

    /// Use the REJECT table to check if there is line of sight between the player and the monster
    pub fn check_line_of_sight(&self, player_sect_idx: u16, monster_sect_idx: u16) -> bool {
        let sector_count = self.lumps[IDX_SECTORS].len() / SECTOR_SIZE;
        let pli = player_sect_idx as usize;
        let moi = monster_sect_idx as usize;
        let bit_idx = moi * sector_count + pli;
        let byte_idx = bit_idx >> 3;
        let bit_mask = 1 << (bit_idx & 0x07);
        (self.lumps[IDX_REJECT])[byte_idx] & bit_mask == 0
    }

    pub fn add_lump(&mut self, lump: &str, bytes: &Bytes) -> bool {
        let idx = match lump {
            "VERTEXES" => IDX_VERTEXES,
            "LINEDEFS" => IDX_LINEDEFS,
            "THINGS" => IDX_THINGS,
            "SIDEDEFS" => IDX_SIDEDEFS,
            "SEGS" => IDX_SEGS,
            "SSECTORS" => IDX_SSECTORS,
            "NODES" => IDX_NODES,
            "SECTORS" => IDX_SECTORS,
            "REJECT" => IDX_REJECT,
            "BLOCKMAP" => IDX_BLOCKMAP,
            _ => usize::MAX,
        };
        // check if it was a valid lump; if not => return false, to signal the end of the map lumps
        // (all the lumps of one map are consecutive, so if we get an invalid one => we're done with this map)
        if idx < LUMP_CNT {
            self.lumps[idx] = bytes.clone();
            if idx == IDX_VERTEXES {
                self.compute_map_bounds();
            }
            true
        } else {
            false
        }
    }

    //----------------------------------

    fn compute_map_bounds(&mut self) {
        let mut bl = self.vertex(0);
        let mut tr = bl;
        for i in 1..self.vertex_count() {
            let v = self.vertex(i);
            bl.x = Ord::min(bl.x, v.x);
            bl.y = Ord::min(bl.y, v.y);
            tr.x = Ord::max(tr.x, v.x);
            tr.y = Ord::max(tr.y, v.y);
        }
        self.bound_min = bl;
        self.bound_max = tr;
    }
}

impl Clone for MapData {
    fn clone(&self) -> Self {
        let lumps: Box<[Bytes; LUMP_CNT]> = Box::new((*self.lumps).clone());
        Self {
            name: self.name.clone(),
            lumps,
            bound_min: self.bound_min,
            bound_max: self.bound_max,
        }
    }
}
