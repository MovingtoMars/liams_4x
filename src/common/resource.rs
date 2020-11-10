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
            Forest => Yields::default().with_production(1),
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
    pub fn yields(self) -> Yields {
        match self {
            Sheep => Yields::default().with_production(1),
            Horses => Yields::default().with_production(1),
            Gold => Yields::default().with_production(1),
            Iron => Yields::default().with_production(1),
            Silver => Yields::default().with_production(1),
            Niter => Yields::default().with_production(1),
            Coal => Yields::default().with_production(1),
            Wheat => Yields::default().with_food(1),
        }
    }
}

pub type Yield = i16;

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Yields {
    pub food: Yield,
    pub production: Yield,
    pub science: Yield,
}

impl std::ops::Add for Yields {
    type Output = Yields;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            food: self.food + rhs.food,
            production: self.production + rhs.production,
            science: self.science + rhs.science,
        }
    }
}

impl Yields {
    pub fn with_food(mut self, food: Yield) -> Self {
        self.food = food;
        self
    }

    pub fn with_production(mut self, production: Yield) -> Self {
        self.production = production;
        self
    }

    pub fn with_science(mut self, science: Yield) -> Self {
        self.science = science;
        self
    }

    pub fn total(self) -> Yield {
        self.food + self.production + self.science
    }
}
