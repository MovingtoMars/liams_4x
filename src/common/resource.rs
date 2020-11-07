use crate::common::*;

use ResourceType::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceType {
    Sheep,
    Horses,
}

impl ResourceType {
    pub fn yields(self) -> Yields {
        match self {
            Sheep => Yields::default().with_food(1).with_production(1),
            Horses => Yields::default().with_production(2),
        }
    }
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Yields {
    pub food: u8,
    pub production: u8,
    pub science: u8,
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
    pub fn with_food(mut self, food: u8) -> Self {
        self.food = food;
        self
    }

    pub fn with_production(mut self, production: u8) -> Self {
        self.production = production;
        self
    }

    pub fn with_science(mut self, science: u8) -> Self {
        self.science = science;
        self
    }
}
