use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

use serde::{Serialize, Deserialize};

use crate::common::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProducingItemId {
    // TODO use UnitTemplateId
    Unit(UnitTemplate),
    Building(BuildingTypeId),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProducingItem {
    Unit(UnitTemplate),
    Building(BuildingType),
}

impl ProducingItem {
    pub fn name(&self) -> &str {
        match self {
            ProducingItem::Unit(unit) => &unit.name,
            ProducingItem::Building(building) => &building.name,
        }
    }

    pub fn production_cost(&self) -> Yield {
        match self {
            ProducingItem::Unit(unit) => unit.production_cost,
            ProducingItem::Building(building) => building.production_cost,
        }
    }
}

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
                    territory_of: None,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameWorld {
    pub map: GameMap,
    players: BTreeMap<PlayerId, Player>,
    units: BTreeMap<UnitId, Unit>,
    cities: BTreeMap<CityId, City>,
    civilizations: BTreeMap<CivilizationId, Civilization>,

    turn: u16,

    unit_id_generator: UnitIdGenerator,
    city_name_generator: CityNameGenerator,
    city_id_generator: CityIdGenerator,
    civilization_id_generator: CivilizationIdGenerator,

    unit_template_manager: UnitTemplateManager,

    building_types: BuildingTypes,
}

impl GameWorld {
    pub fn new(width: MapUnit, height: MapUnit, init_players: Vec<InitPlayer>) -> Self {
        let mut game = GameWorld {
            map: GameMap::new(width, height),
            players: BTreeMap::new(),
            units: BTreeMap::new(),
            cities: BTreeMap::new(),
            civilizations: BTreeMap::new(),
            turn: 0,
            unit_id_generator: UnitIdGenerator::new(),
            city_name_generator: CityNameGenerator::new(),
            city_id_generator: CityIdGenerator::new(),
            civilization_id_generator: CivilizationIdGenerator::new(),
            unit_template_manager: UnitTemplateManager::new(),
            building_types: BuildingTypes::new(),
        };

        for init_player in init_players {
            game.new_civilization(init_player);
        }

        game
    }

    pub fn start(&mut self) {
        if self.turn != 0 {
            panic!("Can only start game when turn is 0");
        }
        self.turn += 1;
        self.on_turn_start();
    }

    #[allow(dead_code)]
    pub fn building_types(&self) -> &BuildingTypes {
        &self.building_types
    }

    pub fn unit_template_manager(&self) -> &UnitTemplateManager {
        &self.unit_template_manager
    }

    fn new_civilization(&mut self, init_player: InitPlayer) {
        let civilization_id = self.civilization_id_generator.next();
        let civilization = Civilization::new(civilization_id, init_player.name.clone());
        self.civilizations.insert(civilization_id, civilization);
        let player = Player::new(init_player.id, init_player.name.clone(), civilization_id);
        self.players.insert(init_player.id, player);
    }

    pub fn players(&self) -> impl Iterator<Item = &Player> {
        self.players.iter().map(|(_, v)| v)
    }

    // TODO Option seems unnecessary
    pub fn player(&self, id: PlayerId) -> Option<&Player> {
        self.players.get(&id)
    }

    pub fn civilizations(&self) -> impl Iterator<Item = &Civilization> {
        self.civilizations.iter().map(|(_, v)| v)
    }

    // TODO Option seems unnecessary
    pub fn civilization(&self, id: CivilizationId) -> Option<&Civilization> {
        self.civilizations.get(&id)
    }

    pub fn turn(&self) -> u16 {
        self.turn
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

    pub fn new_city(&mut self, owner: CivilizationId, position: TilePosition) -> &mut City {
        assert!(self.map.tile(position).city.is_none());

        let id = self.city_id_generator.next();
        let name = self.city_name_generator.next();

        let city = City::new(id, owner, position, name, &mut self.map, &self.building_types);

        self.cities.insert(id, city);
        self.cities.get_mut(&id).unwrap()
    }

    pub(in crate::common) fn next_unit_id(&mut self) -> UnitId {
        self.unit_id_generator.next()
    }

    pub fn new_unit(&mut self, id: UnitId, template: &UnitTemplate, owner: CivilizationId, position: TilePosition) -> &mut Unit {
        assert!(!self.map.tile(position).units.contains_key(&template.unit_type));

        let mut unit = Unit::new(template, id, owner, position);
        unit.on_turn_start();

        self.map.tile_mut(position).units.insert(template.unit_type, id);

        self.units.insert(id, unit);
        self.units.get_mut(&id).unwrap()
    }

    fn next_turn(&mut self) -> Vec<GameEventType> {
        let mut result = vec![];
        let event = GameEventType::NextTurn;
        self.apply_event(&event);
        result.push(event);

        let city_keys = self.cities.keys().map(|k| *k).collect::<Vec<_>>();
        for city_id in city_keys {
            {
                let city = self.cities.get_mut(&city_id).unwrap();
                if city.can_increase_population_from_food() {
                    let event = GameEventType::IncreasePopulationFromFood { city_id };
                    self.apply_event(&event);
                    result.push(event);
                }
            }

            {
                let city = self.cities.get_mut(&city_id).unwrap();
                if city.ready_to_grow_territory() {
                    if let Some(position) = city.next_tile_to_expand_to(&self.map) {
                        let event = GameEventType::AddTerritoryToCity { city_id, position };
                        self.apply_event(&event);
                        result.push(event);
                    }
                }
            }

            let finished_production = {
                let city = self.cities.get(&city_id).unwrap();

                if let Some((producing, ref spent)) = &city.producing {
                    if *spent >= producing.production_cost() {
                        Some(city.producing.clone().unwrap().0)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(finished_production) = finished_production {
                match finished_production {
                    ProducingItem::Unit(template) => {
                        let city = self.cities.get(&city_id).unwrap();

                        let mut positions_to_try = vec![city.position];
                        positions_to_try.extend(city.position.direct_neighbors(self.map.width, self.map.height));

                        let position = positions_to_try.into_iter()
                            .find(|pos| self.map.tile(*pos).unit_can_reside(&template.unit_type));

                        if let Some(position) = position {
                            let owner = city.owner;
                            let unit_id = self.next_unit_id();
                            let event = GameEventType::NewUnit { unit_id, template, owner, position };
                            result.push(self.apply_event_move(event));
                            let event = GameEventType::SetProducing { city_id, producing: None };
                            result.push(self.apply_event_move(event));
                        } else {
                            let message = "Couldn't find an empty space beside city.";
                            result.push(GameEventType::Crash { message: message.into() });
                        }
                    }
                    ProducingItem::Building(building_type) => {
                        let event = GameEventType::NewBuilding { building_type_id: building_type.id, city_id: city_id };
                        result.push(self.apply_event_move(event));
                        let event = GameEventType::SetProducing { city_id, producing: None };
                        result.push(self.apply_event_move(event));
                    }
                }
            }
        }
        result
    }

    pub fn process_action(&mut self, action_type: &GameActionType, actioner_id: PlayerId) -> Vec<GameEventType> {
        let mut result = Vec::new();

        match action_type {
            GameActionType::MoveUnit { unit_id, position } => {
                let unit = if let Some(unit) = self.unit(*unit_id) { unit } else { return vec![] };
                if self.player(actioner_id).unwrap().civilization_id() != unit.owner() { return vec![] };

                let target_tile_moveable = self.map.tile(*position).unit_can_reside(&unit.unit_type());
                let neighbor_map = unit.position().neighbors_at_distance(self.map.width, self.map.height, unit.remaining_movement(), true);
                let distance = neighbor_map.get(position);
                let target_tile_in_range = distance.is_some();

                if target_tile_moveable && target_tile_in_range {
                    let event = GameEventType::MoveUnit {
                        unit_id: *unit_id,
                        position: *position,
                        remaining_movement: unit.remaining_movement() - distance.unwrap(),
                    };
                    result.push(self.apply_event_move(event));
                }
            }
            GameActionType::FoundCity { unit_id } => {
                let unit = if let Some(unit) = self.unit(*unit_id) { unit } else { return vec![] };
                if self.player(actioner_id).unwrap().civilization_id() != unit.owner() { return vec![] };
                let city_exists_on_tile = self.map.tile(unit.position()).city.is_some();

                if unit.has_ability(UnitAbility::Settle) && unit.remaining_movement() >= 1 && !city_exists_on_tile {
                    let events = vec![
                        GameEventType::DeleteUnit { unit_id: *unit_id },
                        GameEventType::FoundCity { position: unit.position(), owner: unit.owner() },
                    ];
                    self.apply_events(&events);
                    result.extend(events);
                }
            }
            GameActionType::RenameCity { city_id, name } => {
                if let Some(city) = self.city(*city_id) {
                    if self.player(actioner_id).unwrap().civilization_id() != city.owner() { return vec![] };

                    let event = GameEventType::RenameCity { city_id: *city_id, name: name.clone() };
                    result.push(self.apply_event_move(event));
                }
            }
            GameActionType::SetReady(ready) => {
                let event = GameEventType::SetPlayerReady{ player_id: actioner_id, ready: *ready };
                result.push(self.apply_event_move(event));

                if self.players().all(|player| player.ready()) {
                    result.extend(self.next_turn());
                }
            }
            GameActionType::SetProducing { city_id, producing } => {
                let city = if let Some(city) = self.city(*city_id) { city } else { return vec![] };
                if self.player(actioner_id).unwrap().civilization_id() != city.owner() { return vec![] };

                let event = GameEventType::SetProducing { city_id: *city_id, producing: producing.clone() };
                result.push(self.apply_event_move(event));
            }
            GameActionType::SetSleeping { unit_id, sleeping } => {
                let unit = if let Some(unit) = self.unit(*unit_id) { unit } else { return vec![] };
                if self.player(actioner_id).unwrap().civilization_id() != unit.owner() { return vec![] };

                let event = GameEventType::SetSleeping { unit_id: *unit_id, sleeping: *sleeping };
                result.push(self.apply_event_move(event));
            }
            GameActionType::SetCitizenLocked { city_id, position, locked } => {
                let city = if let Some(city) = self.city(*city_id) { city } else { return vec![] };
                if self.player(actioner_id).unwrap().civilization_id() != city.owner() { return vec![] };
                if !city.territory().contains_key(position) { return vec![] };

                let event = GameEventType::SetCitizenLocked { city_id: *city_id, position: *position, locked: *locked };
                result.push(self.apply_event_move(event));
            }
            GameActionType::Harvest { unit_id } => {
                let unit = if let Some(unit) = self.unit(*unit_id) { unit } else { return vec![] };
                if self.player(actioner_id).unwrap().civilization_id() != unit.owner() { return vec![] };
                if unit.can_harvest(&self.cities, &self.map) {
                    let event = GameEventType::Harvest { position: unit.position() };
                    result.push(self.apply_event_move(event));

                    let event = GameEventType::DepleteMovement { unit_id: *unit_id };
                    result.push(self.apply_event_move(event));

                    let event = GameEventType::UseCharge { unit_id: *unit_id };
                    result.push(self.apply_event_move(event));

                    let unit = self.unit(*unit_id).unwrap();
                    if unit.charges().unwrap().0 == 0 {
                        let event = GameEventType::DeleteUnit { unit_id: *unit_id };
                        result.push(self.apply_event_move(event));
                    }
                }
            }
        }

        result
    }

    pub fn apply_event_move(&mut self, event: GameEventType) -> GameEventType {
        self.apply_event(&event);
        event
    }

    pub fn apply_event(&mut self, event_type: &GameEventType) {
        match event_type {
            GameEventType::NextTurn => {
                self.turn += 1;
                self.on_turn_start();

                for player in self.players.values_mut() {
                    player.ready = false;
                }
            }
            GameEventType::MoveUnit { unit_id, position, remaining_movement } => {
                self.set_unit_position(*unit_id, *position);
                self.units.get_mut(unit_id).unwrap().remaining_movement = *remaining_movement;
            }
            GameEventType::DeleteUnit { unit_id } => {
                self.delete_unit(*unit_id);
            }
            GameEventType::FoundCity { position, owner } => {
                self.new_city(*owner, *position);
            }
            GameEventType::RenameCity { city_id, name } => {
                self.cities.get_mut(city_id).unwrap().name = name.clone();
            }
            GameEventType::SetPlayerReady { player_id, ready } => {
                self.players.get_mut(player_id).unwrap().ready = *ready;
            }
            GameEventType::SetProducing { city_id, producing } => {
                let producing = producing.as_ref().map(|producing| match producing {
                    ProducingItemId::Unit(template) => {
                        ProducingItem::Unit(template.clone())
                    }
                    ProducingItemId::Building(id) => {
                        ProducingItem::Building(self.building_types.get(*id).clone())
                    }
                });
                self.cities.get_mut(city_id).unwrap().producing = producing.clone().and_then(|x| Some((x, 0.0)));
            }
            GameEventType::NewUnit { template, owner, position, unit_id } => {
                self.new_unit(*unit_id, &template, *owner, *position);
            }
            GameEventType::NewBuilding { city_id, building_type_id } => {
                self.new_building(*city_id, *building_type_id);
            }
            GameEventType::Crash { .. } => {
                // We expect the client to handle this.
            }
            GameEventType::SetSleeping { unit_id, sleeping } => {
                self.units.get_mut(unit_id).unwrap().sleeping = *sleeping;
            }
            GameEventType::SetCitizenLocked { city_id, position, locked } => {
                let city = self.cities.get_mut(city_id).unwrap();
                city.set_citizen_locked(*position, *locked, &self.map, &self.building_types);
            }
            GameEventType::IncreasePopulationFromFood { city_id } => {
                let city = self.cities.get_mut(city_id).unwrap();
                city.increase_population_from_food(&self.map, &self.building_types);
            }
            GameEventType::AddTerritoryToCity { city_id, position } => {
                let city = self.cities.get_mut(city_id).unwrap();
                city.grow_territory(*position, &mut self.map, &self.building_types);
            }
            GameEventType::Harvest { position } => {
                self.map.tile_mut(*position).harvested = true;
            }
            GameEventType::DepleteMovement { unit_id } => {
                self.units.get_mut(unit_id).unwrap().remaining_movement = 0;
            }
            GameEventType::UseCharge { unit_id } => {
                self.units.get_mut(unit_id).unwrap().use_charge();
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
        self.map.tile_mut(position).units.remove(&unit.unit_type());
        self.units.remove(&unit_id);
    }

    fn on_turn_start(&mut self) {
        for city in self.cities.values_mut() {
            city.on_turn_start(&self.map, &self.building_types);
        }

        for unit in self.units.values_mut() {
            unit.on_turn_start();
        }
    }

    fn set_unit_position(&mut self, unit_id: UnitId, new_position: TilePosition) {
        let mut unit = self.units.get_mut(&unit_id).unwrap();

        let old_position = unit.position;
        unit.position = new_position;

        assert!(self.map.tile(old_position).units.get(&unit.unit_type()) == Some(&unit_id));
        self.map.tile_mut(old_position).units.remove(&unit.unit_type());

        assert!(self.map.tile(new_position).units.get(&unit.unit_type()) == None);
        self.map.tile_mut(new_position).units.insert(unit.unit_type(), unit_id);
    }

    fn new_building(&mut self, city_id: CityId, building_type_id: BuildingTypeId) {
        self.cities
            .get_mut(&city_id)
            .unwrap()
            .add_building(self.building_types.get(building_type_id).clone(), &self.map, &self.building_types);
    }
}
