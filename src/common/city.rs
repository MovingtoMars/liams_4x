use std::collections::BTreeMap;

use crate::common::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Citizen {
    Normal,
    Locked,
}

// TODO could create Id<City>, Id<Unit> etc
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct CityId(u16);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CityIdGenerator {
    next: u16,
}

impl CityIdGenerator {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn next(&mut self) -> CityId {
        self.next += 1;
        CityId(self.next)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CityEffect {
    AddYield(Yield),
    MulYield(YieldMultiplier),
}

impl CityEffect {
    // Lower means applied earlier.
    // TODO could implement Sort trait.
    pub fn priority(&self) -> usize {
        match self {
            CityEffect::AddYield(..) => 1,
            CityEffect::MulYield(..) => 2,
        }
    }

    fn apply(&self, city: &mut City) {
        match self {
            CityEffect::AddYield(yield_) => {
                city.yields += *yield_;
            },
            CityEffect::MulYield(yield_mul) => {
                city.yields *= *yield_mul;
            },
        }
    }
}

impl std::fmt::Display for CityEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CityEffect::AddYield(x) => write!(f, "{}\n", x)?,
            CityEffect::MulYield(x) => write!(f, "{}\n", x)?,
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct City {
    pub (in crate::common) id: CityId,
    pub (in crate::common) owner: CivilizationId,
    pub (in crate::common) position: TilePosition,
    pub (in crate::common) name: String,

    // unit being produced and the amount of production put into it
    pub (in crate::common) producing: Option<(ProducingItem, YieldValue)>,

    pub (in crate::common) population: i16,
    // TODO make workable_territory
    pub (in crate::common) territory: BTreeMap<TilePosition, Option<Citizen>>,
    // Generated from territory and cached for perf
    pub (in crate::common) borders: Vec<EdgePosition>,

    turns_until_territory_growth: isize,

    yields: Yields,

    accumulated_food: YieldValue,
    required_food_for_population_increase: YieldValue,

    // TODO should we use BuildingTypeId here?
    buildings: BTreeMap<BuildingTypeId, BuildingType>,
    producible_buildings: Vec<BuildingType>,

    effects: Vec<CityEffect>,
}

impl City {
    const TERRITORY_EXPAND_TURNS: isize = 6;

    pub fn new(id: CityId, owner: CivilizationId, position: TilePosition, name: String, map: &mut GameMap, building_types: &BuildingTypes) -> Self {
        let mut territory = BTreeMap::new();
        for (pos, _) in position.neighbors_at_distance(map.width(), map.height(), 1, true) {
            let mut tile = map.tile_mut(pos);
            if tile.territory_of.is_none() {
                tile.territory_of = Some(id);
                territory.insert(pos, None);
            }
        }
        map.tile_mut(position).city = Some(id);

        let mut city = City {
            position,
            owner,
            name,
            id,
            population: 1,
            producing: None,
            territory,
            turns_until_territory_growth: Self::TERRITORY_EXPAND_TURNS,
            buildings: BTreeMap::new(),

            // Calculated in the update() call below
            borders: vec![],
            yields: Yields::zero(),
            accumulated_food: 0.0,
            required_food_for_population_increase: 0.0,
            producible_buildings: vec![],
            effects: vec![],
        };
        city.update(map, building_types);
        city
    }

    pub fn id(&self) -> CityId {
        self.id
    }

    pub fn owner(&self) -> CivilizationId {
        self.owner
    }

    pub fn position(&self) -> TilePosition {
        self.position
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub (in crate::common) fn on_turn_start(&mut self, map: &GameMap, building_types: &BuildingTypes) {
        self.update(map, building_types);

        if let Some((_, ref mut spent)) = &mut self.producing {
            *spent += self.yields.production;
        }

        self.accumulated_food += self.yields.food;

        if self.turns_until_territory_growth > 0 {
            self.turns_until_territory_growth -= 1;
        }
    }

    pub fn accumulated_food(&self) -> YieldValue {
        self.accumulated_food
    }

    pub fn required_food_for_population_increase(&self) -> YieldValue {
        self.required_food_for_population_increase
    }

    pub fn turns_until_population_increase(&self) -> usize {
        ((self.required_food_for_population_increase - self.accumulated_food) as f32 / self.yields.food as f32).ceil() as usize
    }

    pub fn can_increase_population_from_food(&self) -> bool {
        self.accumulated_food >= self.required_food_for_population_increase
    }

    pub fn ready_to_grow_territory(&self) -> bool {
        self.turns_until_territory_growth <= 0
    }

    pub fn next_tile_to_expand_to(&self, map: &GameMap) -> Option<TilePosition> {
        for distance in 2..=3 {
            let mut tiles_at_distance: Vec<_> = self.position.neighbors_at_distance(map.width(), map.height(), distance, false)
                .keys()
                .map(|pos| *pos)
                .filter(|pos| map.tile(*pos).territory_of.is_none())
                .collect();

            tiles_at_distance.sort_by(|a, b| map.cmp_tile_yields_decreasing(*a, *b));

            if let Some(tile) = tiles_at_distance.first() {
                return Some(*tile);
            }
        }
        None
    }

    pub fn turns_until_territory_growth(&self) -> isize {
        self.turns_until_territory_growth
    }

    pub fn grow_territory(&mut self, position: TilePosition, map: &mut GameMap, building_types: &BuildingTypes) {
        self.territory.insert(position, None);
        map.tile_mut(position).territory_of = Some(self.id);
        self.update(map, building_types);
        self.turns_until_territory_growth = Self::TERRITORY_EXPAND_TURNS;
    }

    pub(in crate::common) fn increase_population_from_food(&mut self, map: &GameMap, building_types: &BuildingTypes) {
        self.population += 1;
        self.accumulated_food = 0.0;
        self.update(map, building_types);
    }

    pub fn producing(&self) -> &Option<(ProducingItem, YieldValue)> {
        &self.producing
    }

    pub fn territory(&self) -> &BTreeMap<TilePosition, Option<Citizen>> {
        &self.territory
    }

    pub fn territory_tiles(&self) -> Vec<TilePosition> {
        self.territory.iter().map(|(tile, _)| *tile).collect()
    }

    fn update_borders_from_territory(&mut self) {
        self.borders = TilePosition::borders(&self.territory_tiles());
    }

    pub fn population(&self) -> i16 {
        self.population
    }

    pub fn employed_citizen_count(&self) -> i16 {
        self.territory.values().filter(|citizen| citizen.is_some()).count() as i16
    }

    pub fn locked_citizen_count(&self) -> i16 {
        self.territory.values().filter(|citizen| **citizen == Some(Citizen::Locked)).count() as i16
    }

    pub fn unemployed_citizen_count(&self) -> i16 {
        self.population - self.employed_citizen_count()
    }

    fn update_citizens(&mut self, map: &GameMap) {
        for citizen in self.territory.values_mut() {
            if let Some(Citizen::Normal) = citizen {
                *citizen = None;
            }
        }

        let mut unworked_tiles: Vec<TilePosition> = self.territory
            .iter()
            .filter(|(pos, _)| **pos != self.position)
            .filter(|(_, citizen)| citizen.is_none())
            .map(|(pos, _)| *pos)
            .collect();

        unworked_tiles.sort_by(|a, b| map.cmp_tile_yields_decreasing(*a, *b));

        for i in 0..self.unemployed_citizen_count() {
            if let Some(unworked_pos) = unworked_tiles.get(i as usize) {
                self.territory.insert(*unworked_pos, Some(Citizen::Normal));
            }
        }
    }

    pub (in crate::common) fn set_citizen_locked(&mut self, position: TilePosition, locked: bool, map: &GameMap, building_types: &BuildingTypes) {
        if locked {
            if self.population == self.locked_citizen_count() {
                let first_locked_citizen = self.territory.values_mut().find(|citizen| **citizen == Some(Citizen::Locked)).unwrap();
                *first_locked_citizen = None;
            }
            self.territory.insert(position, Some(Citizen::Locked));
        } else {
            self.territory.insert(position, None);
        }

        self.update(map, building_types);
    }

    pub fn borders(&self) -> &[EdgePosition] {
        &self.borders
    }

    fn update_yields(&mut self, map: &GameMap) {
        let pop_yields = Yields::zero().with_science(self.population as f32);
        let tile_yields = self.territory
            .iter()
            .filter(|(pos, citizen)| **pos == self.position || citizen.is_some())
            .map(|(pos, _)| map.tile(*pos).yields())
            .fold(Yields::zero(), |y1, y2| y1 + y2);

        self.yields = pop_yields + tile_yields;
    }

    fn update_producible_buildings(&mut self, building_types: &BuildingTypes) {
        self.producible_buildings = building_types
            .all()
            .filter(|building_type| !self.buildings.contains_key(&building_type.id))
            .map(|building_type| building_type.clone())
            .collect();
    }

    fn update_effects(&mut self) {
        self.effects = self.buildings
            .values()
            .flat_map(|building| building.effects.iter())
            .map(|effect| effect.clone())
            .collect();

        self.effects.sort_by_key(|effect| effect.priority());
    }

    fn apply_effects(&mut self) {
        for effect in self.effects.clone() {
            effect.apply(self);
        }
    }

    // TODO maybe create CityUpdateArgs struct
    pub (in crate::common) fn update(&mut self, map: &GameMap, building_types: &BuildingTypes) {
        self.update_borders_from_territory();
        self.update_citizens(map);
        self.update_yields(map);
        self.update_producible_buildings(building_types);
        self.update_effects();
        self.apply_effects();

        self.required_food_for_population_increase = 15.0 + 8.0 * (self.population as YieldValue - 1.0) + (self.population as YieldValue - 1.0).powf(1.5);
    }

    pub fn yields(&self) -> Yields {
        self.yields
    }

    pub(in crate::common) fn add_building(&mut self, building: BuildingType, map: &GameMap, building_types: &BuildingTypes) {
        self.buildings.insert(building.id, building);
        self.update(map, building_types);
    }

    pub fn buildings(&self) -> impl Iterator<Item = &BuildingType> {
        self.buildings.values()
    }

    pub fn producible_buildings(&self) -> impl Iterator<Item = &BuildingType> {
        self.producible_buildings.iter()
    }
}
