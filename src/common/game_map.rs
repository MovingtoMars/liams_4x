use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::common::*;

pub type MapUnit = i16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitId(u16);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameActionType {
    MoveUnit { unit_id: UnitId, position: MapPosition },
    FoundCity { unit_id: UnitId }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEventType {
    NextTurn,
    MoveUnit { unit_id: UnitId, position: MapPosition, remaining_movement: MapUnit },
    DeleteUnit { unit_id: UnitId },
    FoundCity { position: MapPosition }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Plains,
    Mountain,
    Ocean,
}

impl std::fmt::Display for TileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            TileType::Plains => "Plains",
            TileType::Mountain => "Mountain",
            TileType::Ocean => "Ocean",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub position: MapPosition,
    pub tile_type: TileType,

    pub units: HashMap<UnitType, UnitId>,
    pub city: Option<CityId>,
}

impl Tile {
    pub fn units_can_reside(&self) -> bool {
        match self.tile_type {
            TileType::Plains => true,
            _ => false,
        }
    }
}

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
                    position: MapPosition { x, y },
                    tile_type: TileType::Plains,
                    units: HashMap::new(),
                    city: None,
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

    pub fn tile(&self, position: MapPosition) -> &Tile {
        &self.tiles[position.x as usize][position.y as usize]
    }

    pub fn tile_mut(&mut self, position: MapPosition) -> &mut Tile {
        &mut self.tiles[position.x as usize][position.y as usize]
    }

    pub fn tiles(&self) -> impl Iterator<Item = &Tile> {
        (0..self.width)
            .into_iter()
            .flat_map(move |x| {
                (0..self.height).into_iter().map(move |y| self.tile(MapPosition { x, y }))
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    Civilian,
    Soldier,
}

impl std::fmt::Display for UnitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            UnitType::Civilian => "Civilian",
            UnitType::Soldier => "Soldier",
        })
    }
}

// Should this be split into Soldier and Civilian? :/
// Or a Unit trait with Soldier/Civilian impls.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    unit_type: UnitType,
    position: MapPosition,
    id: UnitId,
    remaining_movement: MapUnit,
}

impl Unit {
    pub fn unit_type(&self) -> UnitType {
        self.unit_type
    }

    pub fn position(&self) -> MapPosition {
        self.position
    }

    pub fn id(&self) -> UnitId {
        self.id
    }

    fn on_turn_start(&mut self) {
        self.remaining_movement = self.total_movement();
    }

    pub fn total_movement(&self) -> MapUnit {
        2
    }

    pub fn remaining_movement(&self) -> MapUnit {
        self.remaining_movement
    }

    // Returns if the unit has the ability to settle. Note that this does not mean the unit can
    // settle right now, eg. may be on invalid tile or not enough movement.
    pub fn has_settle_ability(&self) -> bool {
        self.unit_type == UnitType::Civilian
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CityId(u16);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct City {
    position: MapPosition,
    name: String,
    id: CityId,
}

impl City {
    pub fn position(&self) -> MapPosition {
        self.position
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn id(&self) -> CityId {
        self.id
    }

    fn on_turn_start(&mut self) {

    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameWorld {
    pub map: GameMap,
    units: HashMap<UnitId, Unit>,
    cities: HashMap<CityId, City>,

    next_unit_id: u16,
    next_city_id: u16,

    turn: u16,

    city_name_generator: CityNameGenerator,
}

impl GameWorld {
    pub fn new(width: MapUnit, height: MapUnit) -> Self {
        GameWorld {
            map: GameMap::new(width, height),
            units: HashMap::new(),
            cities: HashMap::new(),
            next_unit_id: 0,
            next_city_id: 0,
            turn: 1,
            city_name_generator: CityNameGenerator::new(),
        }
    }

    pub fn turn(&self) -> u16 {
        self.turn
    }

    pub fn next_turn(&mut self) -> Vec<GameEventType> {
        self.apply_event(&GameEventType::NextTurn);
        vec![GameEventType::NextTurn]
    }

    fn next_unit_id(&mut self) -> UnitId {
        self.next_unit_id += 1;
        UnitId(self.next_unit_id)
    }

    fn next_city_id(&mut self) -> CityId {
        self.next_city_id += 1;
        CityId(self.next_city_id)
    }

    pub fn units(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().map(|(_, v)| v)
    }

    pub fn unit(&self, unit_id: UnitId) -> Option<&Unit> {
        self.units.get(&unit_id)
    }

    pub fn cities(&self) -> impl Iterator<Item = &City> {
        self.cities.iter().map(|(_, v)| v)
    }

    pub fn city(&self, city_id: CityId) -> Option<&City> {
        self.cities.get(&city_id)
    }

    pub fn new_city(&mut self, position: MapPosition) -> &mut City {
        assert!(self.map.tile(position).city.is_none());

        let id = self.next_city_id();
        let mut city = City {
            position,
            name: self.city_name_generator.next(),
            id,
        };

        city.on_turn_start();

        self.map.tile_mut(position).city = Some(id);
        self.cities.insert(id, city);
        self.cities.get_mut(&id).unwrap()
    }

    pub fn new_civilian(&mut self, position: MapPosition) -> &mut Unit {
        assert!(!self.map.tile(position).units.contains_key(&UnitType::Civilian));

        let id = self.next_unit_id();
        let mut unit = Unit {
            unit_type: UnitType::Civilian,
            id,
            position,
            remaining_movement: 0,
        };

        unit.on_turn_start();

        self.map.tile_mut(position).units.insert(UnitType::Civilian, id);

        self.units.insert(id, unit);
        self.units.get_mut(&id).unwrap()
    }

    pub fn new_soldier(&mut self, position: MapPosition) -> &mut Unit {
        assert!(!self.map.tile(position).units.contains_key(&UnitType::Soldier));

        let id = self.next_unit_id();
        let mut unit = Unit {
            unit_type: UnitType::Soldier,
            id,
            position,
            remaining_movement: 0,
        };

        unit.on_turn_start();

        self.map.tile_mut(position).units.insert(UnitType::Soldier, id);

        self.units.insert(id, unit);
        self.units.get_mut(&id).unwrap()
    }

    pub fn process_action(&mut self, action_type: &GameActionType) -> Vec<GameEventType> {
        let mut result = Vec::new();

        match action_type {
            GameActionType::MoveUnit { unit_id, position } => {
                let unit = if let Some(unit) = self.unit(*unit_id) { unit } else { return Vec::new() };

                let target_tile_unoccupied = !self.map.tile(*position).units.contains_key(&unit.unit_type);
                let target_tile_moveable = self.map.tile(*position).units_can_reside();
                let neighbor_map = unit.position().neighbors_at_distance(self.map.width, self.map.height, unit.remaining_movement(), true);
                let distance = neighbor_map.get(position);
                let target_tile_in_range = distance.is_some();

                if target_tile_unoccupied && target_tile_moveable && target_tile_in_range {
                    let event = GameEventType::MoveUnit {
                        unit_id: *unit_id,
                        position: *position,
                        remaining_movement: unit.remaining_movement() - distance.unwrap(),
                    };
                    self.apply_event(&event);
                    result.push(event);
                }
            }
            GameActionType::FoundCity { unit_id } => {
                let unit = if let Some(unit) = self.unit(*unit_id) { unit } else { return Vec::new() };
                let city_exists_on_tile = self.map.tile(unit.position()).city.is_some();
                if unit.has_settle_ability() && unit.remaining_movement() >= 1 && !city_exists_on_tile {
                    let events = vec![
                        GameEventType::DeleteUnit { unit_id: *unit_id },
                        GameEventType::FoundCity { position: unit.position() },
                    ];
                    self.apply_events(&events);
                    return events;
                }
            }
        }

        result
    }

    pub fn apply_event(&mut self, event_type: &GameEventType) {
        match event_type {
            GameEventType::NextTurn => {
                self.turn += 1;
                self.on_turn_start();
            }
            GameEventType::MoveUnit { unit_id, position, remaining_movement } => {
                self.set_unit_position(*unit_id, *position);
                self.units.get_mut(unit_id).unwrap().remaining_movement = *remaining_movement;
            }
            GameEventType::DeleteUnit { unit_id } => {
                self.delete_unit(*unit_id);
            }
            GameEventType::FoundCity { position } => {
                self.new_city(*position);
            }
        }
    }

    pub fn apply_events(&mut self, events: &[GameEventType]) {
        for event in events {
            self.apply_event(event);
        }
    }

    fn delete_unit(&mut self, unit_id: UnitId) {
        let unit = self.units.get_mut(&unit_id).unwrap();
        let position = unit.position();
        self.map.tile_mut(position).units.remove(&unit.unit_type);
        self.units.remove(&unit_id);
    }

    fn on_turn_start(&mut self) {
        for city in self.cities.values_mut() {
            city.on_turn_start();
        }

        for unit in self.units.values_mut() {
            unit.on_turn_start();
        }
    }

    fn set_unit_position(&mut self, unit_id: UnitId, new_position: MapPosition) {
        let mut unit = self.units.get_mut(&unit_id).unwrap();

        let old_position = unit.position;
        unit.position = new_position;

        assert!(self.map.tile(old_position).units.get(&unit.unit_type) == Some(&unit_id));
        self.map.tile_mut(old_position).units.remove(&unit.unit_type);

        assert!(self.map.tile(new_position).units.get(&unit.unit_type) == None);
        self.map.tile_mut(new_position).units.insert(unit.unit_type, unit_id);
    }
}

pub fn generate_game_world(width: MapUnit, height: MapUnit) -> GameWorld {
    let mut game = GameWorld::new(width, height);

    for x in 0..width {
        for y in 0..height {
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

            let position = MapPosition { x, y };

            game.map.tile_mut(position).tile_type = tile_type;

            if let (2, 2) = (x, y) {
                game.new_civilian(position);
            }

            if let (3, 3) = (x, y) {
                game.new_soldier(position);
            }

            if let (2, 3) = (x, y) {
                game.new_civilian(position);
                game.new_soldier(position);
            }

            if let (4, 4) = (x, y) {
                game.new_city(position);
            }

            if let (7, 6) = (x, y) {
                game.new_city(position);
            }
        }
    }

    game
}
