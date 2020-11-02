use crate::common::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct UnitId(u16);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitIdGenerator {
    next: u16,
}

impl UnitIdGenerator {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn next(&mut self) -> UnitId {
        self.next += 1;
        UnitId(self.next)
    }
}

// Should this be split into Soldier and Civilian? :/
// Or a Unit trait with Soldier/Civilian impls.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    id: UnitId,
    owner: CivilizationId,
    unit_type: UnitType,
    pub(in crate::common) position: MapPosition,
    pub(in crate::common) remaining_movement: MapUnit,
}

impl Unit {
    pub fn new(id: UnitId, owner: CivilizationId, position: MapPosition, unit_type: UnitType) -> Self {
        let mut ret = Self {
            id,
            owner,
            unit_type,
            position,
            remaining_movement: 0,
        };
        ret.on_turn_start();
        ret
    }

    pub fn owner(&self) -> CivilizationId {
        self.owner
    }

    pub fn unit_type(&self) -> UnitType {
        self.unit_type
    }

    pub fn position(&self) -> MapPosition {
        self.position
    }

    pub fn id(&self) -> UnitId {
        self.id
    }

    pub(in crate::common) fn on_turn_start(&mut self) {
        self.remaining_movement = self.total_movement();
    }

    pub fn total_movement(&self) -> MapUnit {
        2
    }

    pub fn remaining_movement(&self) -> MapUnit {
        self.remaining_movement
    }

    // Returns if the unit has the ability to settle. Note that this does not mean the unit can
    // settle right now, eg. may be on invalid tile or not enough movement.
    pub fn has_settle_ability(&self) -> bool {
        self.unit_type == UnitType::Civilian
    }
}
