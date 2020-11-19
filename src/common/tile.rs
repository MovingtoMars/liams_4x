use std::collections::{BTreeMap, BTreeSet};

use crate::common::*;

use TileType::*;
use ResourceType::*;
use Vegetation::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Plains,
    Mountain,
    Ocean,
    Desert,
}

impl TileType {
    pub fn supported_resources(self) -> Vec<(ResourceType, usize)> {
        match self {
            Plains => vec![(Sheep, 3), (Horses, 2), (Gold, 1), (Iron, 1), (Silver, 1), (Niter, 1), (Coal, 1), (Wheat, 3)],
            Mountain | Ocean | Desert => vec![],
        }
    }

    pub fn supported_vegetation(self) -> Vec<(Vegetation, usize)> {
        match self {
            Plains => vec![(Forest, 4), (Jungle, 1)],
            Mountain | Ocean | Desert => vec![],
        }
    }

    pub fn yields(self) -> Yields {
        match self {
            Plains => Yields::default().with_food(1.0),
            Mountain => Yields::default().with_science(1.0),
            Ocean => Yields::default().with_food(1.0),
            Desert => Yields::default(),
        }
    }
}

impl std::fmt::Display for TileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Plains => "Plains",
            Mountain => "Mountain",
            Ocean => "Ocean",
            Desert => "Desert",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub position: TilePosition,
    pub tile_type: TileType,

    pub units: BTreeMap<UnitType, UnitId>,
    pub city: Option<CityId>,
    pub territory_of: Option<CityId>,

    pub rivers: BTreeSet<TileEdge>,

    pub resource: Option<ResourceType>,
    pub vegetation: Option<Vegetation>,

    pub harvested: bool,
}

impl Tile {
    pub fn resideable(&self) -> bool {
        match self.tile_type {
            Plains => true,
            _ => false,
        }
    }

    pub fn unit_can_reside(&self, unit_type: &UnitType) -> bool {
        self.resideable() && !self.units.contains_key(unit_type)
    }

    pub fn yields(&self) -> Yields {
        if self.city.is_some() {
            return Yields::default().with_food(2.0).with_production(2.0);
        }
        let resource_yields = self.resource.map(|r| r.yields(self.harvested)).unwrap_or(Yields::default());
        let vegetation_yields = self.vegetation.map(|v| v.yields()).unwrap_or(Yields::default());
        let type_yields = self.tile_type.yields();

        resource_yields + vegetation_yields + type_yields
    }
}
