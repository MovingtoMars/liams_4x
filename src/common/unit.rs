use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::common::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct UnitId(u16);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitIdGenerator {
    next: u16,
}

impl UnitIdGenerator {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn next(&mut self) -> UnitId {
        self.next += 1;
        UnitId(self.next)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitAbility {
    Settle,
    Harvest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitTemplate {
    pub unit_type: UnitType,
    pub name: String,
    pub movement: MapUnit,
    pub abilities: BTreeSet<UnitAbility>,
    pub production_cost: YieldValue,
    pub initial_charges: Option<usize>,
}

impl UnitTemplate {
    pub fn turn_cost(&self, production: YieldValue) -> usize {
        (self.production_cost / production).ceil_usize()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitTemplateManager {
    pub settler: UnitTemplate,
    pub worker: UnitTemplate,
    pub warrior: UnitTemplate,
}

impl UnitTemplateManager {
    pub fn new() -> Self {
        Self {
            settler: UnitTemplate {
                unit_type: UnitType::Civilian,
                name: "Settler".into(),
                movement: 2,
                abilities: vec![UnitAbility::Settle].into_iter().collect(),
                production_cost: 20.0.into(),
                initial_charges: None,
            },
            worker: UnitTemplate {
                unit_type: UnitType::Civilian,
                name: "Worker".into(),
                movement: 2,
                abilities: vec![UnitAbility::Harvest].into_iter().collect(),
                production_cost: 15.0.into(),
                initial_charges: Some(3),
            },
            warrior: UnitTemplate {
                unit_type: UnitType::Soldier,
                name: "Warrior".into(),
                movement: 2,
                abilities: vec![].into_iter().collect(),
                production_cost: 14.0.into(),
                initial_charges: None,
            },
        }
    }

    pub fn all(&self) -> Vec<&UnitTemplate> {
        vec![
            &self.settler,
            &self.worker,
            &self.warrior,
        ]
    }
}

// Should this be split into Soldier and Civilian? :/
// Or a Unit trait with Soldier/Civilian impls.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    id: UnitId,
    name: String,
    owner: CivilizationId,
    unit_type: UnitType,
    total_movement: MapUnit,
    abilities: BTreeSet<UnitAbility>,
    // (current, initial)
    charges: Option<(usize, usize)>,
    pub(in crate::common) sleeping: bool,
    pub(in crate::common) position: TilePosition,
    pub(in crate::common) remaining_movement: MapUnit,
}

impl Unit {
    pub fn new(template: &UnitTemplate, id: UnitId, owner: CivilizationId, position: TilePosition) -> Self {
        Self {
            id,
            owner,
            unit_type: template.unit_type,
            position,
            total_movement: template.movement,
            name: template.name.clone(),
            abilities: template.abilities.clone(),
            charges: template.initial_charges.map(|n| (n, n)),

            remaining_movement: 0,
            sleeping: false,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn owner(&self) -> CivilizationId {
        self.owner
    }

    pub fn unit_type(&self) -> UnitType {
        self.unit_type
    }

    pub fn position(&self) -> TilePosition {
        self.position
    }

    pub fn id(&self) -> UnitId {
        self.id
    }

    pub(in crate::common) fn on_turn_start(&mut self) {
        self.remaining_movement = self.total_movement();
    }

    pub fn total_movement(&self) -> MapUnit {
        self.total_movement
    }

    pub fn remaining_movement(&self) -> MapUnit {
        self.remaining_movement
    }

    pub fn sleeping(&self) -> bool {
        self.sleeping
    }

    pub fn has_ability(&self, ability: UnitAbility) -> bool {
        self.abilities.contains(&ability)
    }

    pub fn abilities(&self) -> impl Iterator<Item = &UnitAbility> {
        self.abilities.iter()
    }

    pub fn can_harvest(&self, cities: &BTreeMap<CityId, City>, map: &GameMap) -> bool {
        let tile = map.tile(self.position);
        let city = if let Some(city_id) = tile.territory_of {
            cities.get(&city_id).unwrap()
        } else {
            return false;
        };

        self.has_ability(UnitAbility::Harvest)
            && self.remaining_movement > 0
            && !tile.harvested
            && tile.resource.is_some()
            && city.owner() == self.owner()
            && self.charges.map(|(charges, _)| charges > 0).unwrap_or(true)
    }

    pub fn charges(&self) -> Option<(usize, usize)> {
        self.charges
    }

    pub(in crate::common) fn use_charge(&mut self) {
        if let Some((ref mut current, _)) = self.charges {
            if *current == 0 {
                panic!("called use_charge() on unit with charges = 0");
            } else {
                *current -= 1;
            }
        } else {
            panic!("called use_charge() on unit with charges = None");
        }
    }
}
