use crate::game::scene::update_clipping;
use crate::mgfw;

use super::game;
use super::game::GameData;
use super::game::GameDataHeap;
use super::scene;
use super::enums::*;

use std::fs::{self, File};
use std::io::{self, BufRead};

// Heap Data
#[derive(Clone)]
pub struct Block {
    pub sz: usize,
    pub norbs: usize,
    pub npcs: usize,
    pub map: Vec<u8>,
}

impl Block {
    pub fn empty() -> Block {
        Block {
            sz: 0,
            norbs: 0,
            npcs: 0,
            map: Vec::new(),
        }
    }
}

pub struct LevelData {
    options: Vec<Vec<Block>>,
}

impl LevelData {
    pub fn new() -> LevelData {
        LevelData {
            options: Vec::new(),
        }
    }
}

pub struct HistoryData {
    pieces: [u8; MAX_PIECES], // piece location
    tilemap: [u16; MAP_SZ],
}

impl HistoryData {
    pub fn new() -> HistoryData {
        HistoryData {
            pieces: [BOARD_IDX_INVALID; MAX_PIECES],
            tilemap: [0; MAP_SZ],
        }
    }
}

// Cache Data
pub struct PlayerData {
    level: usize,
    sub_level: usize,
    level_option: usize,
    pub level_displayed: usize,
}

pub struct PieceData {
    pub tile: u16,
    pub board_idx: u8,
    pub active: bool,
    pub home: u8,
}


pub fn initialize(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World, version: &str) {

    let pcs = vec![17, 18, 23, 22, 19, 20, 24, 21, 17, 18, 23, 22, 19, 20, 24, 21, 17, 18, 23, 22, 19, 20, 24, 21];

    let data = &mut cache.pieces;
    for i in 0..MAX_PIECES {
        data[i].tile = pcs[i];
        data[i].board_idx = BOARD_IDX_INVALID;
        data[i].home = i as u8;
        data[i].active = false;
    }

    // load levels
    println!("Loading maps...");
    let file = File::open("assets/levels.dat").unwrap();

    let mut blocks: Vec<Block> = Vec::new();

    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        if line.len() < 2 { continue; }

        // is this the start of a block?
        let split: Vec<&str> = line.split(',').collect();
        let mut block = Block::empty();
        block.npcs = split[0].parse::<usize>().unwrap();
        block.sz = split[1].parse::<usize>().unwrap();
        block.norbs = split[2].parse::<usize>().unwrap();
        block.map = expand(decompress(split[3].to_string()));
        blocks.push(block);
    }

    let nblocks = blocks.len();

    let mut p_npcs = 0;
    let mut p_sz = 0;
    let mut p_norbs = 0;
    let mut level = 0;

    for block in blocks {
        let npcs = block.npcs;
        let sz = block.sz;
        let norbs = block.norbs;

        if npcs != p_npcs || sz != p_sz || norbs != p_norbs {
            level += 1;
            p_npcs = npcs;
            p_sz = sz;
            p_norbs = norbs;

            heap.level_data.options.push(Vec::new());
        }

        heap.level_data.options[level - 1].push(block.clone());
    }

    // count levels
    let mut nlevels = 0;
    for i in 0..heap.level_data.options.len() {
        nlevels += 1;
        if heap.level_data.options[i].len() > 1 {
            nlevels += 1;
        }
    }
    
    world.entity_set_text(cache.version_ent, format!("mirr/orb {version}, {nlevels} levels, {nblocks} maps"));
    world.entity_set_position_xy(cache.version_ent, SCREEN_XRES_HALF as f32 - (world.text_get_width(cache.version_ent) as f32 * 0.5).floor(), SCREEN_YRES as f32 - 16.0);

    // init player
    let data = &mut cache.player_data;
    data.level = 0;
    data.sub_level = 0;
    data.level_displayed = 0;

}

pub fn next_level(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {

    if 0 == cache.player_data.level {
        cache.player_data.level = 1;
        cache.player_data.sub_level = 0;
        cache.player_data.level_option = 0;

    } else {
        if 1 < cache.player_data.level {
            cache.player_data.sub_level = (cache.player_data.sub_level + 1) % 2;
        }
        if 0 == cache.player_data.sub_level {
            cache.player_data.level += 1;
            cache.player_data.level_option = world.rnd_range(0..heap.level_data.options[cache.player_data.level-1].len());
        } else {
            cache.player_data.level_option = (cache.player_data.level_option + 1) % heap.level_data.options[cache.player_data.level-1].len();
        }
    }

    println!("next level: {}, sub-level: {}, option: {}", cache.player_data.level, cache.player_data.sub_level, cache.player_data.level_option);

    if cache.player_data.level == heap.level_data.options.len() && cache.player_data.sub_level == (heap.level_data.options[cache.player_data.level-1].len() - 1) % 2 {
        println!("Final Level!");
        cache.final_level = true;
    }
    
    heap.scene_data = scene::build_scene(&heap.level_data.options[cache.player_data.level-1][cache.player_data.level_option]);

    // randomly flip and rotate starting layout
    if world.rnd() < 0.5 { flip_h(cache, heap, world); }
    if world.rnd() < 0.5 { flip_v(cache, heap, world); }
    if world.rnd() < 0.5 { rotate(cache, heap, world); }
    if world.rnd() < 0.5 { rotate(cache, heap, world); }
    if world.rnd() < 0.5 { rotate(cache, heap, world); }

    update_clipping(cache, heap);

    world.entity_set_tilemap(cache.tilemap_ent, cache.tileset_ent, heap.scene_data.sz, &heap.scene_data.tilemap);
    
    world.entity_set_visibility(cache.source_ent, false);    
    world.entity_set_visibility(cache.tilemap_ent, true);
    world.entity_set_visibility(cache.reflector_ent, true);
    world.entity_set_visibility(cache.beam_ent, true);
    world.entity_set_visibility(cache.anim_ent, true);
    world.entity_set_visibility(cache.game_menu_ent, true);
    world.entity_set_visibility(cache.game_menu2_ent, true);
    world.entity_set_visibility(cache.inventory_ent, true);
    world.entity_set_visibility(cache.pieces_ent, true);
    world.entity_set_visibility(cache.level_ent, true);
    world.entity_set_visibility(cache.logo_mini_ent, true);

    // update inventory geometry
    let inv = vec![GRID; heap.scene_data.num_pcs as usize];
    world.entity_set_tilemap(cache.inventory_ent, cache.tileset_ent, 4, &inv);

    let pcs = match heap.scene_data.num_pcs {
        8 => vec![17, 18, 23, 22, 19, 20, 24, 21],
        16 => vec![17, 18, 23, 22, 19, 20, 24, 21, 17, 18, 23, 22, 19, 20, 24, 21],
        24 => vec![17, 18, 23, 22, 19, 20, 24, 21, 17, 18, 23, 22, 19, 20, 24, 21, 17, 18, 23, 22, 19, 20, 24, 21],
        _ => vec![],
    };
    world.entity_set_tilemap(cache.pieces_ent, cache.tileset_ent, 4, &pcs);

    cache.level_complete = false;
    if 1 == cache.player_data.level {
        cache.player_data.level_displayed = 1;
    } else {
        cache.player_data.level_displayed = (cache.player_data.level - 1) * 2 + cache.player_data.sub_level;
    }
    
    clear_history(cache, heap);
    push_history(cache, heap);

}


pub fn reset_level(cache: &mut GameData, heap: &mut GameDataHeap) {

    let mut update_history = false;
    for i in 0..heap.scene_data.num_pcs as usize {
        if BOARD_IDX_INVALID != cache.pieces[i].board_idx {
            update_history = true;
            break;
        }
    }

    scene::reset_scene(cache, heap);

    if update_history {
        push_history(cache, heap);
    }
}


pub fn trash_level(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {
    
    cache.player_data.level_option = world.rnd_range(0..heap.level_data.options[cache.player_data.level-1].len());
    
    heap.scene_data = scene::build_scene(&heap.level_data.options[cache.player_data.level-1][cache.player_data.level_option]);
    
    world.entity_set_tilemap(cache.tilemap_ent, cache.tileset_ent, heap.scene_data.sz, &heap.scene_data.tilemap);
    
    scene::reset_scene(cache, heap);

    clear_history(cache, heap);
    push_history(cache, heap);
}


pub fn flip_h(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {

    let sz = heap.scene_data.sz;

    // create temporary structure
    let mut scene = scene::SceneData::new(sz);
    scene.num_goals = heap.scene_data.num_goals;
    scene.num_pcs = heap.scene_data.num_pcs;

    // copy in flipped tilemap
    for yy in 0..sz {
        for xx in 0..sz {
            let src = yy * sz + xx;
            let dst = yy * sz + (sz - 1 - xx);
            if 0 == yy || sz-1 == yy || 0 == xx || sz-1 == xx {
                scene.tilemap[src] = heap.scene_data.tilemap[src];
            } else {
                scene.tilemap[dst] = heap.scene_data.tilemap[src];
            }
        }
    }

    let o = 8;
    let o2 = o * 2;

    // REFLECTOR_TL, REFLECTOR_TR, REFLECTOR_U, REFLECTOR_L, REFLECTOR_BL, REFLECTOR_BR, REFLECTOR_R, REFLECTOR_D
    let flip_pairs = [
        (0,    1),    (2,    2),    (3,    6),    (4,    5),    (7,    7),
        (0+o,  1+o),  (2+o,  2+o),  (3+o,  6+o),  (4+o,  5+o),  (7+o,  7+o),
        (0+o2, 1+o2), (2+o2, 2+o2), (3+o2, 6+o2), (4+o2, 5+o2), (7+o2, 7+o2),
    ];

    // flip pieces
    for i in 0..flip_pairs.len() {
        let lhs = flip_pairs[i].0;
        let rhs = flip_pairs[i].1;

        let mut lidx = cache.pieces[lhs].board_idx as usize;
        if BOARD_IDX_INVALID != lidx as u8 {
            let xx = lidx % sz;
            let yy = (lidx - xx) / sz;
            lidx = yy * sz + (sz - 1 - xx);
        }

        let mut ridx = cache.pieces[rhs].board_idx as usize;
        if BOARD_IDX_INVALID != ridx as u8 {
            let xx = ridx % sz;
            let yy = (ridx - xx) / sz;
            ridx = yy * sz + (sz - 1 - xx);
        }

        cache.pieces[lhs].board_idx = ridx as u8;
        cache.pieces[rhs].board_idx = lidx as u8;
    }

    heap.scene_data = scene;

    world.entity_set_tilemap(cache.tilemap_ent, cache.tileset_ent, heap.scene_data.sz, &heap.scene_data.tilemap);
    push_history(cache, heap);

}

pub fn flip_v(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {

    let sz = heap.scene_data.sz;

    // create temporary structure
    let mut scene = scene::SceneData::new(sz);
    scene.num_goals = heap.scene_data.num_goals;
    scene.num_pcs = heap.scene_data.num_pcs;

    // copy in flipped tilemap
    for yy in 0..sz {
        for xx in 0..sz {
            let src = yy * sz + xx;
            let dst = (sz - 1 - yy) * sz + xx;
            if 0 == yy || sz-1 == yy || 0 == xx || sz-1 == xx {
                scene.tilemap[src] = heap.scene_data.tilemap[src];
            } else {
                scene.tilemap[dst] = heap.scene_data.tilemap[src];
            }
        }
    }

    let o = 8;
    let o2 = o * 2;

    // REFLECTOR_TL, REFLECTOR_TR, REFLECTOR_U, REFLECTOR_L, REFLECTOR_BL, REFLECTOR_BR, REFLECTOR_R, REFLECTOR_D
    let flip_pairs = [
        (0,    4),    (1,    5),    (2,    7),    (3,    3),    (6,    6),
        (0+o,  4+o),  (1+o,  5+o),  (2+o,  7+o),  (3+o,  3+o),  (6+o,  6+o),
        (0+o2, 4+o2), (1+o2, 5+o2), (2+o2, 7+o2), (3+o2, 3+o2), (6+o2, 6+o2),
    ];

    // flip pieces
    for i in 0..flip_pairs.len() {
        let lhs = flip_pairs[i].0;
        let rhs = flip_pairs[i].1;

        let mut lidx = cache.pieces[lhs].board_idx as usize;
        if BOARD_IDX_INVALID != lidx as u8 {
            let xx = lidx % sz;
            let yy = (lidx - xx) / sz;
            lidx = (sz - 1 - yy) * sz + xx;
        }

        let mut ridx = cache.pieces[rhs].board_idx as usize;
        if BOARD_IDX_INVALID != ridx as u8 {
            let xx = ridx % sz;
            let yy = (ridx - xx) / sz;
            ridx = (sz - 1 - yy) * sz + xx;
        }

        cache.pieces[lhs].board_idx = ridx as u8;
        cache.pieces[rhs].board_idx = lidx as u8;
    }

    heap.scene_data = scene;

    world.entity_set_tilemap(cache.tilemap_ent, cache.tileset_ent, heap.scene_data.sz, &heap.scene_data.tilemap);
    push_history(cache, heap);

}


pub fn rotate(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {

    let sz = heap.scene_data.sz;
    let sz2 = heap.scene_data.sz2;

    // create temporary structure
    let mut scene = scene::SceneData::new(sz);
    scene.num_goals = heap.scene_data.num_goals;
    scene.num_pcs = heap.scene_data.num_pcs;

    let mut tiles = vec![0; sz2];

    // first copy rotated tiles into temp array
    for yy in 0..sz {
        for xx in 0..sz {
            let src = yy * sz + xx;
            let x1 = sz - 1 - yy;
            let y1 = xx;
            let dst = y1 * sz + x1;

            if 0 == yy || sz-1 == yy || 0 == xx || sz-1 == xx {
                tiles[src] = heap.scene_data.tilemap[src];
            } else {
                tiles[dst] = heap.scene_data.tilemap[src];
            }
        }
    }

    // then copy temp array back into tilemap
    for i in 0..tiles.len() {
        scene.tilemap[i] = tiles[i];
    }


    let o = 8;
    let o2 = o * 2;

    // REFLECTOR_TL, REFLECTOR_TR, REFLECTOR_U, REFLECTOR_L, REFLECTOR_BL, REFLECTOR_BR, REFLECTOR_R, REFLECTOR_D
    let flip_pairs = [
        (0,    1),    (1,    5),    (5,    4),    (4,    0),    (2,    6),    (6,    7),    (7,    3),    (3,    2),
        (0+o,  1+o),  (1+o,  5+o),  (5+o,  4+o),  (4+o,  0+o),  (2+o,  6+o),  (6+o,  7+o),  (7+o,  3+o),  (3+o,  2+o),
        (0+o2, 1+o2), (1+o2, 5+o2), (5+o2, 4+o2), (4+o2, 0+o2), (2+o2, 6+o2), (6+o2, 7+o2), (7+o2, 3+o2), (3+o2, 2+o2),
    ];

    // destination set
    let mut pieces =[BOARD_IDX_INVALID; MAX_PIECES];

    // first assign rotated board indexes
    for i in 0..flip_pairs.len() {
        let lhs = flip_pairs[i].0;
        let rhs = flip_pairs[i].1;
        pieces[rhs] = cache.pieces[lhs].board_idx;
    }

    // then rotate positions
    for i in 0..scene.num_pcs as usize {
        let bidx = pieces[i] as usize;
        if BOARD_IDX_INVALID as usize != bidx {
            let x0 = bidx % sz;
            let y0 = (bidx - x0) / sz;
            let x1 = sz - 1 - y0;
            let y1 = x0;
            pieces[i] = (y1 * sz + x1) as u8;
        }

        cache.pieces[i].board_idx = pieces[i];
    }

    heap.scene_data = scene;

    world.entity_set_tilemap(cache.tilemap_ent, cache.tileset_ent, heap.scene_data.sz, &heap.scene_data.tilemap);
    push_history(cache, heap);

}


fn decompress(txt: String) -> String {
    let mut out = txt;
    
    out = out.replace("\"", "lA");
    out = out.replace("/", ".&");
    out = out.replace("|", ".i");
    out = out.replace("`", "cB");
    out = out.replace("?", "~~");
    out = out.replace(">", ".z");
    out = out.replace("<", ".h");
    out = out.replace(";", "#A");
    out = out.replace(":", ".F");
    out = out.replace("}", "kA");
    out = out.replace("{", "jA");
    out = out.replace("]", "bB");
    out = out.replace("[", ".g");
    out = out.replace("=", "$A");
    out = out.replace("_", "aB");
    out = out.replace("0", ".!");
    out = out.replace("9", ".f");
    out = out.replace("8", "hA");
    out = out.replace("7", "iA");
    out = out.replace("6", ".E");
    out = out.replace("5", ".D");
    out = out.replace("4", ".~");
    out = out.replace("2", "gA");
    out = out.replace("1", ".e");
    out = out.replace("+", ".a");
    out = out.replace("-", "fA");
    out = out.replace(")", "dA");
    out = out.replace("(", ".C");
    out = out.replace("*", "cA");
    out = out.replace("&", "eA");
    out = out.replace("^", ".B");
    out = out.replace("%", ".b");
    out = out.replace("$", ".d");
    out = out.replace("#", ".c");
    out = out.replace("@", ".A");
    out = out.replace("!", "bA");
    out = out.replace("~", "aA");

    out
}

fn expand(txt: String) -> Vec<u8> {

    let mut out = txt;

    out = out.replace(".", "_");

    let mut rhs = String::new();
    for i in 1..27 {
        let lhs = char::from(96 + i);
        rhs = format!("{rhs}.");
        out = out.replace(lhs, rhs.as_str());
    }

    let mut rhs = String::new();
    for i in 1..27 {
        let lhs = char::from(64 + i);
        rhs = format!("{rhs}x");
        out = out.replace(lhs, rhs.as_str());
    }

    out = out.replace("_", "o");

    out.as_bytes().to_vec()
}


pub fn place_piece(cache: &mut GameData, heap: &mut GameDataHeap, holding_idx: u8, board_idx: u8) {

    cache.pieces[holding_idx as usize].board_idx = board_idx;
    push_history(cache, heap);
}


pub fn swap_piece(cache: &mut GameData, heap: &mut GameDataHeap, holding_idx: u8, board_idx: u8, swapping_idx: usize) {

    cache.pieces[holding_idx as usize].board_idx = board_idx;
    cache.pieces[swapping_idx].board_idx = BOARD_IDX_INVALID;
    push_history(cache, heap);
}


pub fn pickup_piece(cache: &mut GameData, heap: &mut GameDataHeap, pickup_idx: usize) {
    
    cache.pieces[pickup_idx].board_idx = BOARD_IDX_INVALID;
    push_history(cache, heap);
}


fn push_history(cache: &mut GameData, heap: &mut GameDataHeap) {

    let mut h = HistoryData::new();
    
    // copy piece information
    for i in 0..MAX_PIECES {
        h.pieces[i] = cache.pieces[i].board_idx;
    }

    // copy tile information
    for i in 0..heap.scene_data.tilemap.len() {
        h.tilemap[i] = heap.scene_data.tilemap[i];
    }

    // trim history tail
    for _ in (cache.history_idx+1)..heap.history.len() {
        heap.history.pop();
    }

    heap.history.push(h);
    cache.history_idx = heap.history.len() - 1;

}

fn clear_history(cache: &mut GameData, heap: &mut GameDataHeap) {
    heap.history.clear();
    cache.history_idx = 0;
}

pub fn history_undo(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {

    if cache.history_idx == 0 { return; }

    // move history pointer
    cache.history_idx -= 1;

    // copy piece information
    for i in 0..MAX_PIECES {
        cache.pieces[i].board_idx = heap.history[cache.history_idx].pieces[i];
    }

    // copy tile information
    for i in 0..heap.scene_data.tilemap.len() {
        heap.scene_data.tilemap[i] = heap.history[cache.history_idx].tilemap[i];
    }

    world.entity_set_tilemap(cache.tilemap_ent, cache.tileset_ent, heap.scene_data.sz, &heap.scene_data.tilemap);
}

pub fn history_redo(cache: &mut game::GameData, heap: &mut game::GameDataHeap, world: &mut mgfw::ecs::World) {

    if cache.history_idx == heap.history.len() - 1 { return; }

    // move history pointer
    cache.history_idx += 1;

    // copy piece information
    for i in 0..MAX_PIECES {
        cache.pieces[i].board_idx = heap.history[cache.history_idx].pieces[i];
    }

    // copy tile information
    for i in 0..heap.scene_data.tilemap.len() {
        heap.scene_data.tilemap[i] = heap.history[cache.history_idx].tilemap[i];
    }

    world.entity_set_tilemap(cache.tilemap_ent, cache.tileset_ent, heap.scene_data.sz, &heap.scene_data.tilemap);
}