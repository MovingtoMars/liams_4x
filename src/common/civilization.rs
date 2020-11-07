use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CivilizationColor {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl CivilizationColor {
    pub fn percents(self) -> [f32; 3] {
        [self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0]
    }
}

const RUBY_RED: CivilizationColor = CivilizationColor { r: 163, g: 30, b: 100 };
const AMETHYST: CivilizationColor = CivilizationColor { r: 136, g: 102, b: 204 };
const MEDIUM_BLUE: CivilizationColor = CivilizationColor { r: 68, g: 6, b: 194 };
const SHEEN_GREEN: CivilizationColor = CivilizationColor { r: 140, g: 207, b: 8 };
const PINK: CivilizationColor = CivilizationColor { r: 252, g: 81, b: 147 };

const COLORS: &[CivilizationColor] = &[
    RUBY_RED,
    AMETHYST,
    MEDIUM_BLUE,
    SHEEN_GREEN,
    PINK,
];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CivilizationId(u8);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CivilizationIdGenerator {
    next_civilization_id: u8,
}

impl CivilizationIdGenerator {
    pub fn new() -> Self {
        Self { next_civilization_id: 0 }
    }

    pub fn next(&mut self) -> CivilizationId {
        let x = self.next_civilization_id;
        self.next_civilization_id += 1;
        CivilizationId(x)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Civilization {
    id: CivilizationId,
    player_name: String,
}

impl Civilization {
    pub fn new<S: Into<String>>(id: CivilizationId, player_name: S) -> Self {
        Self {
            id,
            player_name: player_name.into(),
        }
    }

    pub fn color(&self) -> CivilizationColor {
        COLORS[self.id.0 as usize % COLORS.len()]
    }

    pub fn id(&self) -> CivilizationId {
        self.id
    }

    pub fn player_name(&self) -> &String {
        &self.player_name
    }
}
