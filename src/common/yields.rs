use serde::{Serialize, Deserialize};

pub type YieldValue = f32;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Yield {
    pub value: YieldValue,
    pub yield_type: YieldType,
}

impl std::fmt::Display for Yield {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "+{} {}", self.value, self.yield_type)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct YieldMultiplier {
    pub multiplier: YieldValue,
    pub yield_type: YieldType,
}

impl std::fmt::Display for YieldMultiplier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x{} {}", self.multiplier, self.yield_type)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum YieldType {
    Food,
    Production,
    Science,
}

impl std::fmt::Display for YieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match *self {
            YieldType::Food => "Food",
            YieldType::Production => "Production",
            YieldType::Science => "Science",
        })
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Yields {
    pub food: YieldValue,
    pub production: YieldValue,
    pub science: YieldValue,
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

impl std::ops::Add<Yield> for Yields {
    type Output = Yields;

    fn add(mut self, rhs: Yield) -> Self::Output {
        *self.get_mut(rhs.yield_type) += rhs.value;
        self
    }
}

impl std::ops::AddAssign for Yields {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::AddAssign<Yield> for Yields {
    fn add_assign(&mut self, rhs: Yield) {
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

impl std::ops::Mul<YieldMultiplier> for Yields {
    type Output = Yields;

    fn mul(mut self, rhs: YieldMultiplier) -> Self::Output {
        *self.get_mut(rhs.yield_type) *= rhs.multiplier;
        self
    }
}

impl std::ops::MulAssign for Yields {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl std::ops::MulAssign<YieldMultiplier> for Yields {
    fn mul_assign(&mut self, rhs: YieldMultiplier) {
        *self = *self * rhs;
    }
}

impl Yields {
    pub fn get_mut(&mut self, yield_type: YieldType) -> &mut YieldValue {
        match yield_type {
            YieldType::Food => &mut self.food,
            YieldType::Production => &mut self.production,
            YieldType::Science => &mut self.science,
        }
    }

    pub fn zero() -> Self {
        Self {
            food: 0.0,
            production: 0.0,
            science: 0.0,
        }
    }

    pub fn with_food(mut self, food: YieldValue) -> Self {
        self.food = food;
        self
    }

    pub fn with_production(mut self, production: YieldValue) -> Self {
        self.production = production;
        self
    }

    pub fn with_science(mut self, science: YieldValue) -> Self {
        self.science = science;
        self
    }

    pub fn total(self) -> YieldValue {
        self.food + self.production + self.science
    }
}
