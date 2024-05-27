use crate::mgfw;
use crate::mgfw::ecs::mgui;

use super::game;
use super::scene;
use super::game_state;
use super::enums::*;

const MENU_INVALID: u8 = 0;
//
const MENU_MAIN: u8 = 1;
const MENU_GAME: u8 = 2;
const MENU_NOTIFICATION_POPUP: u8 = 19;

const TRANSITION_MODE_INVALID: u8 = 0;
const TRANSITION_MODE_FLASH: u8 = 1;

pub struct UIData {
    click_delay: u8,
    holding_idx: u8,
    inventory_idx: u8,
    board_idx: u8,
    menu_hover_idx: u8,
    source_hover: u8,
    beam_origin: u8,
    beam_alpha: f32,
    beam_hold: bool,

    menu: u8,
    win_timer: u8,
    transition_mode: u8,
    transition_timer: u16,
    transition_offsets: [u8; 8 * 11],

    board_left: i32,
    board_top: i32,
    menu_left: i32,
    menu_top: i32,
}


pub fn initialize(cache: &mut game::GameData, world: &mut mgfw::ecs::World) {
    
    let data = &mut cache.ui_data;

    data.click_delay = 0;
    data.holding_idx = HOLDING_INVALID;
    data.inventory_idx = INVENTORY_IDX_INVALID;
    data.board_idx = BOARD_IDX_INVALID;
    data.menu_hover_idx = MENU_HOVER_INVALID;
    data.source_hover = BOARD_IDX_INVALID;
    data.beam_origin = BOARD_IDX_INVALID;
    data.beam_alpha = 0.0;
    data.beam_hold = false;

    data.menu = MENU_MAIN;
    data.transition_mode = TRANSITION_MODE_INVALID;
    data.transition_timer = 0;
    data.win_timer = 0;

    data.board_left = 48;
    data.board_top = 48;

}


// core loop
#[rustfmt::skip]
pub fn update(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) -> bool {
    let mut expect_blown = false;

    if 0 != cache.frame % 2 { return false; } // 600 hz

    let mx = world.mouse_x;
    let my = world.mouse_y;

    if MENU_MAIN == cache.ui_data.menu {
        update_main(cache, heap, world);
    }

    if TRANSITION_MODE_INVALID != cache.ui_data.transition_mode {
        update_transition(cache, heap, world);
        // skip the rest while transitioning
        return true;
    }


    // nothing else to do here if not in the game
    if MENU_GAME != cache.ui_data.menu { return false; }

    // update ui positioning
    let uiwidth = (4 + 1 + heap.scene_data.sz) * 16;
    let uiheight = 4 * heap.scene_data.num_pcs as usize + 4 * 18 + 2;
    cache.ui_data.board_left = (SCREEN_XRES_HALF - uiwidth / 2) as i32;
    cache.ui_data.board_top = (SCREEN_YRES_HALF - (heap.scene_data.sz * 8 - 8)) as i32;

    cache.ui_data.menu_left = cache.ui_data.board_left + (heap.scene_data.sz as i32 + 1) * 16;
    cache.ui_data.menu_top = (SCREEN_YRES_HALF - uiheight / 2) as i32 + 8;

    world.entity_set_position_xy(cache.tilemap_ent, cache.ui_data.board_left as f32, cache.ui_data.board_top as f32);
    world.entity_set_position_xy(cache.reflector_ent, cache.ui_data.board_left as f32, cache.ui_data.board_top as f32);
    world.entity_set_position_xy(cache.beam_ent, cache.ui_data.board_left as f32, cache.ui_data.board_top as f32);
    
    world.entity_set_position_xy(cache.game_menu_ent, (cache.ui_data.menu_left + 5) as f32, (cache.ui_data.menu_top + 0 * 16) as f32);
    world.entity_set_position_xy(cache.game_menu2_ent, (cache.ui_data.menu_left + 5) as f32, cache.ui_data.menu_top as f32 + 2.5 * 18.0);
    world.entity_set_position_xy(cache.inventory_ent, cache.ui_data.menu_left as f32, cache.ui_data.menu_top as f32 + 4.0 * 18.0 + 2.0);
    world.entity_set_position_xy(cache.pieces_ent, cache.ui_data.menu_left as f32, cache.ui_data.menu_top as f32 + 4.0 * 18.0 + 2.0);

    world.entity_set_position_xy(cache.logo_mini_ent, cache.ui_data.menu_left as f32 + 32.0, cache.ui_data.menu_top as f32 - 18.0);

    if !cache.final_level {
        world.entity_set_text(cache.level_ent, format!("Level: {}", cache.player_data.level_displayed));
    } else {
        world.entity_set_text(cache.level_ent, format!("FINAL LEVEL!"));
    }
    let textwidth = world.text_get_width(cache.level_ent);
    world.entity_set_position_xy(cache.level_ent, (cache.ui_data.board_left as usize + 8 * heap.scene_data.sz - textwidth / 2) as f32, cache.ui_data.board_top as f32 - 16.0);

    if 0 != cache.frame % 8 { return false; } // 150 hz

    if cache.ui_data.click_delay > 0 {
        cache.ui_data.click_delay -= 1;
    }


    ///////////////////////////////////////////////////////////////////////////
    // game menu
    let mut menu_data = vec![1, 2, 3, 5, 6, 7];
    cache.ui_data.menu_hover_idx = MENU_HOVER_INVALID;
    
    if HOLDING_INVALID == cache.ui_data.holding_idx {
        let menu_left = cache.ui_data.menu_left + 5;
        let menu_top = cache.ui_data.menu_top + 0 * 16;
        let menu_right = menu_left + 3 * 18;
        let menu_bottom = menu_top + 2 * 18;

        if mx > menu_left && mx < menu_right && my > menu_top && my < menu_bottom {
            let xx = ((mx - menu_left) as f32 / 18.0).floor() as u8;
            let yy = ((my - menu_top) as f32 / 18.0).floor() as u8;
            let idx = yy * 3 + xx;
            cache.ui_data.menu_hover_idx = idx;

            if (!cache.level_complete && idx != MENU_HOVER_NEXT) ||
               (cache.level_complete && idx == MENU_HOVER_NEXT && !cache.final_level)
               {
                menu_data[idx as usize] += 8;
            }
        }
    }
    if cache.level_complete && !cache.final_level {
        if cache.ui_data.win_timer > 0 {
            cache.ui_data.win_timer -= 1;
        } else {
            cache.ui_data.win_timer = 64;
        }
        if 31 < cache.ui_data.win_timer { menu_data[MENU_HOVER_NEXT as usize] = 15; }
    }

    world.entity_set_tilemap(cache.game_menu_ent, cache.ui_tiles_ent, 3, &menu_data);

    ///////////////////////////////////////////////////////////////////////////
    // game menu 2
    let mut menu_data = vec![4, 0, 8];
    
    if HOLDING_INVALID == cache.ui_data.holding_idx {
        let menu_left = cache.ui_data.menu_left + 5;
        let menu_top = cache.ui_data.menu_top + (2.5 * 18.0) as i32;
        let menu_right = menu_left + 3 * 18;
        let menu_bottom = menu_top + 1 * 18;

        if mx > menu_left && mx < menu_right && my > menu_top && my < menu_bottom && !cache.level_complete {
            let xx = ((mx - menu_left) as f32 / 18.0).floor() as u8;
            if 0 == xx {
                menu_data[0] += 8;
                cache.ui_data.menu_hover_idx = MENU_HOVER_RESET;
            
            } else if 2 == xx {
                menu_data[2] += 8;
                cache.ui_data.menu_hover_idx = MENU_HOVER_TRASH;
            }
        }
    }
    
    world.entity_set_tilemap(cache.game_menu2_ent, cache.ui_tiles_ent, 3, &menu_data);

    if TRANSITION_MODE_INVALID == cache.ui_data.transition_mode && 0 < cache.blackout_alpha {
        cache.blackout_alpha -= 1;
        world.entity_set_alpha(cache.blackout_ent, cache.blackout_alpha as f32 / 30.0);
    }

    if cache.level_complete {
        return false;
    }

    
    let sz = heap.scene_data.sz;
    let sz2 = heap.scene_data.sz2;
    let sz16 = sz as i32 * 16;

    let board_left = cache.ui_data.board_left;
    let board_top = cache.ui_data.board_top;
    let board_right = board_left + sz16;
    let board_bottom = board_top + sz16;
    
    ///////////////////////////////////////////////////////////////////////////
    // hovering location
    cache.ui_data.board_idx = BOARD_IDX_INVALID;

    if mx > board_left + 16 && mx < board_right - 16 && my > board_top + 16 && my < board_bottom - 16 {
        let xx = ((mx - board_left - 16) as f32 / 16.0).floor() as usize;
        let yy = ((my - board_top - 16) as f32 / 16.0).floor() as usize;
        let idx = (1 + yy) * sz + xx + 1;
        if heap.scene_data.tilemap[idx] == FLOOR || heap.scene_data.tilemap[idx] == FLOOR+1 {
            cache.ui_data.board_idx = idx as u8;
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // holding piece    
    if cache.ui_data.holding_idx != HOLDING_INVALID {
        let idx = cache.ui_data.holding_idx as usize;
        let tile = cache.pieces[idx].tile + 8;
        world.entity_set_tilemap(cache.holding_ent, cache.tileset_ent, 1, &vec![tile]);
        world.entity_set_visibility(cache.holding_ent, true);
        let wx = mx - 8;
        let wy = my - 8;
        world.entity_set_position_xy(cache.holding_ent, wx as f32, wy as f32);

        // snap
        if BOARD_IDX_INVALID != cache.ui_data.board_idx {
            let idx = cache.ui_data.board_idx;
            let xx = idx % sz as u8;
            let yy = (idx - xx) / sz as u8;

            let xx = xx as f32 * 16.0 + board_left as f32;
            let yy = yy as f32 * 16.0 + board_top as f32;

            world.entity_set_position_xy(cache.holding_ent, xx, yy);
        }
        
    } else {
        world.entity_set_visibility(cache.holding_ent, false);
    }

    
    ///////////////////////////////////////////////////////////////////////////
    // check for hovering over inventory;
    cache.ui_data.inventory_idx = INVENTORY_IDX_INVALID;
    let inv_left = cache.ui_data.menu_left;
    let inv_right = inv_left + 4 * 16;
    let inv_top = cache.ui_data.menu_top + 4 * 18 + 2;
    let inv_bottom = inv_top + 4 * heap.scene_data.num_pcs as i32;

    if mx > inv_left && mx < inv_right && my > inv_top && my < inv_bottom {
        let xx = ((mx - inv_left) as f32 / 16.0).floor() as u8;
        let yy = ((my - inv_top) as f32 / 16.0).floor() as u8;
        let idx = yy * 4 + xx;
        if BOARD_IDX_INVALID == cache.pieces[idx as usize].board_idx {
            cache.ui_data.inventory_idx = idx;
        }
    }

    
    ///////////////////////////////////////////////////////////////////////////
    // hovering over cell
    let mut pcs = match heap.scene_data.num_pcs {
        8 => vec![17, 18, 23, 22, 19, 20, 24, 21],
        16 => vec![17, 18, 23, 22, 19, 20, 24, 21, 17, 18, 23, 22, 19, 20, 24, 21],
        24 => vec![17, 18, 23, 22, 19, 20, 24, 21, 17, 18, 23, 22, 19, 20, 24, 21, 17, 18, 23, 22, 19, 20, 24, 21],
        _ => vec![],
    };
    
    // erase pieces from inventory if they are on the game board
    for i in 0..pcs.len() {
        if BOARD_IDX_INVALID != cache.pieces[i].board_idx || i == cache.ui_data.holding_idx as usize {
            pcs[i] = 0;
        }
    }
    if cache.ui_data.inventory_idx != INVENTORY_IDX_INVALID {
        let idx = cache.ui_data.inventory_idx as usize;
        if 0 != pcs[idx] { pcs[idx] += 8; }
    }
    world.entity_set_tilemap(cache.pieces_ent, cache.tileset_ent, 4, &pcs);

    // update board pieces
    heap.scene_data.reflectors = vec![0; sz2];
    for i in 0..pcs.len() {
        if BOARD_IDX_INVALID != cache.pieces[i].board_idx {
            let idx = cache.pieces[i].board_idx as usize;
            heap.scene_data.reflectors[idx] = cache.pieces[i].tile;
        }
    }
    world.entity_set_tilemap(cache.reflector_ent, cache.tileset_ent, sz, &heap.scene_data.reflectors);

    // update beam source
    cache.ui_data.source_hover = BOARD_IDX_INVALID;
    world.entity_set_visibility(cache.source_ent, false);
    if BOARD_IDX_INVALID == cache.ui_data.beam_origin && HOLDING_INVALID == cache.ui_data.holding_idx &&
        mx > board_left && mx < board_right && my > board_top && my < board_bottom {
        let xx = ((mx - board_left) as f32 / 16.0).floor() as usize;
        let yy = ((my - board_top) as f32 / 16.0).floor() as usize;
        let idx = yy * sz + xx;
        let tile = heap.scene_data.tilemap[idx];
        if SOURCE_U == tile || SOURCE_D == tile || SOURCE_R == tile || SOURCE_L == tile {
            cache.ui_data.source_hover = idx as u8;
            let xx = board_left as usize + xx * 16;
            let yy = board_top as usize + yy * 16;
            world.entity_set_position_xy(cache.source_ent, xx as f32, yy as f32);
            world.entity_set_tilemap(cache.source_ent, cache.tileset_ent, 1, &vec![tile + 4]);
        }
    }
    // show source icon
    if BOARD_IDX_INVALID != cache.ui_data.source_hover || BOARD_IDX_INVALID != cache.ui_data.beam_origin {
        world.entity_set_visibility(cache.source_ent, true);
    }

    // update beam
    if BOARD_IDX_INVALID != cache.ui_data.beam_origin {
        let orig = cache.ui_data.beam_origin as usize;

        // starting location
        let xx = orig % sz;
        let yy = (orig - xx) / sz;
        let xx = board_left as f32 + xx as f32 * 16.0;
        let yy = board_top as f32 + yy as f32 * 16.0;
        let tile = heap.scene_data.tilemap[orig];
        world.entity_set_position_xy(cache.source_ent, xx, yy);
        world.entity_set_tilemap(cache.source_ent, cache.tileset_ent, 1, &vec![tile + 4]);
        
        heap.scene_data.beams[orig] = BEAM_STOP_U + (tile - SOURCE_U);

        //loop {
            let mut keep_going = false;
'outer:
            for y in 1..sz-1 {
                for x in 1..sz-1 {
                    let idx = y * sz + x;
                    
                    let mut tile = 255;
                    let up = idx - sz;
                    let down = idx + sz;
                    let left = idx - 1;
                    let right = idx + 1;

                    let beam_up = heap.scene_data.beams[up];
                    let beam_down = heap.scene_data.beams[down];
                    let beam_left = heap.scene_data.beams[left];
                    let beam_right = heap.scene_data.beams[right];

                    let clipped = heap.scene_data.clipping[idx] as u16;
                    
                    if 0 != heap.scene_data.beams[idx] {

                        // only allow crossover
                        if BEAM_H == heap.scene_data.beams[idx] {
                            // anything above or below going vertical?
                            if BEAM_V == beam_up || BEAM_V == beam_down ||
                               BEAM_SPLIT_TL == beam_up || BEAM_SPLIT_TR == beam_up ||
                               BEAM_SPLIT_BL == beam_down || BEAM_SPLIT_BR == beam_down ||
                               BEAM_SPLIT_L == beam_up || BEAM_SPLIT_R == beam_up ||
                               BEAM_SPLIT_L == beam_down || BEAM_SPLIT_R == beam_down ||
                               BEAM_SPLIT_X == beam_up || BEAM_SPLIT_X == beam_down ||
                               orig == up || orig == down
                            {
                                tile = BEAM_SPLIT_X;
                            }
                        
                        } else if BEAM_V == heap.scene_data.beams[idx] {
                            if BEAM_H == beam_left || BEAM_H == beam_right ||
                            BEAM_SPLIT_TL == beam_left || BEAM_SPLIT_BL == beam_left ||
                            BEAM_SPLIT_TR == beam_right || BEAM_SPLIT_BR == beam_right ||
                            BEAM_SPLIT_U == beam_left || BEAM_SPLIT_U == beam_right ||
                            BEAM_SPLIT_D == beam_left || BEAM_SPLIT_D == beam_right ||
                            BEAM_SPLIT_X == beam_left || BEAM_SPLIT_X == beam_right ||
                            orig == left || orig == right
                            {
                                tile = BEAM_SPLIT_X;
                            }
                        }

                        if 255 != tile {
                            heap.scene_data.beams[idx] = tile;
                            keep_going = true;
                        }
                        continue;
                    }

                    if CLIPPING_NONE == clipped as u8 {                    
                        // check for next to source
                        if up == orig || down == orig { tile = BEAM_V; }
                        if left == orig || right == orig { tile = BEAM_H; }

                        if BEAM_V == beam_up || BEAM_V == beam_down || BEAM_SPLIT_X == beam_up || BEAM_SPLIT_X == beam_down {
                            tile = BEAM_V;
                        }

                        if BEAM_H == beam_left || BEAM_H == beam_right || BEAM_SPLIT_X == beam_left || BEAM_SPLIT_X == beam_right {
                            tile = BEAM_H;
                        }

                        if BEAM_SPLIT_BL == beam_left || BEAM_SPLIT_TL == beam_left || BEAM_SPLIT_BR == beam_right || BEAM_SPLIT_TR == beam_right ||
                           BEAM_SPLIT_U == beam_left || BEAM_SPLIT_D == beam_left || BEAM_SPLIT_U == beam_right || BEAM_SPLIT_D == beam_right {
                            tile = BEAM_H;
                        }

                        if BEAM_SPLIT_BL == beam_down || BEAM_SPLIT_TL == beam_up || BEAM_SPLIT_BR == beam_down || BEAM_SPLIT_TR == beam_up ||
                           BEAM_SPLIT_L == beam_up || BEAM_SPLIT_R == beam_up || BEAM_SPLIT_L == beam_down || BEAM_SPLIT_R == beam_down {
                            tile = BEAM_V;
                        }
                    
                    // blocks
                    } else if BLOCK == clipped {
                        if BEAM_V == beam_up ||
                           BEAM_SPLIT_X == beam_up ||
                           BEAM_SPLIT_TL == beam_up ||
                           BEAM_SPLIT_TR == beam_up ||
                           BEAM_SPLIT_L == beam_up ||
                           BEAM_SPLIT_R == beam_up ||
                           orig == up {
                            tile = BEAM_STOP_U;
                        }
                        if BEAM_V == beam_down ||
                           BEAM_SPLIT_X == beam_down ||
                           BEAM_SPLIT_BL == beam_down ||
                           BEAM_SPLIT_BR == beam_down ||
                           BEAM_SPLIT_L == beam_down ||
                           BEAM_SPLIT_R == beam_down ||
                           orig == down {
                            tile = BEAM_STOP_D;
                        }
                        if BEAM_H == beam_left ||
                           BEAM_SPLIT_X == beam_left ||
                           BEAM_SPLIT_TL == beam_left ||
                           BEAM_SPLIT_BL == beam_left ||
                           BEAM_SPLIT_U == beam_left ||
                           BEAM_SPLIT_D == beam_left ||
                           orig == left {
                            tile = BEAM_STOP_L;
                        }
                        if BEAM_H == beam_right ||
                           BEAM_SPLIT_X == beam_right ||
                           BEAM_SPLIT_BR == beam_right ||
                           BEAM_SPLIT_TR == beam_right ||
                           BEAM_SPLIT_U == beam_right ||
                           BEAM_SPLIT_D == beam_right ||
                           orig == right {
                            tile = BEAM_STOP_R;
                        }

                    // orb
                    } else if ORB == clipped {
                        if BEAM_V == beam_up || BEAM_V == beam_down || BEAM_H == beam_left || BEAM_H == beam_right ||
                            BEAM_SPLIT_BL == beam_left || BEAM_SPLIT_BL == beam_down ||
                            BEAM_SPLIT_BR == beam_right || BEAM_SPLIT_BR == beam_down ||
                            BEAM_SPLIT_TL == beam_left || BEAM_SPLIT_TL == beam_up ||
                            BEAM_SPLIT_TR == beam_right || BEAM_SPLIT_TR == beam_up ||
                            BEAM_SPLIT_L == beam_up || BEAM_SPLIT_L == beam_down ||
                            BEAM_SPLIT_R == beam_up || BEAM_SPLIT_R == beam_down ||
                            BEAM_SPLIT_U == beam_left || BEAM_SPLIT_U == beam_right ||
                            BEAM_SPLIT_D == beam_left || BEAM_SPLIT_D == beam_right ||
                            BEAM_SPLIT_X == beam_left || BEAM_SPLIT_X == beam_right || BEAM_SPLIT_X == beam_up || BEAM_SPLIT_X == beam_down ||
                            orig == left || orig == right || orig == up || orig == down {
                            tile = ORB_ACTIVE;
                        }

                    // reflectors
                    } else if REFLECTOR_BL == clipped {
                        if BEAM_V == beam_up || BEAM_H == beam_right ||
                            BEAM_SPLIT_BR == beam_right ||
                            BEAM_SPLIT_TR == beam_right ||
                            BEAM_SPLIT_TR == beam_up ||
                            BEAM_SPLIT_TL == beam_up ||
                            BEAM_SPLIT_D == beam_right ||
                            BEAM_SPLIT_U == beam_right ||
                            BEAM_SPLIT_L == beam_up ||
                            BEAM_SPLIT_R == beam_up ||
                            BEAM_SPLIT_X == beam_up || BEAM_SPLIT_X == beam_right ||
                            orig == up || orig == right {
                            tile = BEAM_SPLIT_BL;
                        }
                    
                    } else if REFLECTOR_BR == clipped {
                        if BEAM_V == beam_up || BEAM_H == beam_left ||
                            BEAM_SPLIT_BL == beam_left ||
                            BEAM_SPLIT_TL == beam_left ||
                            BEAM_SPLIT_TR == beam_up ||
                            BEAM_SPLIT_TL == beam_up ||
                            BEAM_SPLIT_D == beam_left ||
                            BEAM_SPLIT_U == beam_left ||
                            BEAM_SPLIT_L == beam_up ||
                            BEAM_SPLIT_R == beam_up ||
                            BEAM_SPLIT_X == beam_up || BEAM_SPLIT_X == beam_left ||
                            orig == up || orig == left {
                            tile = BEAM_SPLIT_BR;
                        }
                    
                    } else if REFLECTOR_TL == clipped {
                        if BEAM_V == beam_down || BEAM_H == beam_right ||
                            BEAM_SPLIT_BR == beam_right ||
                            BEAM_SPLIT_TR == beam_right ||
                            BEAM_SPLIT_BL == beam_down ||
                            BEAM_SPLIT_BR == beam_down ||
                            BEAM_SPLIT_D == beam_right ||
                            BEAM_SPLIT_U == beam_right ||
                            BEAM_SPLIT_L == beam_down ||
                            BEAM_SPLIT_R == beam_down ||
                            BEAM_SPLIT_X == beam_down || BEAM_SPLIT_X == beam_right ||
                            orig == down || orig == right {
                            tile = BEAM_SPLIT_TL;
                        }
                    
                    } else if REFLECTOR_TR == clipped {
                        if BEAM_V == beam_down || BEAM_H == beam_left ||
                            BEAM_SPLIT_BL == beam_left ||
                            BEAM_SPLIT_TL == beam_left ||
                            BEAM_SPLIT_BR == beam_down ||
                            BEAM_SPLIT_BL == beam_down ||
                            BEAM_SPLIT_D == beam_left ||
                            BEAM_SPLIT_U == beam_left ||
                            BEAM_SPLIT_L == beam_down ||
                            BEAM_SPLIT_R == beam_down ||
                            BEAM_SPLIT_X == beam_down || BEAM_SPLIT_X == beam_left ||
                            orig == down || orig == left {
                            tile = BEAM_SPLIT_TR;
                        }
                    
                    } else if REFLECTOR_D == clipped {
                        if BEAM_V == beam_up ||
                           BEAM_SPLIT_TR == beam_up ||
                           BEAM_SPLIT_TL == beam_up ||
                           BEAM_SPLIT_L == beam_up ||
                           BEAM_SPLIT_R == beam_up ||
                           BEAM_SPLIT_X == beam_up ||
                           orig == up {
                            tile = BEAM_SPLIT_D;
                        }
                    
                    } else if REFLECTOR_L == clipped {
                        if BEAM_H == beam_right ||
                           BEAM_SPLIT_BR == beam_right ||
                           BEAM_SPLIT_TR == beam_right ||
                           BEAM_SPLIT_U == beam_right ||
                           BEAM_SPLIT_D == beam_right ||
                           BEAM_SPLIT_X == beam_right ||
                           orig == right {
                            tile = BEAM_SPLIT_L;
                        }
                    
                    } else if REFLECTOR_U == clipped {
                        if BEAM_V == beam_down ||
                           BEAM_SPLIT_BR == beam_down ||
                           BEAM_SPLIT_BL == beam_down ||
                           BEAM_SPLIT_L == beam_down ||
                           BEAM_SPLIT_R == beam_down ||
                           BEAM_SPLIT_X == beam_down ||
                           orig == down {
                            tile = BEAM_SPLIT_U;
                        }
                
                    } else if REFLECTOR_R == clipped {
                        if BEAM_H == beam_left ||
                           BEAM_SPLIT_BL == beam_left ||
                           BEAM_SPLIT_TL == beam_left ||
                           BEAM_SPLIT_U == beam_left ||
                           BEAM_SPLIT_D == beam_left ||
                           BEAM_SPLIT_X == beam_left ||
                           orig == left {
                            tile = BEAM_SPLIT_R;
                        }
    
                    }

                    if 255 != tile {
                        heap.scene_data.beams[idx] = tile;
                        keep_going = true;
                        break 'outer;
                    }
                }
            }

            //if !keep_going { break; }
        //}

        if !cache.ui_data.beam_hold {
            cache.ui_data.beam_alpha -= 0.05;
            if 0.0 > cache.ui_data.beam_alpha {
                cache.ui_data.beam_origin = BOARD_IDX_INVALID;
                cache.ui_data.beam_alpha = 0.0;
            }
        }
    }

    world.entity_set_tilemap(cache.beam_ent, cache.tileset_ent, sz, &heap.scene_data.beams);
    world.entity_set_visibility(cache.beam_ent, false);
    if 1.0e-6 < cache.ui_data.beam_alpha {
        world.entity_set_visibility(cache.beam_ent, true);
        world.entity_set_alpha(cache.beam_ent, cache.ui_data.beam_alpha);
    }


    expect_blown

}


pub fn click(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) -> bool {

    if cache.ui_data.click_delay > 0 { return false; }

    let mx = world.mouse_x;
    let my = world.mouse_y;

    // main menu input
    if MENU_MAIN == cache.ui_data.menu {
        if TRANSITION_MODE_INVALID == cache.ui_data.transition_mode {
            let xx = SCREEN_XRES_HALF as i32 - 8;
            let yy = 150;
            if mx >= xx && mx < xx + 16 && my >= yy && my < yy + 16 {
                transition(cache, heap, world);
                cache.ui_data.click_delay = 30;

                world.entity_set_text(cache.copyright_ent, String::from("http://mirrorb.io"));
                world.entity_set_position_xy(cache.copyright_ent, SCREEN_XRES_HALF as f32 - (world.text_get_width(cache.copyright_ent) as f32 * 0.5).floor(), SCREEN_YRES as f32 - 28.0);
            }
        }
        return true;
    }

    // game input
    
    let mut consumed = false;

    let npcs = heap.scene_data.num_pcs as usize;

    if !cache.level_complete {
        // check for inventory item pickup
        if INVENTORY_IDX_INVALID != cache.ui_data.inventory_idx {
            cache.ui_data.holding_idx = cache.ui_data.inventory_idx;
            consumed = true;

        } else if BOARD_IDX_INVALID != cache.ui_data.board_idx  {

            // if holding, place piece
            if HOLDING_INVALID != cache.ui_data.holding_idx {
                
                // is board location empty?
                if 0 == heap.scene_data.reflectors[cache.ui_data.board_idx as usize] {
                    game_state::place_piece(cache, heap, cache.ui_data.holding_idx, cache.ui_data.board_idx);
                    cache.ui_data.holding_idx = HOLDING_INVALID;
                    consumed = true;
                
                // else swap with piece on board
                } else {

                    // find piece on this board location
                    for i in 0..npcs {
                        if cache.ui_data.board_idx == cache.pieces[i].board_idx {
                            game_state::swap_piece(cache, heap, cache.ui_data.holding_idx, cache.ui_data.board_idx, i);
                            cache.ui_data.holding_idx = i as u8;
                            consumed = true;
                            break;
                        }
                    }

                }
            
            // else check for piece pickup
            } else {
                // find piece on this board location
                for i in 0..npcs {
                    if cache.ui_data.board_idx == cache.pieces[i].board_idx {
                        game_state::pickup_piece(cache, heap, i);
                        cache.ui_data.holding_idx = i as u8;
                        consumed = true;
                        break;
                    }
                }
            }

        // check for reset click
        } else if HOLDING_INVALID == cache.ui_data.holding_idx && MENU_HOVER_INVALID != cache.ui_data.menu_hover_idx {
            match cache.ui_data.menu_hover_idx {
                MENU_HOVER_UNDO => game_state::history_undo(cache, heap, world),
                MENU_HOVER_REDO => game_state::history_redo(cache, heap, world),
                MENU_HOVER_ROTATE => game_state::rotate(cache, heap, world),
                MENU_HOVER_FLIP_H => game_state::flip_h(cache, heap, world),
                MENU_HOVER_FLIP_V => game_state::flip_v(cache, heap, world),
                MENU_HOVER_RESET => game_state::reset_level(cache, heap),
                MENU_HOVER_TRASH => game_state::trash_level(cache, heap, world),
                _ => (),
            }
            consumed = true;

        } else {
            cache.ui_data.holding_idx = HOLDING_INVALID;
            consumed = true;

        }
    
    // only options when level is complete
    } else {

        if MENU_HOVER_NEXT == cache.ui_data.menu_hover_idx {
            game_state::next_level(cache, heap, world);
            consumed = true;
        }

    }

    if consumed {
        cache.ui_data.click_delay = 30;
    }


    if cache.ui_data.beam_hold {
        check_win(cache, heap, world);
    }
    if !cache.level_complete {
        cache.ui_data.beam_hold = false;
    }

    scene::update_clipping(cache, heap);

    consumed
}


pub fn click_down(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) -> bool {

    if cache.ui_data.click_delay > 0 { return false; }
    if MENU_MAIN == cache.ui_data.menu { return false; }
    if cache.level_complete { return false; }
    
    let mut consumed = false;

    let sz2 = heap.scene_data.sz2;

    // check for source click
    if BOARD_IDX_INVALID != cache.ui_data.source_hover {
        cache.ui_data.beam_origin = cache.ui_data.source_hover;
        cache.ui_data.beam_hold = true;
        cache.ui_data.beam_alpha = 1.0;

        // reset beams
        heap.scene_data.beams = vec![0; sz2];

        consumed = true;
    
    }

    consumed
}


fn update_main(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {

    let mx = world.mouse_x;
    let my = world.mouse_y;

    let xx = SCREEN_XRES_HALF as i32 - 8;
    let yy = 150;

    if mx >= xx && mx < xx + 16 && my >= yy && my < yy + 16 {
        world.entity_set_tilemap(cache.start_ent, cache.ui_tiles_ent, 1, &vec![7 + 8]);
    } else {

        if cache.ui_data.win_timer > 0 {
            cache.ui_data.win_timer -= 1;
        } else {
            cache.ui_data.win_timer = 255;
        }

        if 127 < cache.ui_data.win_timer {
            world.entity_set_tilemap(cache.start_ent, cache.ui_tiles_ent, 1, &vec![7 + 8]);
        } else {
            world.entity_set_tilemap(cache.start_ent, cache.ui_tiles_ent, 1, &vec![7]);
        }
    }

}


fn check_win(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {

    if cache.level_complete { return; }

    // count active orbs vs num orbs
    let mut nactive = 0;
    for i in 0..heap.scene_data.sz2 {
        if ORB_ACTIVE == heap.scene_data.beams[i] {
            nactive += 1;
        }
    }

    if nactive == heap.scene_data.num_goals {
        scene::reset_scene(cache, heap);
        transition(cache, heap, world);
        cache.level_complete = true;
        cache.ui_data.win_timer = 0;
        cache.ui_data.beam_hold = false;
        cache.ui_data.beam_origin = BOARD_IDX_INVALID;
        cache.ui_data.beam_alpha = 0.0;
    }

}


pub fn transition(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {
    
    cache.ui_data.transition_mode = TRANSITION_MODE_FLASH;
    cache.ui_data.transition_timer = 600;
    cache.blackout_alpha = 30;

    for i in 0..cache.ui_data.transition_offsets.len() {
        cache.ui_data.transition_offsets[i] = world.rnd_range(0..255);
    }

    update_transition(cache, heap, world);

}

fn update_transition(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {

    if 0 < cache.ui_data.transition_timer {
        cache.ui_data.transition_timer -= 1;
        if 0 == cache.ui_data.transition_timer {
            cache.ui_data.transition_mode = TRANSITION_MODE_INVALID;
            if MENU_MAIN == cache.ui_data.menu {
                cache.ui_data.menu = MENU_GAME;
                game_state::next_level(cache, heap, world);
            }
        }
    }

    let mut data = vec![0; 8 * 11];
    let ratio = cache.ui_data.transition_timer as f32 / 600.0 * 1.5;

    for yy in 0..8 {
        for xx in 0..11 {
            let idx = yy * 11 + xx;
            let ofs = cache.ui_data.transition_offsets[idx] as f32 / 256.0 * 0.5;

            let mut an = (mgfw::PI as f32 * (0.1 + ratio - ofs)).sin();
            if 0.0 > an { an = 0.0; }
            let mut c = (1.0 + an * 64.0) as u16;
            if yy > 1 && yy < 6 { c /= 4; }
            if 0 == c { c = 1; }
            if 64 < c { c = 64; }
            data[idx] = c;
        }
    }

    let mut alpha = ratio * 4.0;
    if 1.0 < alpha { alpha = 1.0; }
    if MENU_MAIN == cache.ui_data.menu && alpha < 1.0 {
        world.entity_set_visibility(cache.logo_ent, false);
        world.entity_set_visibility(cache.start_ent, false);
    }

    world.entity_set_color_rgba(cache.transition_ent, 1.0, 1.0, 1.0, alpha);
    world.entity_set_color_rgba(cache.complete_ent, 1.0, 1.0, 1.0, alpha);

    world.entity_set_tilemap(cache.transition_ent, cache.gradient_ent, 11, &data);
    world.entity_set_visibility(cache.transition_ent, TRANSITION_MODE_INVALID != cache.ui_data.transition_mode);
    world.entity_set_visibility(cache.complete_ent, TRANSITION_MODE_INVALID != cache.ui_data.transition_mode && MENU_GAME == cache.ui_data.menu);

    world.entity_set_alpha(cache.blackout_ent, cache.blackout_alpha as f32 / 30.0);
    // hack
    //world.entity_set_visibility(cache.transition_ent, false);

}