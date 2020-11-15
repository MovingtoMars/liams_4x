use serde::{Serialize, Deserialize};

const CITY_NAMES: &[&str] = &[
    "Auckland",
    "Wellington",
    "Christchurch",
    "Palmerston North",
    "Taupo",
    "Tauranga",
    "Dunedin",
    "Gisborne",
    "New Plymouth",
    "Whangarei",
    "Invercargill",
    "Queenstown",
    "Nelson",
    "Napier",
    "Queenstown",
    "Porirua",
    "Rotorua",
    "Hastings",
    "Upper Hutt",
    "Lower Hutt",
    "Whanganui",
    "Levin",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CityNameGenerator {
    next_name_id: usize,
}

impl CityNameGenerator {
    pub fn new() -> Self {
        Self {
            next_name_id: 0,
        }
    }

    pub fn next(&mut self) -> String {
        let result = CITY_NAMES[self.next_name_id % CITY_NAMES.len()].to_owned();
        self.next_name_id += 1;
        result
    }
}
