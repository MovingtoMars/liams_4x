use serde::{Serialize, Deserialize};

pub type Yield = f32;

// TODO possibly split off YieldsMultiplier
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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

impl std::ops::AddAssign for Yields {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Mul for Yields {
    type Output = Yields;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            food: self.food * rhs.food,
            production: self.production * rhs.production,
            science: self.science * rhs.science,
        }
    }
}

impl std::ops::MulAssign for Yields {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Yields {
    pub fn identity() -> Self {
        Self {
            food: 1.0,
            production: 1.0,
            science: 1.0,
        }
    }

    pub fn zero() -> Self {
        Self {
            food: 0.0,
            production: 0.0,
            science: 0.0,
        }
    }

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

    pub fn iter(self) -> impl Iterator<Item = (Yield, &'static str)> {
        vec![
            (self.food, "Food"),
            (self.production, "Production"),
            (self.science, "Science"),
        ].into_iter()
    }

    pub fn iter_non_zero(self) -> impl Iterator<Item = (Yield, &'static str)> {
        self.iter().filter(|(y, _)| *y != 0.0)
    }

    pub fn iter_non_identity(self) -> impl Iterator<Item = (Yield, &'static str)> {
        self.iter().filter(|(y, _)| *y != 1.0)
    }
}
