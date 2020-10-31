use crate::common::{UnitId, MapPosition, CityId};

#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub enum ObjectType {
    Tile(MapPosition),
    Unit(UnitId),
    City(CityId),
}
