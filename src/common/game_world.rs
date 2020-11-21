use std::collections::BTreeMap;

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

    pub fn production_cost(&self) -> YieldValue {
        match self {
            ProducingItem::Unit(unit) => unit.production_cost,
            ProducingItem::Building(building) => building.production_cost,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameWorld {
    pub map: GameMap,
    players: BTreeMap<PlayerId, Player>,
    units: BTreeMap<UnitId, Unit>,
    cities: BTreeMap<CityId, City>,

    // TODO possibly make Civilizations struct with helper methods.
    civilizations: BTreeMap<CivilizationId, Civilization>,

    turn: u16,

    unit_id_generator: UnitIdGenerator,
    city_name_generator: CityNameGenerator,
    city_id_generator: CityIdGenerator,
    civilization_id_generator: CivilizationIdGenerator,

    unit_templates: UnitTemplates,

    building_types: BuildingTypes,

    tech_tree: TechTree,
}

impl GameWorld {
    pub fn new(width: MapUnit, height: MapUnit, init_players: Vec<InitPlayer>) -> Self {
        let building_types = BuildingTypes::new();
        let unit_templates = UnitTemplates::new();

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
            tech_tree: TechTree::generate(&building_types, &unit_templates),
            unit_templates,
            building_types,
        };

        for init_player in init_players {
            game.new_civilization(init_player);
        }

        game
    }

    pub fn tech_tree(&self) -> &TechTree {
        &self.tech_tree
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

    pub fn unit_templates(&self) -> &UnitTemplates {
        &self.unit_templates
    }

    fn new_civilization(&mut self, init_player: InitPlayer) {
        let civilization_id = self.civilization_id_generator.next();
        let civilization = Civilization::new(civilization_id, init_player.name.clone(), &self.tech_tree);
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

        let tech_progress = self.civilizations.get(&owner).unwrap().tech_progress();
        let args = CityArgsMut { map: &mut self.map, building_types: &self.building_types, tech_progress, unit_templates: &self.unit_templates };
        let city = City::new(id, owner, position, name, args);

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
                        positions_to_try.extend(city.position.direct_neighbors(self.map.width(), self.map.height()));

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
                        let event = GameEventType::NewBuilding { building_type_id: building_type.id, city_id };
                        result.push(self.apply_event_move(event));
                        let event = GameEventType::SetProducing { city_id, producing: None };
                        result.push(self.apply_event_move(event));
                    }
                }
            }
        }

        for civilization_id in self.civilizations.keys().map(|id| *id).collect::<Vec<_>>() {
            let civilization = self.civilizations.get_mut(&civilization_id).unwrap();
            if civilization.tech_progress.can_finish_research(&self.tech_tree) {
                let event = GameEventType::FinishResearch { civilization_id };
                result.push(self.apply_event_move(event));
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
                let neighbor_map = unit.position().neighbors_at_distance(self.map.width(), self.map.height(), unit.remaining_movement(), true);
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
            GameActionType::SetResearch { tech_id } => {
                let civilization_id = self.player(actioner_id).unwrap().civilization_id();
                let tech_progress = &self.civilizations.get(&civilization_id).unwrap().tech_progress;
                if tech_progress.can_research(*tech_id, &self.tech_tree) {
                    let event = GameEventType::SetResearch { civilization_id, tech_id: *tech_id };
                    result.push(self.apply_event_move(event));
                }
            }
        }

        result
    }

    pub fn apply_event_move(&mut self, event: GameEventType) -> GameEventType {
        self.apply_event(&event);
        event
    }

    fn on_turn_start(&mut self) {
        for city in self.cities.values_mut() {
            let tech_progress = self.civilizations.get(&city.owner()).unwrap().tech_progress();
            let args = CityArgs { map: &self.map, building_types: &self.building_types, tech_progress, unit_templates: &self.unit_templates };
            city.on_turn_start(args);
        }

        for unit in self.units.values_mut() {
            unit.on_turn_start();
        }

        for civilization_id in self.civilizations.keys().map(|id| *id).collect::<Vec<_>>() {
            let science_yield = self.civilization_science_yield(civilization_id).value;
            self.civilizations.get_mut(&civilization_id).unwrap().on_turn_start(science_yield);
        }
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
                self.cities.get_mut(city_id).unwrap().producing = producing.clone().and_then(|x| Some((x, 0.0.into())));
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
                let tech_progress = self.civilizations.get(&city.owner()).unwrap().tech_progress();
                let args = CityArgs { map: &self.map, building_types: &self.building_types, tech_progress, unit_templates: &self.unit_templates };
                city.set_citizen_locked(*position, *locked, args);
            }
            GameEventType::IncreasePopulationFromFood { city_id } => {
                let city = self.cities.get_mut(city_id).unwrap();
                let tech_progress = self.civilizations.get(&city.owner()).unwrap().tech_progress();
                let args = CityArgs { map: &self.map, building_types: &self.building_types, tech_progress, unit_templates: &self.unit_templates };
                city.increase_population_from_food(args);
            }
            GameEventType::AddTerritoryToCity { city_id, position } => {
                let city = self.cities.get_mut(city_id).unwrap();
                let tech_progress = self.civilizations.get(&city.owner()).unwrap().tech_progress();
                let args = CityArgsMut { map: &mut self.map, building_types: &self.building_types, tech_progress, unit_templates: &self.unit_templates };
                city.grow_territory(*position, args);
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
            GameEventType::FinishResearch { civilization_id } => {
                self.civilizations.get_mut(civilization_id).unwrap().tech_progress.finish_research(&self.tech_tree);
            }
            GameEventType::SetResearch { civilization_id, tech_id } => {
                self.civilizations.get_mut(civilization_id).unwrap().tech_progress.set_researching(Some(*tech_id));
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
        let city = self.cities.get_mut(&city_id).unwrap();
        let civ = self.civilizations.get(&city.owner).unwrap();
        let args = CityArgs { map: &self.map, building_types: &self.building_types, tech_progress: civ.tech_progress(), unit_templates: &self.unit_templates };
        city.add_building(self.building_types.get(building_type_id).clone(), args);
    }

    pub fn civilization_science_yield(&self, civ_id: CivilizationId) -> Yield {
        let mut sum = Yield { yield_type: YieldType::Science, value: 0.0.into() };

        for city in self.cities().filter(|city| city.owner() == civ_id) {
            sum.value += city.yields().science;
        }

        sum
    }
}
