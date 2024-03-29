use std::collections::BTreeMap;

use crate::common::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct BuildingTypeId(u16);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingType {
    pub id: BuildingTypeId,
    pub name: String,
    pub effects: Vec<CityEffect>,
    pub production_cost: YieldValue,
}

impl BuildingType {
    pub fn turn_cost(&self, production: YieldValue) -> usize {
        self.production_cost.div_to_get_turn_count(production)
    }

    pub fn effect_info(&self) -> String {
        // Could use a str buffer
        let mut info = "".to_owned();
        for effect in &self.effects {
            info += &format!("{}", effect);
        }
        info
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingTypeIdGenerator {
    next: u16,
}

impl BuildingTypeIdGenerator {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn next(&mut self) -> BuildingTypeId {
        self.next += 1;
        BuildingTypeId(self.next)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingTypes {
    building_types: BTreeMap<BuildingTypeId, BuildingType>,
    generator: BuildingTypeIdGenerator,
}

impl BuildingTypes {
    pub fn new() -> Self {
        let mut s = Self {
            building_types: BTreeMap::new(),
            generator: BuildingTypeIdGenerator::new(),
        };

        let building_types = vec![
            BuildingType {
                id: s.generator.next(),
                name: "Granary".into(),
                production_cost: 25.0.into(),
                effects: vec![
                    CityEffect::MulYield(YieldMultiplier { multiplier: 1.2.into(), yield_type: YieldType::Food }),
                ],
            },
            BuildingType {
                id: s.generator.next(),
                name: "Stable".into(),
                production_cost: 25.0.into(),
                effects: vec![
                    CityEffect::AddTileYield {
                        yield_: Yield { value: 1.0.into(), yield_type: YieldType::Production },
                        matcher: TileMatcher::HasResource(ResourceType::Horses),
                    },
                ],
            }
        ];

        for x in building_types.into_iter() {
            s.add(x);
        }

        s
    }

    fn add(&mut self, building_type: BuildingType) {
        self.building_types.insert(building_type.id, building_type);
    }

    pub fn get(&self, id: BuildingTypeId) -> &BuildingType {
        self.building_types.get(&id).unwrap()
    }

    pub fn get_by_name(&self, name: &str) -> &BuildingType {
        self.building_types.values().find(|bt| bt.name == name).unwrap()
    }
}
