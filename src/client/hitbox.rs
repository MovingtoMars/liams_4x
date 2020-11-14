use std::collections::HashMap;
use std::rc::Rc;

use ncollide2d::shape::Ball;
use ncollide2d::shape::{ConvexPolygon, Shape};
use ncollide2d::math::{Isometry, Translation, Point, Vector};
use ncollide2d::query::PointQuery;

use crate::client::constants::*;
use crate::common::TilePosition;
use crate::common::UnitId;
use crate::client::utils::get_tile_window_pos;
use crate::common::UnitType;

#[derive(Clone)]
pub struct Hitbox {
    pub shape: Rc<Box<dyn Shape<f32>>>,
    pub isometry: Isometry<f32>,
    pub z: f32, // -ve is into screen, +ve is out of screen
}

impl Hitbox {
    pub fn set_tile_pos(&mut self, position: TilePosition) {
        let render_pos = get_tile_window_pos(position);
        self.isometry = Isometry::new(Vector::new(render_pos.x + TILE_WIDTH / 2.0, render_pos.y + TILE_HEIGHT / 2.0), 0.0);
    }

    pub fn tile(pos: TilePosition) -> Self {
        let pos = get_tile_window_pos(pos);
        let points = &[
            Point::new(-0.5 * TILE_INNER_SMALL_WIDTH,  0.5 * TILE_INNER_HEIGHT),
            Point::new( 0.5 * TILE_INNER_SMALL_WIDTH,  0.5 * TILE_INNER_HEIGHT),
            Point::new( 0.5 * TILE_INNER_WIDTH,        0.0),

            Point::new( 0.5 * TILE_INNER_SMALL_WIDTH, -0.5 * TILE_INNER_HEIGHT),
            Point::new(-0.5 * TILE_INNER_SMALL_WIDTH, -0.5 * TILE_INNER_HEIGHT),
            Point::new(-0.5 * TILE_INNER_WIDTH,        0.0),
        ];

        Self {
            shape: Rc::new(Box::new(ConvexPolygon::try_from_points(points).unwrap())),
            isometry: Isometry::new(Vector::new(pos.x + TILE_WIDTH / 2.0, pos.y + TILE_HEIGHT / 2.0), 0.0),
            z: 0.0,
        }
    }

    pub fn unit(pos: TilePosition, unit_type: UnitType) -> Self {
        match unit_type {
            UnitType::Civilian => Self::civilian(get_tile_window_pos(pos)),
            UnitType::Soldier => Self::soldier(get_tile_window_pos(pos)),
        }
    }

    fn civilian(pos: Point<f32>) -> Self {
        let points = &[
            Point::new(-UNIT_SHORT_WIDTH,  0.5 * UNIT_HEIGHT),
            Point::new( 0.0,               0.5 * UNIT_HEIGHT),
            Point::new( 0.0,              -0.5 * UNIT_HEIGHT),

            Point::new(-UNIT_SHORT_WIDTH, -0.5 * UNIT_HEIGHT),
            Point::new(-UNIT_WIDTH,        0.0),
        ];

        Self {
            shape: Rc::new(Box::new(ConvexPolygon::try_from_points(points).unwrap())),
            isometry: Isometry::new(Vector::new(pos.x + TILE_WIDTH / 2.0, pos.y + TILE_HEIGHT / 2.0), 0.0),
            z: 1.0,
        }
    }

    fn soldier(pos: Point<f32>) -> Self {
        let points = &[
            Point::new(0.0,               0.5 * UNIT_HEIGHT),

            Point::new(UNIT_SHORT_WIDTH,  0.5 * UNIT_HEIGHT),
            Point::new(UNIT_WIDTH,        0.0),
            Point::new(UNIT_SHORT_WIDTH, -0.5 * UNIT_HEIGHT),

            Point::new(0.0,              -0.5 * UNIT_HEIGHT),
        ];

        Self {
            shape: Rc::new(Box::new(ConvexPolygon::try_from_points(points).unwrap())),
            isometry: Isometry::new(Vector::new(pos.x + TILE_INNER_WIDTH / 2.0, pos.y + TILE_INNER_HEIGHT / 2.0), 0.0),
            z: 1.0,
        }
    }

    pub fn citizen(pos: TilePosition) -> Self {
        let pos = get_tile_window_pos(pos);

        Self {
            shape: Rc::new(Box::new(Ball::new(CITIZEN_ICON_WIDTH / 2.0))),
            isometry: Isometry::new(Vector::new(pos.x + TILE_WIDTH / 2.0, pos.y + TILE_HEIGHT / 2.0), 0.0),
            z: 2.0,
        }
    }
}

#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub enum HitboxKey {
    Tile(TilePosition),
    Unit(UnitId),
    Citizen(TilePosition),
}

pub fn get_hovered_object<'a>(
    mouse_window_x: f32,
    mouse_window_y: f32,
    zoom: f32,
    map_offset: &Translation<f32>,
    hitboxes: &HashMap<HitboxKey, Hitbox>,
) -> Option<HitboxKey> {
    let point = map_offset.inverse() * Point::new(mouse_window_x / zoom, mouse_window_y / zoom);

    let mut result: Option<(HitboxKey, f32)> = None;

    for (&key, hitbox) in hitboxes {
        if hitbox.shape.contains_point(&hitbox.isometry, &point) {
            match result {
                Option::Some((_, earlier_z)) => {
                    if hitbox.z > earlier_z  {
                        result = Some((key, hitbox.z));
                    }
                }
                Option::None => {
                    result = Some((key, hitbox.z));
                }
            }
        }
    }

    result.map(|(key, _)| key)
}
