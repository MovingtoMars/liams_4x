use std::collections::HashMap;

use ncollide2d::shape::{ConvexPolygon, Shape};
use ncollide2d::math::{Isometry, Translation, Point, Vector};
use ncollide2d::query::PointQuery;

use crate::client::constants::*;
use crate::common::MapPosition;

use super::object::ObjectType;
use super::utils::get_tile_window_pos;

pub struct Hitbox {
    pub shape: Box<dyn Shape<f32>>,
    pub isometry: Isometry<f32>,
    pub z: f32, // -ve is into screen, +ve is out of screen
}

impl Hitbox {
    pub fn set_tile_pos(&mut self, position: MapPosition) {
        let render_pos = get_tile_window_pos(position);
        self.isometry = Isometry::new(Vector::new(render_pos.x + TILE_WIDTH / 2.0, render_pos.y + TILE_HEIGHT / 2.0), 0.0);
    }

    pub fn tile(pos: Point<f32>) -> Self {
        let points = &[
            Point::new(-0.5 * TILE_INNER_SMALL_WIDTH,  0.5 * TILE_INNER_HEIGHT),
            Point::new( 0.5 * TILE_INNER_SMALL_WIDTH,  0.5 * TILE_INNER_HEIGHT),
            Point::new( 0.5 * TILE_INNER_WIDTH,        0.0),

            Point::new( 0.5 * TILE_INNER_SMALL_WIDTH, -0.5 * TILE_INNER_HEIGHT),
            Point::new(-0.5 * TILE_INNER_SMALL_WIDTH, -0.5 * TILE_INNER_HEIGHT),
            Point::new(-0.5 * TILE_INNER_WIDTH,        0.0),
        ];

        Self {
            shape: Box::new(ConvexPolygon::try_from_points(points).unwrap()),
            isometry: Isometry::new(Vector::new(pos.x + TILE_WIDTH / 2.0, pos.y + TILE_HEIGHT / 2.0), 0.0),
            z: 0.0,
        }
    }

    pub fn civilian(pos: Point<f32>) -> Self {
        let points = &[
            Point::new(-UNIT_SHORT_WIDTH,  0.5 * UNIT_HEIGHT),
            Point::new( 0.0,               0.5 * UNIT_HEIGHT),
            Point::new( 0.0,              -0.5 * UNIT_HEIGHT),

            Point::new(-UNIT_SHORT_WIDTH, -0.5 * UNIT_HEIGHT),
            Point::new(-UNIT_WIDTH,        0.0),
        ];

        Self {
            shape: Box::new(ConvexPolygon::try_from_points(points).unwrap()),
            isometry: Isometry::new(Vector::new(pos.x + TILE_WIDTH / 2.0, pos.y + TILE_HEIGHT / 2.0), 0.0),
            z: 1.0,
        }
    }

    pub fn soldier(pos: Point<f32>) -> Self {
        let points = &[
            Point::new(0.0,               0.5 * UNIT_HEIGHT),

            Point::new(UNIT_SHORT_WIDTH,  0.5 * UNIT_HEIGHT),
            Point::new(UNIT_WIDTH,        0.0),
            Point::new(UNIT_SHORT_WIDTH, -0.5 * UNIT_HEIGHT),

            Point::new(0.0,              -0.5 * UNIT_HEIGHT),
        ];

        Self {
            shape: Box::new(ConvexPolygon::try_from_points(points).unwrap()),
            isometry: Isometry::new(Vector::new(pos.x + TILE_INNER_WIDTH / 2.0, pos.y + TILE_INNER_HEIGHT / 2.0), 0.0),
            z: 1.0,
        }
    }
}

pub fn get_hovered_object<'a>(
    mouse_window_x: f32,
    mouse_window_y: f32,
    map_offset: &Translation<f32>,
    hitboxes: &HashMap<ObjectType, Hitbox>,
) -> Option<ObjectType> {
    let point = map_offset.inverse() * Point::new(mouse_window_x, mouse_window_y);

    let mut result: Option<(ObjectType, f32)> = None;

    for (&object, hitbox) in hitboxes {
        if hitbox.shape.contains_point(&hitbox.isometry, &point) {
            match result {
                Option::Some((_, earlier_z)) => {
                    if hitbox.z > earlier_z  {
                        result = Some((object, hitbox.z));
                    }
                }
                Option::None => {
                    result = Some((object, hitbox.z));
                }
            }
        }
    }

    result.map(|(object, _)| object)
}
