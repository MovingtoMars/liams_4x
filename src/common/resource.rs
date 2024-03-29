use crate::common::*;

use ResourceType::*;
use Vegetation::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Vegetation {
    Forest,
    Jungle,
}

impl std::fmt::Display for Vegetation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Forest => "Forest",
            Jungle => "Jungle",
        })
    }
}

impl Vegetation {
    pub fn yields(self) -> Yields {
        match self {
            Forest => Yields::default().with_production(1.0),
            Jungle => Yields::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceType {
    Sheep,
    Horses,
    Gold,
    Iron,
    Silver,
    Niter,
    Coal,
    Wheat,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Sheep => "Sheep",
            Horses => "Horses",
            Gold => "Gold",
            Iron => "Iron",
            Silver => "Silver",
            Niter => "Niter",
            Coal => "Coal",
            Wheat => "Wheat",
        })
    }
}

impl ResourceType {
    pub fn yields(self, harvested: bool) -> Yields {
        if !harvested {
            match self {
                Sheep => Yields::default().with_production(1.0),
                Horses => Yields::default().with_production(2.0),
                Gold => Yields::default().with_production(1.0),
                Iron => Yields::default().with_production(1.0),
                Silver => Yields::default().with_production(1.0),
                Niter => Yields::default().with_production(1.0),
                Coal => Yields::default().with_production(1.0),
                Wheat => Yields::default().with_food(1.0),
            }
        } else {
            match self {
                Sheep => Yields::default().with_production(2.0),
                Horses => Yields::default().with_production(3.0),
                Gold => Yields::default().with_production(2.0),
                Iron => Yields::default().with_production(2.0),
                Silver => Yields::default().with_production(2.0),
                Niter => Yields::default().with_production(2.0),
                Coal => Yields::default().with_production(2.0),
                Wheat => Yields::default().with_food(2.0),
            }
        }
    }
}
