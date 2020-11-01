use imgui::ImString;

use crate::common::{UnitId, MapPosition, CityId};

// TODO rename to GuiState or something?
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SelectedObject {
    Tile(MapPosition),
    Unit(UnitId),
    City(CityId, ImString),
}
