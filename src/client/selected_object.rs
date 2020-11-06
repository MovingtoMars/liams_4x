use imgui::ImString;

use crate::common::{UnitId, TilePosition, CityId};

// TODO rename to GuiState or something?
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SelectedObject {
    Tile(TilePosition),
    Unit(UnitId),
    City(CityId, ImString),
}
