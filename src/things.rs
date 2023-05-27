//! Actors = player and monsters.
//!
//! See [things](https://doomwiki.org/wiki/Thing) and
//! [thing types](https://doomwiki.org/wiki/Thing_types) at Doom Wiki.

// TODO temporary !!!
#![allow(dead_code)]

use crate::{angle::*, map_items::Vertex, utils::*};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThingType {
    Player(u8),
    Monster(u8),
    Weapon(u8),
    Ammo(u8, u8),
    ArtifactItem,
    Collectible,
    Key,
    Obstacle,
    Decoration,
    Other(u16),
    // TODO .....
    Unknown,
}

#[derive(Clone, Default)]
pub struct Thing {
    pub pos: Vertex,
    pub angle: Angle,
    type_code: u16,
    flags: u16,
    // TODO (later) fill in other values, based on type code
    // typ: ThingType,
    // radius: u8,
    // height: u8,
    // sprite: [u8; 4],
}

impl Thing {
    pub fn from(lump_data: &[u8]) -> Self {
        assert!(lump_data.len() >= 10);
        let angle_deg = buf_to_i16(&lump_data[4..6]) as i32;
        let angle = Angle::from_degrees(angle_deg);
        let type_code = buf_to_u16(&lump_data[6..8]);

        Self {
            pos: Vertex {
                x: buf_to_i16(&lump_data[0..2]) as i32,
                y: buf_to_i16(&lump_data[2..4]) as i32,
            },
            angle,
            type_code,
            flags: buf_to_u16(&lump_data[8..10]),
        }
    }

    #[inline]
    pub fn type_code(&self) -> u16 {
        self.type_code
    }

    pub fn is_on_skill_level(&self, level: u8) -> bool {
        (0 == (self.flags & 0x10)) // only use stuff from single player
        && (0 != match level {
            0 => 1,
            1 | 2 => self.flags & 0x01,
            3 => self.flags & 0x02,
            4 | 5 => self.flags & 0x04,
            _ => 0,
        })
    }

    pub fn is_waiting_in_ambush(&self) -> bool {
        0 != (self.flags & 0x08)
    }
}
