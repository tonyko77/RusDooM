//! An "active" level map, where all the map data is expanded and "mutable".
//! Built from an existing MapData.

use crate::angle::Angle;
use crate::map::*;
use crate::map_items::*;
use crate::pixmap::Texture;
use crate::things::Thing;
use crate::utils::*;
use crate::*;
use std::cell::RefCell;

// LineDef flags
const LINE_BLOCKS: u16 = 0x0001;
//const LINE_BLOCKS_MONSTERS: u16 = 0x0002;
const LINE_TWO_SIDED: u16 = 0x0004;
//const LINE_UPPER_UNPEGGED: u16 = 0x0008;
//const LINE_LOWER_UNPEGGED: u16 = 0x0010;
const LINE_SECRET: u16 = 0x0020;
//const LINE_BLOCKS_SND: u16 = 0x0040;
const LINE_NEVER_ON_AMAP: u16 = 0x0080;
const LINE_ALWAYS_ON_AMAP: u16 = 0x0100;

// Automap zoom limits
const DEFAULT_AUTOMAP_ZOOM: f64 = 0.1875;
const AUTOMAP_ZOOM_MIN: f64 = 0.1;
const AUTOMAP_ZOOM_MAX: f64 = 0.8;

// BSP node flag, for signaling leaf nodes, which point to sub-sectors instead of other nodes
const SSECTOR_FLAG: u16 = 0x8000;

const AMAP_MOVE_SPEED: f64 = 800.0;
const AMAP_ZOOM_SPEED: f64 = 0.0625;
const PLAYER_MOVE_SPEED: f64 = 200.0;
const PLAYER_ROT_SPEED: f64 = 1.5;

const AUTOMAP_OLD_STYLE_ARROW: bool = false;

// Level Flags
const FLAG_AUTOMAP_ON: u32 = 1 << 0;
const FLAG_HAS_COMPUTER_MAP: u32 = 1 << 1;
const FLAG_AUTOMAP_EXTRA_COLORS: u32 = 1 << 2;

pub struct ActiveLevel {
    cfg: GameConfig,
    map_data: MapData,
    player: Thing,
    amap_center: Vertex,
    amap_zoom: f64,
    sky: Texture,
    player_x: f64,
    player_y: f64,
    amap_cx: f64,
    amap_cy: f64,
    flags: u32,
    seen_lines: RefCell<Vec<u8>>,
}

impl ActiveLevel {
    pub fn new(cfg: GameConfig, map_idx: usize) -> Self {
        let map_data = cfg.wad().map(map_idx).clone();
        let player = find_player_thing(&map_data);
        let pc = player.pos;
        let amap_center = player.pos;
        let sky = load_sky(&cfg);
        let seen_lines_size = (map_data.linedef_count() + 7) >> 3;
        let flags = 0xFFFF; // TODO should be 0 in normal gameplay
        Self {
            cfg,
            map_data,
            player,
            amap_center,
            amap_zoom: DEFAULT_AUTOMAP_ZOOM,
            sky,
            player_x: pc.x as f64,
            player_y: pc.y as f64,
            amap_cx: amap_center.x as f64,
            amap_cy: amap_center.y as f64,
            flags,
            seen_lines: RefCell::new(vec![0; seen_lines_size]),
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.map_data.name()
    }

    pub fn move_automap_x(&mut self, dx: f64) {
        self.amap_cx += dx * AMAP_MOVE_SPEED;
        let (cv, was_clamped) = clamp_value(self.amap_cx as i32, self.map_data.min_x(), self.map_data.max_x());
        self.amap_center.x = cv;
        if was_clamped {
            self.amap_cx = cv as f64;
        }
    }

    pub fn move_automap_y(&mut self, dy: f64) {
        self.amap_cy += dy * AMAP_MOVE_SPEED;
        let (cv, was_clamped) = clamp_value(self.amap_cy as i32, self.map_data.min_y(), self.map_data.max_y());
        self.amap_center.y = cv;
        if was_clamped {
            self.amap_cy = cv as f64;
        }
    }

    pub fn zoom_automap(&mut self, dzoom: f64) {
        let new_zoom = self.amap_zoom + dzoom * AMAP_ZOOM_SPEED;
        self.amap_zoom = f64::clamp(new_zoom, AUTOMAP_ZOOM_MIN, AUTOMAP_ZOOM_MAX);
    }

    pub fn move_player(&mut self, ellapsed_time: f64) {
        self.translate_player(ellapsed_time, self.player.angle);
    }

    pub fn strafe_player(&mut self, ellapsed_time: f64) {
        self.translate_player(ellapsed_time, self.player.angle - Angle::with_90_deg());
    }

    pub fn rotate_player(&mut self, ellapsed_time: f64) {
        self.player.angle = self.player.angle + ellapsed_time * PLAYER_ROT_SPEED;
    }

    fn translate_player(&mut self, ellapsed_time: f64, angle: Angle) {
        let (dx, dy) = float_polar_translate(ellapsed_time * PLAYER_MOVE_SPEED, angle);
        self.player_x += dx;
        self.player_y += dy;
        self.player.pos = Vertex {
            x: self.player_x as i32,
            y: self.player_y as i32,
        }
    }

    #[inline]
    pub fn toggle_automap(&mut self) {
        self.flags ^= FLAG_AUTOMAP_ON;
    }

    #[inline]
    pub fn is_automap_on(&self) -> bool {
        (self.flags & FLAG_AUTOMAP_ON) != 0
    }

    pub fn paint(&self, painter: &mut dyn Painter) {
        if self.flags & FLAG_AUTOMAP_ON == 0 {
            self.paint_3d_view(painter);
        } else {
            self.paint_automap(painter);
        }
    }

    //---------------
    // private methods
    //---------------

    fn line_was_seen(&self, line_idx: u16) {
        let byte_idx = (line_idx as usize) >> 3;
        let bit_mask = 1 << (line_idx & 0x07);
        let mut vec = self.seen_lines.borrow_mut();
        (*vec)[byte_idx] |= bit_mask;
    }

    fn was_line_seen(&self, line_idx: u16) -> bool {
        let byte_idx = (line_idx as usize) >> 3;
        let bit_mask = 1 << (line_idx & 0x07);
        let vec = self.seen_lines.borrow();
        ((*vec)[byte_idx] & bit_mask) != 0
    }

    fn paint_3d_view(&self, painter: &mut dyn Painter) {
        // TODO implement this .............
        let w = painter.get_screen_width();
        let h = painter.get_screen_height();
        painter.fill_rect(0, 0, w, h, CYAN);
        // TODO properly align the sky with the player's rotation + fill the whole horizon width
        self.sky.paint(0, 0, painter, self.cfg.palette());

        // collect segments, for painting
        let segs = self.player_visible_segments(&self.player);
        let ppos = self.player.pos;
        let width = self.cfg.scr_width() as usize;
        let mut painted = vec![0_u8; width];
        let mut dbg_color = 240_u8;
        for seg in segs.iter() {
            // TODO render the seg CORRECTLY !
            let a1 = Angle::from_vector(ppos, seg.start) - self.player.angle;
            let a2 = Angle::from_vector(ppos, seg.end) - self.player.angle;
            let (x1, _v1, clipped_1) = self.view_angle_to_x(a1, true);
            let (x2, _v2, clipped_2) = self.view_angle_to_x(a2, false);
            for x in x1..x2 {
                if x < 0 || x >= (width as i32) {
                    continue;
                }
                if painted[x as usize] != 0 {
                    continue;
                }
                // ok to paint
                let color = RGB::from(0, dbg_color, dbg_color);
                painter.draw_line(x, 20, x, 50, color);
                // also, mark seg as seen
                painted[x as usize] = 1;
                self.line_was_seen(seg.linedef_idx);
            }
            if dbg_color >= 10 {
                dbg_color -= 10;
            } else {
                dbg_color = 0;
            }
            // TODO TEMP: also draw lines for each seg's edges
            painter.draw_line(x1, 50, x1, 60, if clipped_1 { PINK } else { RED });
            painter.draw_line(x2, 60, x2, 70, if clipped_2 { BLUE } else { GREEN });
        }

        // TODO - TEMP message
        let txt = format!("SEGs: {} / {}", segs.len(), self.map_data.seg_count());
        self.cfg.font().draw_text(3, 15, &txt, WHITE, painter);
    }

    // TODO this is kinda hacky + not very efficient, but if it works, it's OK :))
    fn view_angle_to_x(&self, angle: Angle, _is_start: bool) -> (i32, Angle, bool) {
        let w = self.cfg.scr_width();
        let hfov = self.cfg.half_fov();
        let minus_hfov = hfov * (-1.0);
        if angle <= hfov || angle >= minus_hfov {
            // in view
            let dist = self.cfg.dist_from_screen();
            let dx = (angle.rad().tan() * dist) as i32;
            (w / 2 - dx, angle, false)
        } else if angle < Angle::with_180_deg() {
            // out of view, left edge
            (0, hfov, true)
        } else {
            // out of view, right edge
            (w - 1, minus_hfov, true)
        }
    }

    fn paint_automap(&self, painter: &mut dyn Painter) {
        // clear the screen first
        painter.fill_rect(0, 0, painter.get_screen_width(), painter.get_screen_height(), BLACK);
        // paint the map itself
        for idx in 0..self.map_data.linedef_count() {
            let line = self.map_data.linedef(idx);
            let color = self.pick_automap_line_color(idx as u16, &line);
            if color != BLACK {
                self.draw_automap_line(line.v1, line.v2, color, painter);
            }
        }

        // paint the player, as a white arrow
        let pos = self.player.pos;
        let ang = self.player.angle;
        if AUTOMAP_OLD_STYLE_ARROW {
            let v1 = pos.polar_translate(25.0, ang);
            let v2 = pos.polar_translate(18.0, -ang);
            let v3 = pos.polar_translate(25.0, -ang);
            self.draw_automap_line(v1, v3, WHITE, painter);
            // draw the arrow head + fins
            let a2 = ang + 2.7;
            let a3 = ang - 2.7;
            let va = v1.polar_translate(18.0, a2);
            let vb = v1.polar_translate(18.0, a3);
            self.draw_automap_line(v1, va, WHITE, painter);
            self.draw_automap_line(v1, vb, WHITE, painter);
            let a2 = ang + 2.5;
            let a3 = ang - 2.5;
            let va = v2.polar_translate(13.0, a2);
            let vb = v2.polar_translate(13.0, a3);
            self.draw_automap_line(v2, va, WHITE, painter);
            self.draw_automap_line(v2, vb, WHITE, painter);
            let va = v3.polar_translate(13.0, a2);
            let vb = v3.polar_translate(13.0, a3);
            self.draw_automap_line(v3, va, WHITE, painter);
            self.draw_automap_line(v3, vb, WHITE, painter);
        } else {
            // a dot at the player's actual position
            let p = self.translate_automap_vertex(pos);
            painter.fill_rect(p.x - 1, p.y - 1, 3, 3, GREEN);
            // a line towards the player direction
            let v = pos.polar_translate(40.0, ang);
            self.draw_automap_line(pos, v, WHITE, painter);
        }

        // text with the map name
        let txt = format!("Map: {}", self.name());
        self.cfg.font().draw_text(3, 3, &txt, RED, painter);

        self.temp_paint_segs(painter);
    }

    // TODO TEMPORARY: collect and paint the visible SEG-s
    fn temp_paint_segs(&self, painter: &mut dyn Painter) {
        let segs = self.player_visible_segments(&self.player);
        let txt = format!("Collected SEGs: {} / {}", segs.len(), self.map_data.seg_count());
        self.cfg.font().draw_text(3, 15, &txt, GREY, painter);
        for seg in segs.iter() {
            self.line_was_seen(seg.linedef_idx); // TODO - this should be done in 3D VIEW paint
            self.draw_automap_line(seg.start, seg.end, GREY, painter);
            // also draw segment direction ticks
            let mid = Vertex {
                x: (seg.start.x + seg.end.x) / 2,
                y: (seg.start.y + seg.end.y) / 2,
            };
            let a = Angle::from_vector(seg.start, seg.end) - Angle::with_90_deg();
            let m2 = mid.polar_translate(20.0, a);
            self.draw_automap_line(mid, m2, GREY, painter);
        }
    }
    // select color based on line type
    // TODO some colors may be wrong, or temporary => CHECK against Crispy Doom
    fn pick_automap_line_color(&self, line_idx: u16, line: &LineDef) -> RGB {
        let f = line.flags;
        let extras = (self.flags & FLAG_AUTOMAP_EXTRA_COLORS) != 0;

        // quick return, for lines that should NOT appear on automap
        let dont_show = (f & LINE_NEVER_ON_AMAP) != 0;
        if dont_show && !extras {
            return BLACK;
        }
        // quick return, for lines that were not yet seen
        let was_seen = (f & LINE_ALWAYS_ON_AMAP) != 0 || self.was_line_seen(line_idx);
        if !was_seen {
            if (self.flags & FLAG_HAS_COMPUTER_MAP) == 0 {
                return BLACK;
            } else {
                return CYAN;
            }
        }

        if extras {
            // highlight secrets
            if f & LINE_SECRET != 0 {
                return GREEN;
            }
            // highlight actionable lines
            if line.special_type != 0 {
                return BLUE;
            }
            // if non-special and invisible
            if dont_show {
                return BLACK;
            }
        }

        if f & LINE_TWO_SIDED != 0 {
            let details = self.get_line_details(&line);
            let s1 = details.left_sector.unwrap();
            let s2 = details.right_sector.unwrap();
            return if s1.floor_height != s2.floor_height {
                // stairs
                CHOCO
            } else if s1.ceiling_height != s2.ceiling_height {
                // ceiling diff
                YELLOW
            } else {
                // no height delta => simply don't draw
                BLACK
            };
        }

        if f & LINE_BLOCKS != 0 {
            return RED;
        }

        // TODO temporary - just highlight lines that don't match any of the above
        // (later, the default returned here should be BLACK)
        PINK
    }

    fn draw_automap_line(&self, v1: Vertex, v2: Vertex, color: RGB, painter: &mut dyn Painter) {
        let xv1 = self.translate_automap_vertex(v1);
        let xv2 = self.translate_automap_vertex(v2);
        painter.draw_line(xv1.x, xv1.y, xv2.x, xv2.y, color);
    }

    fn translate_automap_vertex(&self, orig_vertex: Vertex) -> Vertex {
        // scale the original coordinates
        let sv = (orig_vertex - self.amap_center).fscale(self.amap_zoom);
        // translate the scaled coordinates + mirror y
        Vertex {
            x: sv.x + (self.cfg.scr_width() / 2),
            y: (self.cfg.scr_height() / 2) - sv.y,
        }
    }

    fn get_line_details(&self, linedef: &LineDef) -> LineDefDetails {
        let mut details = LineDefDetails {
            left_sidedef: None,
            left_sector: None,
            right_sidedef: None,
            right_sector: None,
        };
        if linedef.left_side_idx != 0xFFFF {
            let side = self.map_data.sidedef(linedef.left_side_idx as usize);
            let sect = self.map_data.sector(side.sector_idx as usize);
            details.left_sidedef = Some(side);
            details.left_sector = Some(sect);
        }
        if linedef.right_side_idx != 0xFFFF {
            let side = self.map_data.sidedef(linedef.right_side_idx as usize);
            let sect = self.map_data.sector(side.sector_idx as usize);
            details.right_sidedef = Some(side);
            details.right_sector = Some(sect);
        }

        // sanity checks
        // TODO: these should be tested during WAD validation
        assert!(details.left_sidedef.is_some());
        assert!(details.left_sector.is_some());
        if linedef.flags & LINE_TWO_SIDED == 0 {
            // NOT 2-sided
            assert!(details.right_sidedef.is_none());
            assert!(details.right_sector.is_none());
        } else {
            // is 2-sided
            assert!(details.right_sidedef.is_some());
            assert!(details.right_sector.is_some());
        }

        details
    }

    fn player_visible_segments(&self, player: &Thing) -> Vec<Seg> {
        let mut sect_collector = Vec::with_capacity(self.map_data.seg_count() >> 1);
        let start_idx = self.map_data.root_bsp_node_idx();
        self.render_node(player, start_idx, &mut sect_collector);
        sect_collector
    }

    fn render_node(&self, player: &Thing, node_idx: u16, seg_collector: &mut Vec<Seg>) {
        if (node_idx & SSECTOR_FLAG) == 0 {
            // NOT a leaf
            let node = self.map_data.bsp_node(node_idx as usize);
            let (kid1, kid2) = node.child_indices_based_on_point_pos(player.pos);
            self.render_node(player, kid1, seg_collector);
            // TODO? if self.check_bounding_box(player, &node.2nd_kid_box_bl, &node.2nd_kid_box_bl)
            self.render_node(player, kid2, seg_collector);
        } else {
            // it's a LEAF => render sector
            self.render_sub_sector(node_idx, seg_collector);
        }
    }

    fn render_sub_sector(&self, sect_idx: u16, seg_collector: &mut Vec<Seg>) {
        let idx = (sect_idx & !SSECTOR_FLAG) as usize;
        let sub_sector_segs = self.map_data.sub_sector(idx);
        for seg in sub_sector_segs {
            if self.is_seg_in_player_fov(&seg) {
                seg_collector.push(seg);
            }
        }
    }

    // TODO I have simplified things, compared to DIY DOOM
    // -> see: https://github.com/amroibrahim/DIYDoom/tree/master/DIYDOOM/Notes010/notes
    // I don't care about clipped angles
    fn is_seg_in_player_fov(&self, seg: &Seg) -> bool {
        // compute the initial, real-world angles from the player to the 2 edges of the SEG
        let a1 = Angle::from_vector(self.player.pos, seg.start);
        let a2 = Angle::from_vector(self.player.pos, seg.end);

        // drop segments which are "orthogonal" to the player's view
        if a1 == a2 {
            return false;
        }
        // drop segments which are oriented AWAY from the player
        let span_v1_to_v2 = a1 - a2;
        if span_v1_to_v2 >= Angle::with_180_deg() {
            return false;
        }

        // prepare some useful data
        let half_fov = self.cfg.half_fov();
        let full_fov = half_fov * 2.0;
        // rotate angles, so that they're relative to player's angle + move FOV to range 0..full_fov
        let a1 = a1 - self.player.angle + half_fov;
        let a2 = a2 - self.player.angle + half_fov;

        // segment MAY BE visible if:
        //  - at least one segment edge is within player's fov
        //  - both edges are outside player's FOV
        a1 < full_fov || a2 < full_fov || a1 < (full_fov + span_v1_to_v2)
    }
}

//--------------------
//  Internal stuff

struct LineDefDetails {
    right_sidedef: Option<SideDef>,
    right_sector: Option<Sector>,
    left_sidedef: Option<SideDef>,
    left_sector: Option<Sector>,
}

fn find_player_thing(map_data: &MapData) -> Thing {
    for idx in 0..map_data.thing_count() {
        let th = map_data.thing(idx);
        if th.type_code() == 1 {
            return th;
        }
    }
    // TODO validate this upon WAD loading, so we can panic here
    panic!("No player thing found in map {}", map_data.name());
}

// TODO (later) pick the texture name based on level: https://doomwiki.org/wiki/Sky
// (DOOM1, DOOM, DOOMU) ExMy => SKYx
fn load_sky(cfg: &GameConfig) -> Texture {
    let name = "SKY1";
    let key = hash_lump_name(name.as_bytes());
    cfg.graphics().get_texture(key).unwrap()
}

/// Clamp a value, but also signal if it was clamped or not
#[inline]
fn clamp_value<T: PartialOrd>(val: T, min: T, max: T) -> (T, bool) {
    if val < min {
        (min, true)
    } else if val > max {
        (max, true)
    } else {
        (val, false)
    }
}

#[inline]
fn float_polar_translate(dist: f64, angle: Angle) -> (f64, f64) {
    let (s, c) = angle.rad().sin_cos();
    (dist * c, dist * s)
}
