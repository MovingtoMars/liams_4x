use crate::common::*;

impl GameWorld {
    pub fn generate(width: MapUnit, height: MapUnit, init_players: Vec<InitPlayer>) -> Self {
        let mut world = Self::new(width, height, init_players);

        for x in 0..world.map.width() {
            for y in 0..world.map.height() {
                let tile_type = match (x, y) {
                    (0, 1) |
                    (1, 1) |
                    (_, 0) => TileType::Ocean,
                    (2, 2) |
                    (2, 3) |
                    (3, 3) |
                    (4, 4) |
                    (7, 6) => TileType::Plains,
                    _ => {
                        if rand::random::<f32>() > 0.85 {
                            TileType::Mountain
                        } else {
                            TileType::Plains
                        }
                    }
                };

                let civ1 = world.civilizations().next().unwrap().id();
                let position = MapPosition { x, y };

                world.map.tile_mut(position).tile_type = tile_type;

                if let (2, 2) = (x, y) {
                    world.new_unit(civ1, position, UnitType::Civilian);
                }

                if let (3, 3) = (x, y) {
                    world.new_unit(civ1, position, UnitType::Soldier);
                }

                if let (2, 3) = (x, y) {
                    world.new_unit(civ1, position, UnitType::Civilian);
                    world.new_unit(civ1, position, UnitType::Soldier);
                }

                if let (4, 4) = (x, y) {
                    world.new_city(civ1, position);
                }

                if let (7, 6) = (x, y) {
                    world.new_city(civ1, position);
                }
            }
        }

        world
    }
}
