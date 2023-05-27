//! DIY Doom Engine, written in Rust :)
//!
//! * see [Recreating DOOM on YouTube](https://www.youtube.com/playlist?list=PLi77irUVkDasNAYQPr3N8nVcJLQAlANva)
//! * see [DIY Doom on GitHub](https://github.com/amroibrahim/DIYDoom)

// This magic line prevents the opening of a terminal when launching a release build
#![cfg_attr(not(any(test, debug_assertions)), windows_subsystem = "windows")]

use rustoom::*;

const SCR_WIDTH: i32 = 480;
const SCR_HEIGHT: i32 = 360;
const PIXEL_SIZE: i32 = 2;
const SLEEP_KIND: SleepKind = SleepKind::YIELD;

//const WAD_PATH: &str = "s:\\DOOM_Quake\\IWADs\\HERETIC.WAD";
//const WAD_PATH: &str = "s:\\DOOM_Quake\\IWADs\\DOOM2.WAD";
const WAD_PATH: &str = "DOOM1.WAD";

fn main() -> Result<(), String> {
    // build the game engine
    let wad_data = WadData::load(WAD_PATH, true)?;
    let cfg = GameConfig::new(wad_data, SCR_WIDTH, SCR_HEIGHT);
    let mut doom_game = DoomGame::new(cfg)?;

    // main game loop
    let sdl_config = SdlConfiguration::new("RusTooM", SCR_WIDTH, SCR_HEIGHT, PIXEL_SIZE, SLEEP_KIND);
    run_sdl_loop(&sdl_config, &mut doom_game)?;

    println!("RusTooM finished OK :)");
    Ok(())
}
