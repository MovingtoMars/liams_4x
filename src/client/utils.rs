use ncollide2d::math::Point;

use crate::client::constants::*;
use crate::common::TilePosition;

pub fn get_tile_window_pos(position: TilePosition) -> Point<f32> {
    let y = if position.x % 2 == 0 {
        (position.y as f32 + 0.5) * TILE_HEIGHT
    } else {
        (position.y as f32 + 1.0) * TILE_HEIGHT
    };

    let x = (position.x as f32) * TILE_SMALL_WIDTH + TILE_WIDTH * 0.5;

    Point::new(x, y)
}
