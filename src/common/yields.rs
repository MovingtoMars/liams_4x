use serde::{Serialize, Deserialize};

pub type Yield = f32;

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
