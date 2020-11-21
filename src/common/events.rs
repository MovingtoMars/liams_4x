use crate::common::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEventType {
    NextTurn,
    MoveUnit { unit_id: UnitId, position: TilePosition, remaining_movement: MapUnit },
    DeleteUnit { unit_id: UnitId },
    FoundCity { position: TilePosition, owner: CivilizationId },
    RenameCity { city_id: CityId, name: String },
    SetPlayerReady { player_id: PlayerId, ready: bool },
    SetProducing { city_id: CityId, producing: Option<ProducingItemId> },
    NewUnit { template: UnitTemplate, owner: CivilizationId, position: TilePosition, unit_id: UnitId },
    NewBuilding { building_type_id: BuildingTypeId, city_id: CityId },
    Crash { message: String },
    SetSleeping { unit_id: UnitId, sleeping: bool },
    SetCitizenLocked { city_id: CityId, position: TilePosition, locked: bool },
    IncreasePopulationFromFood { city_id: CityId },
    AddTerritoryToCity { city_id: CityId, position: TilePosition },
    Harvest { position: TilePosition },
    DepleteMovement { unit_id: UnitId },
    UseCharge { unit_id: UnitId },
    FinishResearch { civilization_id: CivilizationId },
    SetResearch { civilization_id: CivilizationId, tech_id: TechId },
}
