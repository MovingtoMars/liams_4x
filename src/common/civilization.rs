use serde::{Serialize, Deserialize};

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
        self.next_civilization_id += 1;
        CivilizationId(self.next_civilization_id)
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

    pub fn id(&self) -> CivilizationId {
        self.id
    }

    pub fn player_name(&self) -> &String {
        &self.player_name
    }
}
