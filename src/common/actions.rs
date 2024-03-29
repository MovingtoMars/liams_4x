use crate::common::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameActionType {
    MoveUnit { unit_id: UnitId, position: TilePosition },
    FoundCity { unit_id: UnitId },
    RenameCity { city_id: CityId, name: String },
    SetReady(bool),
    SetProducing { city_id: CityId, producing: Option<ProducingItemId> },
    SetSleeping { unit_id: UnitId, sleeping: bool },
    SetCitizenLocked { city_id: CityId, position: TilePosition, locked: bool },
    Harvest { unit_id: UnitId },
    SetResearch { tech_id: TechId },
}
