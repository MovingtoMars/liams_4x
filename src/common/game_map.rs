use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet};

use crate::common::*;

pub type MapUnit = i16;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameMap {
    // Number of tiles
    width: MapUnit,
    height: MapUnit,

    // tiles[x][y]
    tiles: Vec<Vec<Tile>>,
}

impl GameMap {
    pub fn new(width: MapUnit, height: MapUnit) -> Self {
        let mut tile_cols = Vec::new();
        for x in 0..width {
            let mut tile_col = Vec::new();
            for y in 0..height {
                tile_col.push(Tile {
                    position: TilePosition { x, y },
                    tile_type: TileType::Plains,
                    units: BTreeMap::new(),
                    rivers: BTreeSet::new(),
                    city: None,
                    territory: None,
                    resource: None,
                    vegetation: None,
                    harvested: false,
                });
            }
            tile_cols.push(tile_col);
        }

        GameMap {
            width,
            height,
            tiles: tile_cols,
        }
    }

    pub fn width(&self) -> MapUnit {
        self.width
    }

    pub fn height(&self) -> MapUnit {
        self.height
    }

    #[allow(dead_code)]
    pub fn try_tile(&self, position: TilePosition) -> Option<&Tile> {
        if self.has_tile(position) {
            Some(self.tile(position))
        } else {
            None
        }
    }

    pub fn has_tile(&self, position: TilePosition) -> bool {
        position.x >= 0 && position.y >= 0 && position.x < self.width && position.y < self.height
    }

    pub fn tile(&self, position: TilePosition) -> &Tile {
        &self.tiles[position.x as usize][position.y as usize]
    }

    pub fn tile_mut(&mut self, position: TilePosition) -> &mut Tile {
        &mut self.tiles[position.x as usize][position.y as usize]
    }

    pub fn tiles(&self) -> impl Iterator<Item = &Tile> {
        (0..self.width)
            .into_iter()
            .flat_map(move |x| {
                (0..self.height).into_iter().map(move |y| self.tile(TilePosition { x, y }))
            })
    }

    pub fn add_river(&mut self, pos: CanonicalEdgePosition) -> bool {
        let mut modified = false;
        for (tile, edge) in &pos.boundary_tile_and_edges() {
            if self.has_tile(*tile) {
                self.tile_mut(*tile).rivers.insert(*edge);
                modified = true;
            }
        }
        modified
    }
    pub fn shortest_path(&mut self,s:TilePosition, d:TilePosition) -> Option<Vec<TilePosition>> { // Move this to game_map.rs?
        // Find shortest path using A* algorithm
        let mut open_nodes = BinaryHeap::new();
        let mut visited_nodes = HashSet::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();

        let init_d = s.distance_to(d);
        g_score.insert(s,0);

        open_nodes.push((Reverse(init_d),s));

        while !open_nodes.is_empty() {
            let current_node = open_nodes.peek().unwrap().1;
            if current_node == d {
                let mut path = Vec::new();
                let mut counter = &current_node;
                while let Some(mp) = came_from.get(counter) {
                    path.push(*mp);
                    counter = &mp;
                }
                return Some(path);
            }

            open_nodes.pop();
            let neigh = current_node.neighbors_at_distance(self.width(), self.height(),2,false); // may be different for different units

            for (n,_one) in neigh {
                let temp = g_score.get(&current_node).unwrap()+1; // may need to change this when adding hills
                if let Some(neighg) = g_score.get(&n) {
                    if temp > *neighg {
                        continue;
                    }
                }

                came_from.insert(n,current_node);
                g_score.insert(n,temp);
                if !visited_nodes.contains(&n) {
                    visited_nodes.insert(n);
                    open_nodes.push((Reverse(temp+n.distance_to(d)),n));
                }
            }
        }
        None
    }

    pub fn cmp_tile_yields_decreasing(&self, a: TilePosition, b: TilePosition) -> Ordering {
        let a_yields = self.tile(a).yields().total();
        let b_yields = self.tile(b).yields().total();
        b_yields.partial_cmp(&a_yields).unwrap()
    }
}
