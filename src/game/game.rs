use crate::mgfw;

use super::game_state;
use super::scene;
use super::ui;
use super::enums::*;

pub struct GameDataHeap {
    // WARNING: Anything below this line is not in cache!
    pub scene_data: scene::SceneData,
    pub level_data: game_state::LevelData,
    pub history: Vec<game_state::HistoryData>,
}

impl Default for GameDataHeap {
    fn default() -> Self {
        GameDataHeap {
            scene_data: scene::SceneData::empty(),
            level_data: game_state::LevelData::new(),
            history: Vec::new(),
        }
    }
}

// cache data
pub struct GameData {
    // system
    pub heap: *mut GameDataHeap,
    pub frame: u8,
    ready: bool,
    pub quit_requested: bool,

    pub player_data: game_state::PlayerData,
    pub ui_data: ui::UIData,
    pub tileset_ent: usize,
    pub tilemap_ent: usize,
    pub reflector_ent: usize,
    pub source_ent: usize,
    pub beam_ent: usize,
    pub anim_ent: usize,

    pub inventory_ent: usize,
    pub pieces_ent: usize,
    pub holding_ent: usize,

    pub pieces: [game_state::PieceData; MAX_PIECES],

    pub game_menu_ent: usize,
    pub game_menu2_ent: usize,

    pub logo_ent: usize,
    pub logo_mini_ent: usize,
    pub start_ent: usize,
    pub ui_tiles_ent: usize,
    pub gradient_ent: usize,
    pub transition_ent: usize,
    pub complete_ent: usize,

    pub level_complete: bool,
    pub level_ent: usize,
    pub blackout_ent: usize,
    pub blackout_alpha: u8,

    pub final_level: bool,
    pub history_idx: usize,
    
    pub copyright_ent: usize,
    pub version_ent: usize,
}


#[rustfmt::skip]
pub fn initialize(cache: &mut GameData, heap: &mut GameDataHeap, world: &mut mgfw::ecs::World) {

    world.parse_world("assets/world.dat");

    let version = "v0.9.0";

    cache.level_complete = false;
    cache.final_level = false;
    cache.history_idx = 0;

    let ent = world.new_entity();
    world.entity_set_billboard(ent, String::from("assets/background.png"));
    world.entity_set_visibility(ent, true);
    world.entity_set_scale_xy(ent, 368.0, 264.0);
    world.entity_set_position_xy(ent, SCREEN_XRES_HALF as f32, SCREEN_YRES_HALF as f32);

    cache.copyright_ent = world.new_entity();
    world.entity_set_text(cache.copyright_ent, String::from("(C) 2024 Daniel 'syn9' Kennedy. http://mirrorb.io"));
    world.entity_set_position_xy(cache.copyright_ent, SCREEN_XRES_HALF as f32 - (world.text_get_width(cache.copyright_ent) as f32 * 0.5).floor(), SCREEN_YRES as f32 - 28.0);
    world.entity_set_color_rgba(cache.copyright_ent, 0.1, 0.2, 0.3, 1.0);
    world.entity_set_visibility(cache.copyright_ent, true);

    cache.version_ent = world.new_entity();
    world.entity_set_color_rgba(cache.version_ent, 0.1, 0.2, 0.3, 1.0);
    world.entity_set_visibility(cache.version_ent, true);


    // todo - consider moving these somewhere else
    cache.tileset_ent = world.new_entity();
    world.entity_set_tileset(cache.tileset_ent, String::from("assets/tiles.png"), 128, 128, 16, 16);

    cache.ui_tiles_ent = world.new_entity();
    world.entity_set_tileset(cache.ui_tiles_ent, String::from("assets/ui.png"), 64, 64, 16, 16);

    cache.gradient_ent = world.new_entity();
    world.entity_set_tileset(cache.gradient_ent, String::from("assets/gradient.png"), 32, 32, 4, 4);

    cache.tilemap_ent = world.new_entity();
    world.entity_set_scale_xy(cache.tilemap_ent, 16.0, 16.0);
    world.entity_set_position_xy(cache.tilemap_ent, 48.0, 48.0);

    cache.reflector_ent = world.new_entity();
    world.entity_set_scale_xy(cache.reflector_ent, 16.0, 16.0);
    world.entity_set_position_xy(cache.reflector_ent, 48.0, 48.0);
    
    cache.source_ent = world.new_entity();
    world.entity_set_scale_xy(cache.source_ent, 16.0, 16.0);

    cache.beam_ent = world.new_entity();
    world.entity_set_scale_xy(cache.beam_ent, 16.0, 16.0);
    world.entity_set_position_xy(cache.beam_ent, 48.0, 48.0);
    world.entity_set_alpha(cache.beam_ent, 0.0);
    
    // cache.anim_ent = world.new_entity();
    // world.entity_set_scale_xy(cache.anim_ent, 16.0, 16.0);
    // world.entity_set_position_xy(cache.anim_ent, 48.0, 48.0);

    cache.logo_mini_ent = world.new_entity();
    world.entity_set_billboard(cache.logo_mini_ent, String::from("assets/logo_mini.png"));
    world.entity_set_scale_xy(cache.logo_mini_ent, 66.0, 24.0);

    cache.game_menu_ent = world.new_entity();
    world.entity_set_scale_xy(cache.game_menu_ent, 18.0, 18.0);

    cache.game_menu2_ent = world.new_entity();
    world.entity_set_scale_xy(cache.game_menu2_ent, 18.0, 18.0);

    cache.inventory_ent = world.new_entity();
    world.entity_set_scale_xy(cache.inventory_ent, 16.0, 16.0);

    cache.pieces_ent = world.new_entity();
    world.entity_set_scale_xy(cache.pieces_ent, 16.0, 16.0);

    cache.level_ent = world.new_entity();

    game_state::initialize(cache, heap, world, version);
    ui::initialize(cache, world);

    
    cache.holding_ent = world.new_entity();
    world.entity_set_scale_xy(cache.holding_ent, 16.0, 16.0);

    cache.logo_ent = world.new_entity();
    world.entity_set_billboard(cache.logo_ent, String::from("assets/logo.png"));
    world.entity_set_visibility(cache.logo_ent, true);
    world.entity_set_scale_xy(cache.logo_ent, 294.0, 134.0);
    world.entity_set_position_xy(cache.logo_ent, 188.0, 67.0);

    cache.start_ent = world.new_entity();
    world.entity_set_scale_xy(cache.start_ent, 18.0, 18.0);
    world.entity_set_visibility(cache.start_ent, true);
    world.entity_set_position_xy(cache.start_ent, SCREEN_XRES_HALF as f32 - 8.0, 150.0);
    world.entity_set_tilemap(cache.start_ent, cache.ui_tiles_ent, 1, &vec![7]);

    cache.blackout_ent = world.new_entity();
    world.entity_set_scale_xy(cache.blackout_ent, SCREEN_XRES as f32, SCREEN_YRES as f32);
    world.entity_set_tilemap(cache.blackout_ent, cache.gradient_ent, 1, &vec![1]);
    world.entity_set_visibility(cache.blackout_ent, true);
    cache.blackout_alpha = 0;
    world.entity_set_alpha(cache.blackout_ent, cache.blackout_alpha as f32 / 30.0);

    cache.transition_ent = world.new_entity();
    world.entity_set_scale_xy(cache.transition_ent, 34.0, 33.0);

    cache.complete_ent = world.new_entity();
    world.entity_set_text(cache.complete_ent, String::from("Level Complete!"));
    world.entity_set_scale_xy(cache.complete_ent, 3.0, 3.0);
    world.entity_set_visibility(cache.complete_ent, true);
    world.entity_set_alpha(cache.complete_ent, 0.0);
    let w = world.text_get_width(cache.complete_ent) as f32;
    world.entity_set_position_xy(cache.complete_ent, SCREEN_XRES_HALF as f32 - w * 1.5, SCREEN_YRES_HALF as f32 - 8.0 * 2.0);



}



pub fn quit_requested(cache: &mut GameData) -> bool {
    cache.quit_requested
}

pub fn shutdown(_cache: &mut GameData, heap: &mut GameDataHeap) {
    // deallocate and overwrite existing memory
    *heap = GameDataHeap::default();

    // re-box and consume
    //let _temp = unsafe { Box::from_raw(cache.heap) };
}


// this gets called by MGFW with input events
#[rustfmt::skip]
pub fn event(cache: &mut GameData, heap: &mut GameDataHeap, world: &mut mgfw::ecs::World, event_id: u8) -> bool {
    let mut consumed = false;

    if mgfw::EVENT_INPUT_KEYBOARD_RELEASED_ESCAPE == event_id {
        cache.quit_requested = true;
        consumed = true;
    
    } else if mgfw::EVENT_INPUT_MOUSE_BUTTON_UP == event_id {
        consumed = ui::click(cache, heap, world);

    } else if mgfw::EVENT_INPUT_MOUSE_BUTTON_DOWN == event_id {
        consumed = ui::click_down(cache, heap, world);
        
    }

    consumed
}


// this gets called by MGFW at 1200hz
#[rustfmt::skip]
pub fn update(cache: &mut GameData, heap: &mut GameDataHeap, world: &mut mgfw::ecs::World) -> bool {
    
    let mut expect_blown = false;
    cache.frame = (cache.frame + 1) % 128;
    //let dt = 1.0 / 1200.0;

    if !cache.ready {
        if 127 == cache.frame {
            cache.ready = true;
        }
        return false;
    }

    // update game subsystems
    expect_blown |= ui::update(cache, heap, world);
    expect_blown
}



