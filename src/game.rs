//! My wacky DOOM game engine :P

/*
STILL TO DO:
    - I MUST TAKE A BREAK !!!!!!!!!!
    - continue DIY: https://github.com/amroibrahim/DIYDoom/tree/master/DIYDOOM/Notes012/notes

NICE TO HAVE:
    - player - NO LONGER move through walls
    - paint proper SKY for map, from graphics, based on user rotation !!
    - doc comments !!
 */

use crate::level::ActiveLevel;
use crate::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

// key flags - for ALL keys (some only act once on press => they need 2 bits !!)
const KEY_MOVE_FWD: u32 = 1 << 0;
const KEY_MOVE_BACK: u32 = 1 << 1;
const KEY_STRAFE_LEFT: u32 = 1 << 2;
const KEY_STRAFE_RIGHT: u32 = 1 << 3;
const KEY_CURS_UP: u32 = 1 << 4;
const KEY_CURS_DOWN: u32 = 1 << 5;
const KEY_CURS_LEFT: u32 = 1 << 6;
const KEY_CURS_RIGHT: u32 = 1 << 7;
const KEY_USE: u32 = 1 << 8;
const KEY_SHOOT: u32 = 1 << 9;
const KEY_ZOOM_IN: u32 = 1 << 10;
const KEY_ZOOM_OUT: u32 = 1 << 11;

pub struct DoomGame {
    cfg: GameConfig,
    map_idx: usize,
    level: ActiveLevel,
    key_flags: u32,
}

impl DoomGame {
    pub fn new(cfg: GameConfig) -> Result<DoomGame, String> {
        let level = ActiveLevel::new(cfg.clone(), 0);
        let mut engine = DoomGame {
            cfg,
            map_idx: 0,
            level,
            key_flags: 0,
        };
        engine.load_map(0);
        engine.update_state(0.0);
        Ok(engine)
    }

    pub fn load_map(&mut self, idx: usize) {
        if self.map_idx != idx && idx < self.cfg.wad().map_count() {
            self.map_idx = idx;
            self.level = ActiveLevel::new(self.cfg.clone(), idx);
        }
    }
}

impl GraphicsLoop for DoomGame {
    fn handle_event(&mut self, event: &Event) -> bool {
        // check keys
        match event {
            Event::KeyDown { keycode: Some(key), .. } => {
                match key {
                    Keycode::Tab => self.level.toggle_automap(),
                    Keycode::KpPlus => self.key_flags |= KEY_ZOOM_IN,
                    Keycode::KpMinus => self.key_flags |= KEY_ZOOM_OUT,
                    Keycode::Up => self.key_flags |= KEY_CURS_UP,
                    Keycode::Down => self.key_flags |= KEY_CURS_DOWN,
                    Keycode::Left => self.key_flags |= KEY_CURS_LEFT,
                    Keycode::Right => self.key_flags |= KEY_CURS_RIGHT,
                    Keycode::W => self.key_flags |= KEY_MOVE_FWD,
                    Keycode::S => self.key_flags |= KEY_MOVE_BACK,
                    Keycode::A => self.key_flags |= KEY_STRAFE_LEFT,
                    Keycode::D => self.key_flags |= KEY_STRAFE_RIGHT,
                    Keycode::Space | Keycode::E => self.key_flags |= KEY_USE,
                    Keycode::RCtrl | Keycode::LAlt => self.key_flags |= KEY_SHOOT,
                    Keycode::PageUp => {
                        // TODO temp
                        if self.map_idx > 0 {
                            let new_map_idx = self.map_idx - 1;
                            self.load_map(new_map_idx);
                        }
                    }
                    Keycode::PageDown => {
                        // TODO temp
                        if self.map_idx < self.cfg.wad().map_count() - 1 {
                            let new_map_idx = self.map_idx + 1;
                            self.load_map(new_map_idx);
                        }
                    }
                    _ => {}
                }
            }
            Event::KeyUp { keycode: Some(key), .. } => match key {
                Keycode::KpPlus => self.key_flags &= !KEY_ZOOM_IN,
                Keycode::KpMinus => self.key_flags &= !KEY_ZOOM_OUT,
                Keycode::Up => self.key_flags &= !KEY_CURS_UP,
                Keycode::Down => self.key_flags &= !KEY_CURS_DOWN,
                Keycode::Left => self.key_flags &= !KEY_CURS_LEFT,
                Keycode::Right => self.key_flags &= !KEY_CURS_RIGHT,
                Keycode::W => self.key_flags &= !KEY_MOVE_FWD,
                Keycode::S => self.key_flags &= !KEY_MOVE_BACK,
                Keycode::A => self.key_flags &= !KEY_STRAFE_LEFT,
                Keycode::D => self.key_flags &= !KEY_STRAFE_RIGHT,
                Keycode::Space | Keycode::E => self.key_flags &= KEY_USE,
                Keycode::RCtrl | Keycode::LAlt => self.key_flags |= KEY_SHOOT,
                _ => {}
            },
            _ => {}
        }
        true
    }

    fn update_state(&mut self, elapsed_time: f64) -> bool {
        // cursor always rotates and moves player
        match self.key_flags & (KEY_CURS_LEFT | KEY_CURS_RIGHT) {
            KEY_CURS_LEFT => self.level.rotate_player(elapsed_time),
            KEY_CURS_RIGHT => self.level.rotate_player(-elapsed_time),
            _ => {}
        }
        match self.key_flags & (KEY_CURS_UP | KEY_CURS_DOWN) {
            KEY_CURS_UP => self.level.move_player(elapsed_time),
            KEY_CURS_DOWN => self.level.move_player(-elapsed_time),
            _ => {}
        }

        // automap vs player specific movements
        if self.level.is_automap_on() {
            // in automap mode
            match self.key_flags & (KEY_STRAFE_LEFT | KEY_STRAFE_RIGHT) {
                KEY_STRAFE_LEFT => self.level.move_automap_x(-elapsed_time),
                KEY_STRAFE_RIGHT => self.level.move_automap_x(elapsed_time),
                _ => {}
            }
            match self.key_flags & (KEY_MOVE_FWD | KEY_MOVE_BACK) {
                KEY_MOVE_FWD => self.level.move_automap_y(elapsed_time),
                KEY_MOVE_BACK => self.level.move_automap_y(-elapsed_time),
                _ => {}
            }
            match self.key_flags & (KEY_ZOOM_IN | KEY_ZOOM_OUT) {
                KEY_ZOOM_IN => self.level.zoom_automap(elapsed_time),
                KEY_ZOOM_OUT => self.level.zoom_automap(-elapsed_time),
                _ => {}
            }
        } else {
            // in 3D view mode
            match self.key_flags & (KEY_STRAFE_LEFT | KEY_STRAFE_RIGHT) {
                KEY_STRAFE_LEFT => self.level.strafe_player(-elapsed_time),
                KEY_STRAFE_RIGHT => self.level.strafe_player(elapsed_time),
                _ => {}
            }
            match self.key_flags & (KEY_MOVE_FWD | KEY_MOVE_BACK) {
                KEY_MOVE_FWD => self.level.move_player(elapsed_time),
                KEY_MOVE_BACK => self.level.move_player(-elapsed_time),
                _ => {}
            }
        }

        true
    }

    fn paint(&self, painter: &mut dyn Painter) {
        self.level.paint(painter);
    }
}
