use serde::{Serialize, Deserialize};

use crate::common::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitPlayer {
    pub id: PlayerId,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    id: PlayerId,
    name: String,
    civilization_id: CivilizationId,
}

impl Player {
    pub fn new(id: PlayerId, name: String, civilization_id: CivilizationId) -> Self {
        Self {
            id,
            name,
            civilization_id,
        }
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn civilization_id(&self) -> CivilizationId {
        self.civilization_id
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(u8);

pub struct PlayerIdGenerator {
    next: u8,
}

impl PlayerIdGenerator {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn next(&mut self) -> PlayerId {
        self.next += 1;
        PlayerId(self.next)
    }
}
