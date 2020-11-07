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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnitAbility {
    Settle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitTemplate {
    pub unit_type: UnitType,
    pub name: String,
    pub movement: MapUnit,
    pub abilities: Vec<UnitAbility>,
    pub production_cost: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitTemplateManager {
    pub settler: UnitTemplate,
    pub warrior: UnitTemplate,
}

impl UnitTemplateManager {
    pub fn new() -> Self {
        Self {
            settler: UnitTemplate {
                unit_type: UnitType::Civilian,
                name: "Settler".into(),
                movement: 2,
                abilities: vec![UnitAbility::Settle],
                production_cost: 30,
            },
            warrior: UnitTemplate {
                unit_type: UnitType::Soldier,
                name: "Warrior".into(),
                movement: 2,
                abilities: vec![],
                production_cost: 21,
            },
        }
    }

    pub fn all(&self) -> Vec<&UnitTemplate> {
        vec![
            &self.settler,
            &self.warrior,
        ]
    }
}

// Should this be split into Soldier and Civilian? :/
// Or a Unit trait with Soldier/Civilian impls.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    id: UnitId,
    name: String,
    owner: CivilizationId,
    unit_type: UnitType,
    total_movement: MapUnit,
    pub(in crate::common) sleeping: bool,
    pub(in crate::common) position: TilePosition,
    pub(in crate::common) remaining_movement: MapUnit,
}

impl Unit {
    pub fn new(template: &UnitTemplate, id: UnitId, owner: CivilizationId, position: TilePosition) -> Self {
        Self {
            id,
            owner,
            unit_type: template.unit_type,
            position,
            total_movement: template.movement,
            name: template.name.clone(),

            remaining_movement: 0,
            sleeping: false,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn owner(&self) -> CivilizationId {
        self.owner
    }

    pub fn unit_type(&self) -> UnitType {
        self.unit_type
    }

    pub fn position(&self) -> TilePosition {
        self.position
    }

    pub fn id(&self) -> UnitId {
        self.id
    }

    pub(in crate::common) fn on_turn_start(&mut self) {
        self.remaining_movement = self.total_movement();
    }

    pub fn total_movement(&self) -> MapUnit {
        self.total_movement
    }

    pub fn remaining_movement(&self) -> MapUnit {
        self.remaining_movement
    }

    pub fn sleeping(&self) -> bool {
        self.sleeping
    }

    // Returns if the unit has the ability to settle. Note that this does not mean the unit can
    // settle right now, eg. may be on invalid tile or not enough movement.
    pub fn has_settle_ability(&self) -> bool {
        // TODO get from UnitTemplate
        self.unit_type == UnitType::Civilian
    }
}
