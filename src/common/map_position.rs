#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::common::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TileEdge {
    TopLeft,
    Top,
    TopRight,
    BottomRight,
    Bottom,
    BottomLeft,
}

impl TileEdge {
    pub fn clockwise(self) -> Self {
        use TileEdge::*;
        match self {
            TopLeft     => Top,
            Top         => TopRight,
            TopRight    => BottomRight,
            BottomRight => Bottom,
            Bottom      => BottomLeft,
            BottomLeft  => TopLeft,
        }
    }

    pub fn counterclockwise(self) -> Self {
        use TileEdge::*;
        match self {
            TopLeft     => BottomLeft,
            BottomLeft  => Bottom,
            Bottom      => BottomRight,
            BottomRight => TopRight,
            TopRight    => Top,
            Top         => TopLeft,
        }
    }

    // Starts from TopLeft and goes clockwise
    pub fn index(self) -> usize {
        use TileEdge::*;
        match self {
            TopLeft     => 0,
            Top         => 1,
            TopRight    => 2,
            BottomRight => 3,
            Bottom      => 4,
            BottomLeft  => 5,
        }
    }

    pub fn canonical(self) -> Option<CanonicalTileEdge> {
        match self {
            Self::TopLeft => Some(CanonicalTileEdge::TopLeft),
            Self::Top => Some(CanonicalTileEdge::Top),
            Self::TopRight => Some(CanonicalTileEdge::TopRight),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalTileEdge {
    TopLeft,
    Top,
    TopRight,
}

impl CanonicalTileEdge {
    pub fn general(self) -> TileEdge {
        match self {
            Self::TopLeft => TileEdge::TopLeft,
            Self::Top => TileEdge::Top,
            Self::TopRight => TileEdge::TopRight,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgePosition(pub TilePosition, pub CanonicalTileEdge);

impl EdgePosition {
    pub fn boundary_tile_and_edges(self) -> [(TilePosition, TileEdge); 2] {
        use TileEdge::*;
        let Self(tile, edge) = self;

        match edge {
            CanonicalTileEdge::TopLeft     => [(tile.top_left(), BottomRight), (tile, TopLeft)],
            CanonicalTileEdge::Top         => [(tile.top(), Bottom), (tile, Top)],
            CanonicalTileEdge::TopRight    => [(tile.top_right(), BottomLeft), (tile, TopRight)],
        }
    }

    pub fn top_left(self) -> Self {
        use CanonicalTileEdge::*;
        let Self(tile, edge) = self;

        match edge {
            TopLeft => Self(tile.top_left(), TopRight),
            Top => Self(tile.top_left(), TopRight),
            TopRight => Self(tile, Top),
        }
    }

    pub fn top_right(self) -> Self {
        use CanonicalTileEdge::*;
        let Self(tile, edge) = self;

        match edge {
            TopLeft => Self(tile, Top),
            Top => Self(tile.top_right(), TopLeft),
            TopRight => Self(tile.top_right(), TopLeft),
        }
    }

    pub fn bottom_left(self) -> Self {
        use CanonicalTileEdge::*;
        let Self(tile, edge) = self;

        match edge {
            TopLeft => Self(tile.bottom_left(), Top),
            Top => Self(tile, TopLeft),
            TopRight => Self(tile.bottom_right(), TopLeft),
        }
    }

    pub fn bottom_right(self) -> Self {
        use CanonicalTileEdge::*;
        let Self(tile, edge) = self;

        match edge {
            TopLeft => Self(tile.bottom_left(), TopRight),
            Top => Self(tile, TopRight),
            TopRight => Self(tile.bottom_right(), Top),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TilePosition {
    pub x: MapUnit,
    pub y: MapUnit,
}

impl std::fmt::Display for TilePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("({}, {})", self.x, self.y))
    }
}

impl TilePosition {
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
        positions: &HashMap<TilePosition, MapUnit>,
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
            TilePosition::neighbors_for_positions(&neighbors_at_distance_x, map_width, map_height, inclusive)
        } else {
            panic!();
        }
    }

    pub fn neighbor(self, edge: TileEdge) -> Self {
        use TileEdge::*;
        match edge {
            TopLeft => self.top_left(),
            Top => self.top(),
            TopRight => self.top_right(),
            BottomRight => self.bottom_right(),
            Bottom => self.bottom(),
            BottomLeft => self.bottom_left(),
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
