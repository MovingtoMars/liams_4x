use crate::common::*;
use voronoi::*;
use ncollide2d::shape::{ConvexPolygon,Shape};
use ncollide2d::math::{Isometry, Translation, Vector};
use ncollide2d::query::PointQuery;

use std::collections::HashMap;

use std::convert::TryInto;

fn choose_with_weights<T: Copy>(inputs: &[(T, usize)]) -> T {
    assert!(inputs.len() > 0);

    let total: usize = inputs.iter().map(|(_, weight)| weight).sum();
    let mut r = rand::random::<usize>() % total;

    for &(choice, weight) in inputs {
        if r < weight {
            return choice;
        }
        r -= weight;
    }

    unreachable!()
}

fn map_size(num_players: usize) -> (MapUnit, MapUnit) {
    let num_tiles_wanted = num_players * 250;

    let width = (num_tiles_wanted as f32).sqrt() * 1.2;
    let height = num_tiles_wanted as f32 / width;

    (width as MapUnit, height as MapUnit)
}


fn contains_point(poly:Vec<Point>,p:Point) {

}

impl GameWorld {
    fn generate_river(&mut self, start_position: TilePosition) {
        let mut river_current = CanonicalEdgePosition(start_position, CanonicalTileEdge::Top);

        let direction_fns = &[
            CanonicalEdgePosition::top_left,
            CanonicalEdgePosition::top_right,
            CanonicalEdgePosition::bottom_left,
            CanonicalEdgePosition::bottom_right,
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

        // biome generation
        let BIOME_AMT = 3;
        let mut vor_pts = Vec::new();//vec![Point::new(0.0, 1.0), Point::new(2.0, 3.0), Point::new(10.0, 12.0)];
        for i in 0..BIOME_AMT {
            vor_pts.push(voronoi::Point::new((rand::random::<f32>()*(width as f32)).into(),(rand::random::<f32>()*(height as f32)).into()));
        }
        let vor_diagram = voronoi(vor_pts, 800.0);
        let vor_polys = make_polygons(&vor_diagram);
        let mut polys = Vec::new();

        for i in vor_polys {
            let mut points = Vec::new();
            for j in i {
                points.push(ncollide2d::math::Point::new(j.x.0,j.y.0));
            }
            //let a = points.try_into().unwrap();
            polys.push(ConvexPolygon::try_from_points(&points).unwrap());
        }

        println!("ok");

        let mut biomes = HashMap::new();

        for x in 0..world.map.width() {
            for y in 0..world.map.height() {
                let p = ncollide2d::math::Point::new(x as f64,y as f64);
                let m = Isometry::new(Vector::new(0.0,0.0),0.0);
                let mut i = 0;
                for z in &polys {
                    if z.contains_point(&m,&p) {
                        biomes.insert((x,y),i);
                        break;
                    }
                    i+=1;
                }
            }
        }

        println!("{:?}",biomes);

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
                        if let Some(m) = biomes.get(&(x,y)) {//rand::random::<f32>() > 0.85 {
                            match (*m) {
                                0 => TileType::Mountain,
                                1 => TileType::Plains,
                                2 => TileType::Desert,
                            }
                        } else {
                            TileType::Plains
                        }
                    }
                };

                let position = TilePosition { x, y };
                world.map.tile_mut(position).tile_type = tile_type;

                let supported_resources = tile_type.supported_resources();
                if supported_resources.len() > 0 && rand::random::<f32>() > 0.7 {
                    let resource = choose_with_weights(&supported_resources);
                    world.map.tile_mut(position).resource = Some(resource);
                }

                let supported_vegetation = tile_type.supported_vegetation();
                if supported_vegetation.len() > 0 && rand::random::<f32>() > 0.7 {
                    let vegetation = choose_with_weights(&supported_vegetation);
                    world.map.tile_mut(position).vegetation = Some(vegetation);
                }
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
