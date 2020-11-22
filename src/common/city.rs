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
    AddTileYield { yield_: Yield, matcher: TileMatcher },
}

impl CityEffect {
    // Lower means applied earlier.
    // TODO could implement Sort trait.
    pub fn priority(&self) -> usize {
        match self {
            CityEffect::AddTileYield { .. } => 1,
            CityEffect::AddYield(..) => 2,
            CityEffect::MulYield(..) => 3,
        }
    }

    fn apply(&self, city: &mut City, map: &mut GameMap) {
        match self {
            CityEffect::AddYield(yield_) => {
                city.yields += *yield_;
            },
            CityEffect::MulYield(yield_mul) => {
                city.yields *= *yield_mul;
            },
            CityEffect::AddTileYield { yield_, matcher } => {
                for tile_position in city.territory_tiles() {
                    let tile = map.tile_mut(*tile_position);
                    if matcher.matches(tile) {
                        tile.territory.as_mut().unwrap().city_effect_yields += *yield_;
                    }
                }
            }
        }
    }
}

// TODO move to client
impl std::fmt::Display for CityEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CityEffect::AddYield(x) => write!(f, "{}\n", x)?,
            CityEffect::MulYield(x) => write!(f, "{}\n", x)?,
            CityEffect::AddTileYield { yield_, matcher } => write!(f, "{} for {}\n", yield_, matcher)?,
        }
        Ok(())
    }
}

pub struct CityArgs<'a> {
    pub map: &'a mut GameMap,
    pub building_types: &'a BuildingTypes,
    pub tech_progress: &'a TechProgress,
    pub unit_templates: &'a UnitTemplates,
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

    buildings: BTreeMap<BuildingTypeId, BuildingType>,
    // TODO should we use BuildingTypeId here?
    producible_buildings: Vec<BuildingType>,
    producible_units: Vec<UnitTemplateId>,

    effects: Vec<CityEffect>,
}

impl City {
    const TERRITORY_EXPAND_TURNS: isize = 6;

    pub fn new(id: CityId, owner: CivilizationId, position: TilePosition, name: String, args: CityArgs) -> Self {
        let mut territory = BTreeMap::new();
        for (pos, _) in position.neighbors_at_distance(args.map.width(), args.map.height(), 1, true) {
            let mut tile = args.map.tile_mut(pos);
            if tile.territory.is_none() {
                tile.territory = Some(Territory { city_id: id, city_effect_yields: Yields::default() });
                territory.insert(pos, None);
            }
        }
        args.map.tile_mut(position).city = Some(id);

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
            yields: Yields::default(),
            accumulated_food: 0.0.into(),
            required_food_for_population_increase: 0.0.into(),
            producible_buildings: vec![],
            effects: vec![],
            producible_units: vec![],
        };
        city.update(args);
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

    pub (in crate::common) fn on_turn_start(&mut self, args: CityArgs) {
        self.update(args);

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
        (self.required_food_for_population_increase - self.accumulated_food).div_to_get_turn_count(self.yields.food)
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
                .filter(|pos| map.tile(*pos).territory.is_none())
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

    pub fn grow_territory(&mut self, position: TilePosition, args: CityArgs) {
        self.territory.insert(position, None);
        args.map.tile_mut(position).territory = Some(Territory { city_id: self.id, city_effect_yields: Yields::default() });
        self.update(args);
        self.turns_until_territory_growth = Self::TERRITORY_EXPAND_TURNS;
    }

    pub(in crate::common) fn increase_population_from_food(&mut self, args: CityArgs) {
        self.population += 1;
        self.accumulated_food = 0.0.into();
        self.update(args);
    }

    pub fn producing(&self) -> &Option<(ProducingItem, YieldValue)> {
        &self.producing
    }

    pub fn territory(&self) -> &BTreeMap<TilePosition, Option<Citizen>> {
        &self.territory
    }

    pub fn territory_tiles(&self) -> impl Iterator<Item = &TilePosition> {
        self.territory.keys()
    }

    fn update_borders_from_territory(&mut self) {
        self.borders = TilePosition::borders(&self.territory_tiles().map(|p| *p).collect::<Vec<_>>());
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

    pub (in crate::common) fn set_citizen_locked(&mut self, position: TilePosition, locked: bool, args: CityArgs) {
        if locked {
            if self.population == self.locked_citizen_count() {
                let first_locked_citizen = self.territory.values_mut().find(|citizen| **citizen == Some(Citizen::Locked)).unwrap();
                *first_locked_citizen = None;
            }
            self.territory.insert(position, Some(Citizen::Locked));
        } else {
            self.territory.insert(position, None);
        }

        self.update(args);
    }

    pub fn borders(&self) -> &[EdgePosition] {
        &self.borders
    }

    fn update_yields(&mut self, map: &GameMap) {
        let pop_yields = Yields::default().with_science(self.population as f32);
        let tile_yields = self.territory
            .iter()
            .filter(|(pos, citizen)| **pos == self.position || citizen.is_some())
            .map(|(pos, _)| map.tile(*pos).yields())
            .fold(Yields::default(), |y1, y2| y1 + y2);

        self.yields = pop_yields + tile_yields;
    }

    fn update_producible_buildings(&mut self, building_types: &BuildingTypes, tech_progress: &TechProgress) {
        self.producible_buildings = tech_progress
            .unlocked_buildings()
            .iter()
            .filter(|building_type| !self.buildings.contains_key(&building_type))
            .map(|building_type| building_types.get(*building_type).clone())
            .collect();
    }

    fn update_producible_units(&mut self, tech_progress: &TechProgress) {
        self.producible_units = tech_progress
            .unlocked_units()
            .iter()
            .map(|unit_template_id| *unit_template_id)
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

    fn apply_effects(&mut self, map: &mut GameMap) {
        // Reset all the city effect yields for our territory tiles because the yields will be recalculated.
        for position in self.territory.keys() {
            if let Some(territory) = &mut map.tile_mut(*position).territory {
                territory.city_effect_yields = Yields::default();
            }
        }

        for effect in self.effects.clone() {
            effect.apply(self, map);
        }
    }

    pub (in crate::common) fn update(&mut self, args: CityArgs) {
        self.update_borders_from_territory();
        self.update_citizens(args.map);
        self.update_yields(args.map);
        self.update_producible_buildings(args.building_types, args.tech_progress);
        self.update_producible_units(args.tech_progress);
        self.update_effects();
        self.apply_effects(args.map);

        let req_food = 15.0 + 8.0 * (self.population as f32 - 1.0) + (self.population as f32 - 1.0).powf(1.5);
        self.required_food_for_population_increase = req_food.into();
    }

    pub fn yields(&self) -> Yields {
        self.yields
    }

    pub(in crate::common) fn add_building(&mut self, building: BuildingType, args: CityArgs) {
        self.buildings.insert(building.id, building);
        self.update(args);
    }

    pub fn buildings(&self) -> impl Iterator<Item = &BuildingType> {
        self.buildings.values()
    }

    pub fn producible_buildings(&self) -> impl Iterator<Item = &BuildingType> {
        self.producible_buildings.iter()
    }

    pub fn producible_units(&self) -> impl Iterator<Item = &UnitTemplateId> {
        self.producible_units.iter()
    }
}
