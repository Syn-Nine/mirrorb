use crate::mgfw;
use crate::mgfw::ecs;
use crate::mgfw::log;

use super::game;
use super::enums::*;
use super::game::GameData;
use super::game::GameDataHeap;
use super::game_state::Block;

use rand;
use rand::prelude::*;


pub type TileMap = Vec<u16>;
pub type ClippingMap = Vec<u8>;

#[derive(Default, Clone, Copy)]
pub struct PieceData {
    pub tile: u16,
    pub board_idx: u8,
    pub active: bool,
    pub home: u8,
    pub available: bool,
}

// heap data
#[derive(Default, Clone)]
pub struct SceneData {
    pub tilemap: TileMap,
    pub clipping: ClippingMap,
    pub reflectors: TileMap,
    pub beams: TileMap,
    pub sz: usize,
    pub sz2: usize,
    pub num_goals: u8,
    pub num_pcs: u8,
}


impl SceneData {
    pub fn new(sz: usize) -> SceneData {
        let sz2 = sz * sz;
        SceneData {
            tilemap: vec![0; sz2],
            clipping: vec![0; sz2],
            reflectors: vec![0; sz2],
            beams: vec![0; sz2],
            sz,
            sz2,
            num_goals: 0,
            num_pcs: 0,
        }
    }

    pub fn empty() -> SceneData {
        SceneData {
            tilemap: Vec::new(),
            clipping: Vec::new(),
            reflectors: Vec::new(),
            beams: Vec::new(),
            sz: 1,
            sz2: 1,
            //pieces: SceneData::init_pieces(),
            num_goals: 0,
            num_pcs: 0,
        }
    }

    /*fn init_pieces() -> [PieceData; MAX_PIECES] {
        let pcs = vec![17, 18, 23, 22, 19, 20, 24, 21, 17, 18, 23, 22, 19, 20, 24, 21, 17, 18, 23, 22, 19, 20, 24, 21];
        assert!(MAX_PIECES == pcs.len());

        let mut data = [PieceData::default(); MAX_PIECES];
        for i in 0..MAX_PIECES {
            data[i].tile = pcs[i];
            data[i].board_idx = BOARD_IDX_INVALID;
            data[i].home = i as u8;
            data[i].active = false;
            data[i].available = true;
        }

        data
    }*/
}


pub fn build_scene(block: &Block) -> SceneData {

    let mut scene = SceneData::new(block.sz);

    let mut rng: ThreadRng = rand::thread_rng();

    let sz = scene.sz;
    let sz2 = scene.sz2;

    // fill with floor
    for y in 1..sz-1 {
        for x in 1..sz-1 {
            let idx = y * sz + x;
            scene.tilemap[idx] = FLOOR + ((y + x) % 2) as u16;
        }
    }

    // set blocks
    for i in 0..sz2 {
        if 46 == block.map[i] && rng.gen::<f32>() < 0.3 {
            scene.tilemap[i] = BLOCK;
        }
    }
    
    // clear corners
    scene.tilemap[0] = 0;
    scene.tilemap[sz-1] = 0;
    scene.tilemap[(sz-1)*sz] = 0;
    scene.tilemap[sz2-1] = 0;

    // set edge arrows
    for i in 1..sz-1 {
        scene.tilemap[i * sz + 0] = SOURCE_R;
        scene.tilemap[(i + 1) * sz - 1] = SOURCE_L;
        scene.tilemap[i] = SOURCE_D;
        scene.tilemap[(sz - 1) * sz + i] = SOURCE_U;
    }

    // set goals
    for i in 0..sz2 {
        if 111 == block.map[i] {
            scene.tilemap[i] = ORB;
            scene.num_goals += 1;
        }
    }

    
    // base clipping
    let mut clip = vec![CLIPPING_NONE; sz2];

    for i in 0..sz2 {
        let tile = scene.tilemap[i];
        if SOURCE_U >= tile && SOURCE_R <= tile {
            clip[i] = CLIPPING_SOURCE;
        }
        if 0 == tile || BLOCK == tile { clip[i] = BLOCK as u8; }
        if ORB == tile { clip[i] = ORB as u8; }
    }

    scene.clipping = clip;
    scene.num_pcs = block.npcs as u8;

    scene

}

pub fn update_clipping(cache: &mut GameData, heap: &mut GameDataHeap) {

    let sz2 = heap.scene_data.sz2;
    let npcs = heap.scene_data.num_pcs as usize;

    let mut clip = vec![CLIPPING_NONE; sz2];

    for i in 0..sz2 {
        let tile = heap.scene_data.tilemap[i];
        if SOURCE_U >= tile && SOURCE_R <= tile {
            clip[i] = CLIPPING_SOURCE;
        }
        if BLOCK == tile { clip[i] = tile as u8; }
        if ORB == tile { clip[i] = tile as u8; }
    }

    for i in 0..npcs {
        if BOARD_IDX_INVALID != cache.pieces[i].board_idx {
            clip[cache.pieces[i].board_idx as usize] = cache.pieces[i].tile as u8;
        }
    }

    heap.scene_data.clipping = clip;

}



pub fn reset_scene(cache: &mut GameData, heap: &mut GameDataHeap) {
    let npcs = heap.scene_data.num_pcs as usize;
    for i in 0..npcs {
        cache.pieces[i].board_idx = BOARD_IDX_INVALID;
    }
}



