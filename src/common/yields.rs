use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub struct YieldValue(f32);

impl std::fmt::Display for YieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (self.0 * 100.0).round() / 100.0)
    }
}

impl YieldValue {
    pub fn round(self) -> usize {
        self.0.round() as usize
    }

    pub fn div_to_get_turn_count(self, rhs: Self) -> usize {
        let turn_count = (self.0 / rhs.0).ceil();
        if turn_count >= 0.0 {
            turn_count as usize
        } else {
            0
        }
    }
}

impl<T: Into<f32>> From<T> for YieldValue {
    fn from(x: T) -> Self {
        YieldValue(x.into())
    }
}

impl std::ops::Add for YieldValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        YieldValue(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for YieldValue {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub for YieldValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        YieldValue(self.0 - rhs.0)
    }
}

impl std::ops::SubAssign for YieldValue {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl std::ops::Mul for YieldValue {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        YieldValue(self.0 * rhs.0)
    }
}

impl std::ops::MulAssign for YieldValue {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl std::ops::Div for YieldValue {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        YieldValue(self.0 / rhs.0)
    }
}

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

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
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

    pub fn with_food(mut self, food: f32) -> Self {
        self.food = food.into();
        self
    }

    pub fn with_production(mut self, production: f32) -> Self {
        self.production = production.into();
        self
    }

    pub fn with_science(mut self, science: f32) -> Self {
        self.science = science.into();
        self
    }

    pub fn total(self) -> YieldValue {
        self.food + self.production + self.science
    }
}
