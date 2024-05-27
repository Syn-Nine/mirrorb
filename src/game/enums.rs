#![allow(dead_code)]

pub const SCREEN_XRES: usize = 368;
pub const SCREEN_XRES_HALF: usize = SCREEN_XRES / 2;
pub const SCREEN_YRES: usize = 264;
pub const SCREEN_YRES_HALF: usize = SCREEN_YRES / 2;
pub const SCREEN_Y_OFFSET: f32 = 24.0;

pub const TILE_SZ: usize = 16;
pub const TILE_SZ_HALF: usize = TILE_SZ / 2;
pub const TILE_SZ_QTR: usize = TILE_SZ / 4;

pub const MAP_MAX_WIDTH: usize = 15;
pub const MAP_MAX_HEIGHT: usize = MAP_MAX_WIDTH;
pub const MAP_SZ: usize = MAP_MAX_WIDTH * MAP_MAX_HEIGHT;

pub const MAX_PIECES: usize = 24;

pub const CLIPPING_NONE: u8 = 0;
pub const CLIPPING_SOURCE: u8 = 1;

pub const REFLECTOR_INVALID: u16 = 0;
pub const REFLECTOR_TL: u16 = 17;
pub const REFLECTOR_TR: u16 = 18;
pub const REFLECTOR_BL: u16 = 19;
pub const REFLECTOR_BR: u16 = 20;
pub const REFLECTOR_D: u16 = 21;
pub const REFLECTOR_L: u16 = 22;
pub const REFLECTOR_U: u16 = 23;
pub const REFLECTOR_R: u16 = 24;

pub const SOURCE_U: u16 = 33;
pub const SOURCE_D: u16 = 34;
pub const SOURCE_L: u16 = 35;
pub const SOURCE_R: u16 = 36;

pub const BURST: u16 = 57;

pub const BEAM_H: u16 = 41;
pub const BEAM_V: u16 = 42;
pub const BEAM_STOP_U: u16 = 43;
pub const BEAM_STOP_D: u16 = 44;
pub const BEAM_STOP_L: u16 = 45;
pub const BEAM_STOP_R: u16 = 46;

pub const BEAM_SPLIT_INVALID: u16 = 0;
pub const BEAM_SPLIT_TL: u16 = 49;
pub const BEAM_SPLIT_TR: u16 = 50;
pub const BEAM_SPLIT_BL: u16 = 51;
pub const BEAM_SPLIT_BR: u16 = 52;
pub const BEAM_SPLIT_D: u16 = 53;
pub const BEAM_SPLIT_L: u16 = 54;
pub const BEAM_SPLIT_U: u16 = 55;
pub const BEAM_SPLIT_R: u16 = 56;
pub const BEAM_SPLIT_X: u16 = 47;

pub const GRID: u16 = 2;
pub const FLOOR: u16 = 3;
pub const BLOCK: u16 = 5;
pub const ORB: u16 = 6;
pub const ORB_ACTIVE: u16 = 7;

pub const HOLDING_INVALID: u8 = 24;
pub const INVENTORY_IDX_INVALID: u8 = 24;
pub const BOARD_IDX_INVALID: u8 = 255;

pub const MENU_HOVER_INVALID: u8 = 255;
pub const MENU_HOVER_UNDO: u8 = 0;
pub const MENU_HOVER_REDO: u8 = 1;
pub const MENU_HOVER_ROTATE: u8 = 2;
pub const MENU_HOVER_FLIP_H: u8 = 3;
pub const MENU_HOVER_FLIP_V: u8 = 4;
pub const MENU_HOVER_NEXT: u8 = 5;
pub const MENU_HOVER_RESET: u8 = 6;
pub const MENU_HOVER_TRASH: u8 = 7;
