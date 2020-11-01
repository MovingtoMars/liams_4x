use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::common::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MapPosition {
    pub x: MapUnit,
    pub y: MapUnit,
}

impl std::fmt::Display for MapPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("({}, {})", self.x, self.y))
    }
}

impl MapPosition {
    pub fn new(x: MapUnit, y: MapUnit) -> Self {
        Self { x, y }
    }

    pub fn x_even(self) -> bool {
        self.x % 2 == 0
    }

    pub fn y_even(self) -> bool {
        self.y % 2 == 0
    }

    fn neighbors_for_positions(
        positions: &HashMap<MapPosition, MapUnit>,
        map_width: MapUnit,
        map_height: MapUnit,
        inclusive: bool,
    ) -> HashMap<Self, MapUnit> {
        let mut result = if inclusive { positions.clone() } else { HashMap::new() };

        for (position, distance) in positions {
            for (neighbor, neighbor_distance) in position.neighbors_at_distance(map_width, map_height, 1, inclusive) {
                if !positions.contains_key(&neighbor) {
                    result.insert(neighbor, distance + neighbor_distance);
                }
            }
        }

        result
    }

    pub fn neighbors_at_distance(
        self,
        map_width: MapUnit,
        map_height: MapUnit,
        distance: MapUnit,
        inclusive: bool,
    ) -> HashMap<Self, MapUnit> {
        if distance == 0 {
            let mut result = HashMap::new();
            if inclusive {
                result.insert(self, 0);
            }
            result
        } else if distance == 1 {
            let mut result = HashMap::new();
            let neighbors = &[
                self.top(),
                self.bottom(),
                self.top_left(),
                self.top_right(),
                self.bottom_left(),
                self.bottom_right(),
            ];
            for &pos in neighbors {
                if pos.x >= 0 && pos.y >= 0 && pos.x < map_width && pos.y < map_height {
                    result.insert(pos, 1);
                }
            }

            if inclusive {
                result.insert(self, 0);
            }

            result

        } else if distance > 1 {
            let neighbors_at_distance_x = self.neighbors_at_distance(map_width, map_height, distance - 1, true);
            MapPosition::neighbors_for_positions(&neighbors_at_distance_x, map_width, map_height, inclusive)
        } else {
            panic!();
        }
    }

    pub fn top(self) -> Self {
        let Self { x, y } = self;
        Self { x, y: y - 1 }
    }

    pub fn bottom(self) -> Self {
        let Self { x, y } = self;
        Self { x, y: y + 1 }
    }

    pub fn top_left(self) -> Self {
        let Self { x, y } = self;

        match (self.x_even(), self.y_even()) {
            (true, true)   => Self::new(x - 1, y - 1),
            (true, false)  => Self::new(x - 1, y - 1),
            (false, true)  => Self::new(x - 1, y),
            (false, false) => Self::new(x - 1, y),
        }
    }

    pub fn bottom_left(self) -> Self {
        let Self { x, y } = self;

        match (self.x_even(), self.y_even()) {
            (true, true)   => Self::new(x - 1, y),
            (true, false)  => Self::new(x - 1, y),
            (false, true)  => Self::new(x - 1, y + 1),
            (false, false) => Self::new(x - 1, y + 1),
        }
    }

    pub fn top_right(self) -> Self {
        let Self { x, y } = self;

        match (self.x_even(), self.y_even()) {
            (true, true)   => Self::new(x + 1, y - 1),
            (true, false)  => Self::new(x + 1, y - 1),
            (false, true)  => Self::new(x + 1, y),
            (false, false) => Self::new(x + 1, y),
        }
    }

    pub fn bottom_right(self) -> Self {
        let Self { x, y } = self;

        match (self.x_even(), self.y_even()) {
            (true, true)   => Self::new(x + 1, y),
            (true, false)  => Self::new(x + 1, y),
            (false, true)  => Self::new(x + 1, y + 1),
            (false, false) => Self::new(x + 1, y + 1),
        }
    }
}
