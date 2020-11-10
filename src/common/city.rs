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
pub struct City {
    pub (in crate::common) id: CityId,
    pub (in crate::common) owner: CivilizationId,
    pub (in crate::common) position: TilePosition,
    pub (in crate::common) name: String,
    pub (in crate::common) production: i16,

    // unit being produced and the amount of production put into it
    pub (in crate::common) producing: Option<(UnitTemplate, i16)>,

    pub (in crate::common) population: i16,
    // TODO make workable_territory
    pub (in crate::common) territory: BTreeMap<TilePosition, Option<Citizen>>,
    // Generated from territory and cached for perf
    pub (in crate::common) borders: Vec<EdgePosition>,

    yields: Yields,
}

impl City {
    pub fn new(id: CityId, owner: CivilizationId, position: TilePosition, name: String, world: &GameWorld) -> Self {
        let mut territory = BTreeMap::new();
        for (pos, _) in position.neighbors_at_distance(world.map.width(), world.map.height(), 1, true) {
            territory.insert(pos, None);
        }

        let mut city = City {
            position,
            owner,
            name,
            id,
            production: 5,
            population: 1,
            producing: None,
            territory,
            borders: vec![],
            yields: Yields::default(),
        };
        city.update(&world.map);
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

    pub (in crate::common) fn on_turn_start(&mut self) {
        if let Some((_, ref mut spent)) = &mut self.producing {
            *spent += self.production;
        }
    }

    pub fn production(&self) -> i16 {
        self.production
    }

    pub fn producing(&self) -> &Option<(UnitTemplate, i16)> {
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

        unworked_tiles.sort_by_key(|pos| -map.tile(*pos).yields().total());

        println!("{} {}", self.unemployed_citizen_count(), unworked_tiles.len());

        for i in 0..self.unemployed_citizen_count() {
            if let Some(unworked_pos) = unworked_tiles.get(i as usize) {
                self.territory.insert(*unworked_pos, Some(Citizen::Normal));
            }
        }
    }

    pub (in crate::common) fn set_citizen_locked(&mut self, position: TilePosition, locked: bool) {
        if locked {
            if self.population == self.locked_citizen_count() {
                let first_locked_citizen = self.territory.values_mut().find(|citizen| **citizen == Some(Citizen::Locked)).unwrap();
                *first_locked_citizen = None;
            }
            self.territory.insert(position, Some(Citizen::Locked));
        } else {
            self.territory.insert(position, None);
        }
    }

    pub fn borders(&self) -> &[EdgePosition] {
        &self.borders
    }

    fn update_yields(&mut self, map: &GameMap) {
        self.yields = self.territory
            .iter()
            .filter(|(pos, citizen)| **pos == self.position || citizen.is_some())
            .map(|(pos, _)| map.tile(*pos).yields())
            .fold(Yields::default(), |y1, y2| y1 + y2);
    }

    pub (in crate::common) fn update(&mut self, map: &GameMap) {
        self.update_borders_from_territory();
        self.update_citizens(&map);
        self.update_yields(&map);
    }

    pub fn yields(&self) -> Yields {
        self.yields
    }
}
