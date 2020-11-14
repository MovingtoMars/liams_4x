pub const TILE_WIDTH: f32 = 200.0;
pub const TILE_HEIGHT: f32 = TILE_WIDTH * 174.0 / 200.0;
pub const TILE_SMALL_WIDTH: f32 = TILE_WIDTH * 0.75;

pub const TILE_INNER_WIDTH: f32 = 195.0;
pub const TILE_INNER_HEIGHT: f32 = 170.0;
pub const TILE_INNER_SMALL_WIDTH: f32 = 95.0;

pub const UNIT_SHORT_WIDTH: f32 = 30.0;
pub const UNIT_WIDTH: f32 = 60.0;
pub const UNIT_HEIGHT: f32 = 106.0;

// Tile spritesheet

pub const _SPRITE_TILE_BLANK: usize = 0;
pub const SPRITE_TILE_PLAINS: usize = 1;
pub const SPRITE_TILE_OCEAN: usize = 2;
pub const SPRITE_TILE_HIGHLIGHT: usize = 3;
pub const SPRITE_TILE_MOUNTAIN: usize = 4;

pub const SPRITE_SOLDIER: usize = 6;
pub const SPRITE_CIVILIAN_HIGHLIGHT: usize = 7;
pub const SPRITE_SOLDIER_HIGHLIGHT: usize = 8;

pub const SPRITE_TILE_HIGHLIGHT_BLUE_1: usize = 15;
pub const SPRITE_TILE_HIGHLIGHT_BLUE_2: usize = 15;
pub const SPRITE_TILE_HIGHLIGHT_BLUE_3: usize = 15;

pub const SPRITE_RIVER: usize = 13;

pub const SPRITE_BORDER: usize = 14;

pub const SPRITE_JUNGLE: usize = 18;
pub const SPRITE_FOREST: usize = 19;

pub const SPRITE_RESOURCE_SHEEP: usize = 20;
pub const SPRITE_RESOURCE_HORSES: usize = 21;
pub const SPRITE_RESOURCE_GOLD: usize = 22;
pub const SPRITE_RESOURCE_IRON: usize = 23;
pub const SPRITE_RESOURCE_SILVER: usize = 24;
pub const SPRITE_RESOURCE_NITER: usize = 25;
pub const SPRITE_RESOURCE_COAL: usize = 26;
pub const SPRITE_RESOURCE_WHEAT: usize = 27;

pub const SPRITE_SETTLER: usize = 30;
pub const SPRITE_WORKER: usize = 31;

pub const SPRITE_TILE_DESERT: usize = 70;

// Yield spritesheet
pub const YIELD_ICON_WIDTH: f32 = 16.0;
pub const YIELD_ICON_HEIGHT: f32 = 16.0;

pub const SPRITE_YIELD_FOOD: usize = 0;
pub const SPRITE_YIELD_PRODUCTION: usize = 1;
pub const SPRITE_YIELD_SCIENCE: usize = 2;

// Citizens spritesheet
pub const CITIZEN_ICON_WIDTH: f32 = 100.0;
#[allow(dead_code)]
pub const CITIZEN_ICON_HEIGHT: f32 = 100.0;

pub const SPRITE_CITIZEN_LOCKED: usize = 0;
pub const SPRITE_CITIZEN_NORMAL: usize = 1;
pub const SPRITE_CITIZEN_NONE: usize = 2;
