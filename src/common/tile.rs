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
pub struct Territory {
    pub city_id: CityId,
    pub city_effect_yields: Yields,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    // TODO make stuff private
    pub position: TilePosition,
    pub tile_type: TileType,

    pub units: BTreeMap<UnitType, UnitId>,
    pub city: Option<CityId>,
    pub territory: Option<Territory>,

    pub rivers: BTreeSet<TileEdge>,

    pub resource: Option<ResourceType>,
    pub vegetation: Option<Vegetation>,

    pub harvested: bool,
}

pub struct TileYieldContributors {
    pub type_yields: Option<Yields>,
    pub resource_yields: Option<Yields>,
    pub vegetation_yields: Option<Yields>,
    pub city_yields: Option<Yields>,
    pub city_effect_yields: Option<Yields>,
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

    pub fn yield_contributors(&self) -> TileYieldContributors {
        TileYieldContributors {
            resource_yields: self.resource.map(|r| r.yields(self.harvested)),
            vegetation_yields: self.vegetation.map(|v| v.yields()),
            type_yields: if self.city.is_none() { Some(self.tile_type.yields()) } else { None },
            city_yields: self.city.map(|_| Yields::default().with_food(2.0).with_production(2.0)),
            city_effect_yields: self.territory.as_ref().map(|t| t.city_effect_yields),
        }
    }

    pub fn yields(&self) -> Yields {
        let TileYieldContributors {
            resource_yields,
            vegetation_yields,
            type_yields,
            city_yields,
            city_effect_yields,
        } = self.yield_contributors();

        let all = &[
            resource_yields,
            vegetation_yields,
            type_yields,
            city_yields,
            city_effect_yields,
        ];

        let mut result = Yields::default();
        for yields in all {
            if let Some(yields) = yields {
                result += *yields;
            }
        }

        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TileMatcher {
    IsType(TileType),
    HasResource(ResourceType),
    HasVegetation(Vegetation),
    // Shouldn't have more than one layer of or/and to avoid confusion
    And(Vec<TileMatcher>),
    Or(Vec<TileMatcher>),
}

impl TileMatcher {
    pub fn matches(&self, tile: &Tile) -> bool {
        match self {
            TileMatcher::IsType(tile_type) => {
                tile.tile_type == *tile_type
            }
            TileMatcher::HasResource(resource_type) => {
                tile.resource == Some(*resource_type)
            }
            TileMatcher::HasVegetation(vegetation) => {
                tile.vegetation == Some(*vegetation)
            }
            TileMatcher::And(matchers) => {
                matchers.iter().all(|matcher| matcher.matches(tile))
            }
            TileMatcher::Or(matchers) => {
                matchers.iter().any(|matcher| matcher.matches(tile))
            }
        }
    }

    fn display_inner(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileMatcher::IsType(tile_type) => {
                write!(f, "{}", tile_type)
            }
            TileMatcher::HasResource(resource_type) => {
                write!(f, "{}", resource_type)
            }
            TileMatcher::HasVegetation(vegetation) => {
                write!(f, "{}", vegetation)
            }
            TileMatcher::And(matchers) => {
                let s = matchers.iter().map(|matcher| format!("{}", matcher)).collect::<Vec<_>>().join(" ");
                write!(f, "{}", s)
            }
            TileMatcher::Or(matchers) => {
                let s = matchers.iter().map(|matcher| format!("{}", matcher)).collect::<Vec<_>>().join(" and ");
                write!(f, "{}", s)
            }
        }
    }
}

// TODO move to client
impl std::fmt::Display for TileMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_inner(f)?;
        write!(f, " tiles")
    }
}
