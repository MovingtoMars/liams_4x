use crate::common::*;

fn map_size(num_players: usize) -> (MapUnit, MapUnit) {
    let num_tiles_wanted = num_players * 250;

    let width = (num_tiles_wanted as f32).sqrt() * 1.2;
    let height = num_tiles_wanted as f32 / width;

    (width as MapUnit, height as MapUnit)
}

impl GameWorld {
    fn generate_river(&mut self, start_position: TilePosition) {
        let mut river_current = EdgePosition(start_position, CanonicalTileEdge::Top);

        let direction_fns = &[
            EdgePosition::top_left,
            EdgePosition::top_right,
            EdgePosition::bottom_left,
            EdgePosition::bottom_right,
        ];
        let direction_fn = direction_fns[rand::random::<usize>() % direction_fns.len()];

        while self.map.add_river(river_current) {
            river_current = direction_fn(river_current);
        }
    }

    fn random_tile_position(&self) -> TilePosition {
        TilePosition {
            x: rand::random::<MapUnit>().abs() % self.map.width(),
            y: rand::random::<MapUnit>().abs() % self.map.height(),
        }
    }

    pub fn generate(init_players: Vec<InitPlayer>) -> Self {
        let (width, height) = map_size(init_players.len());
        let num_tiles = width * height;
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

                let position = TilePosition { x, y };
                world.map.tile_mut(position).tile_type = tile_type;


            }
        }

        for i in 0..world.civilizations().count() {
            let civilization_id = world.civilizations().nth(i).unwrap().id();

            let x = world.map.width() as f32 / (world.civilizations().count() as f32 + 1.0) * (i as f32 + 1.0);
            let y = if i % 2 == 0 {
                world.map.height() as f32 / 3.0
            } else {
                world.map.height() as f32 / 3.0 * 2.0
            };
            let position = TilePosition { x: x as MapUnit, y: y as MapUnit };

            if !world.map.tile(position).resideable() {
                world.map.tile_mut(position).tile_type = TileType::Plains;
            }

            let id = world.next_unit_id();
            world.new_unit(id, &world.unit_template_manager().settler.clone(), civilization_id, position);
            let id = world.next_unit_id();
            world.new_unit(id, &world.unit_template_manager().warrior.clone(), civilization_id, position);
        }

        for _ in 0..(num_tiles / 50) {
            world.generate_river(world.random_tile_position());
        }

        world
    }
}
