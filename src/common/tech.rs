use crate::common::*;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use petgraph::Directed;
use petgraph::graph::{Graph, NodeIndex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tech {
    name: String,
    dependencies: Vec<TechId>,
    // TODO should we use IDs for these instead?
    // Also, should we switch to using string IDs? e.g. `granary`.
    buildings: Vec<BuildingType>,
    units: Vec<UnitTemplateId>,
    cost: YieldValue,
    // TODO should be in client code
    position: (f32, f32),
}

impl Tech {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dependencies(&self) -> &[TechId] {
        &self.dependencies
    }

    pub fn position(&self) -> (f32, f32) {
        self.position
    }

    pub fn cost(&self) -> YieldValue {
        self.cost
    }

    // TODO move to client code
    pub fn info(&self, unit_templates: &UnitTemplates) -> String {
        let mut ret: Vec<String> = Vec::new();

        ret.push(format!("Cost: {} Science", self.cost));
        ret.push("".into());

        if self.buildings.len() > 0 {
            ret.push("Buildings:".into());
            for building in &self.buildings {
                ret.push(format!("{}", building.name));
            }
            ret.push("".into());
        }

        if self.buildings.len() > 0 {
            ret.push("Units:".into());
            for unit_template_id in &self.units {
                let unit_template = unit_templates.get(*unit_template_id);
                ret.push(format!("{}", unit_template.name));
            }
            ret.push("".into());
        }

        ret.dedup();
        ret.join("\n").trim_end().to_owned()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TechId(u8);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechProgress {
    completed: BTreeSet<TechId>,
    researching: Option<TechId>,
    progress: YieldValue,
    unlocked_buildings: BTreeSet<BuildingTypeId>,
    unlocked_units: BTreeSet<UnitTemplateId>,
}

impl TechProgress {
    pub fn new(tech_tree: &TechTree) -> Self {
        let mut ret = Self {
            completed: tech_tree.initial_techs().clone(),
            researching: None,
            progress: 0.0.into(),
            unlocked_buildings: BTreeSet::new(),
            unlocked_units: BTreeSet::new(),
        };

        ret.update_unlocked_buildings(tech_tree);
        ret.update_unlocked_units(tech_tree);
        ret
    }

    pub fn has_completed(&self, tech_id: TechId) -> bool {
        self.completed.contains(&tech_id)
    }

    fn update_unlocked_buildings(&mut self, tech_tree: &TechTree) {
        self.unlocked_buildings = self.completed
            .iter()
            .flat_map(|tech_id| {
                tech_tree.get(*tech_id).buildings.iter().map(|b| b.id)
            })
            .collect();
    }

    fn update_unlocked_units(&mut self, tech_tree: &TechTree) {
        self.unlocked_units = self.completed
            .iter()
            .flat_map(|tech_id| {
                tech_tree.get(*tech_id).units.iter().map(|b| *b)
            })
            .collect();
    }

    pub(in crate::common) fn add_completed(&mut self, tech: TechId, tech_tree: &TechTree) {
        self.completed.insert(tech);
        self.update_unlocked_buildings(tech_tree);
        self.update_unlocked_units(tech_tree);
    }

    #[allow(dead_code)]
    pub fn completed(&self) -> impl Iterator<Item = &TechId> {
        self.completed.iter().map(|t| t)
    }

    pub fn researching(&self) -> Option<TechId> {
        self.researching
    }

    pub(in crate::common) fn set_researching(&mut self, researching: Option<TechId>) {
        self.researching = researching;
    }

    pub fn unlocked_buildings(&self) -> &BTreeSet<BuildingTypeId> {
        &self.unlocked_buildings
    }

    pub fn unlocked_units(&self) -> &BTreeSet<UnitTemplateId> {
        &self.unlocked_units
    }

    pub(in crate::common) fn on_turn_start(&mut self, science_yield: YieldValue) {
        self.progress += science_yield;
    }

    pub fn can_finish_research(&self, tech_tree: &TechTree) -> bool {
        if let Some(researching) = self.researching {
            tech_tree.get(researching).cost <= self.progress
        } else {
            false
        }
    }

    pub(in crate::common) fn finish_research(&mut self, tech_tree: &TechTree) {
        let finished = self.researching.take().unwrap();
        self.progress -= tech_tree.get(finished).cost;
        self.add_completed(finished, tech_tree);
    }

    pub fn can_research(&self, tech_id: TechId, tech_tree: &TechTree) -> bool {
        if self.has_completed(tech_id) {
            return false;
        }

        tech_tree.get(tech_id).dependencies().into_iter().all(|dep| self.has_completed(*dep))
    }

    pub fn can_research_any(&self, tech_tree: &TechTree) -> bool {
        self.completed.len() < tech_tree.all().len()
    }

    pub fn remaining_turns_for_current_research(&self, tech_tree: &TechTree, science: YieldValue) -> Option<usize> {
        self.researching.map(|current| (tech_tree.get(current).cost - self.progress).div_to_get_turn_count(science))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechTree {
    graph: Graph<TechId, (), Directed>,
    id_node_map: BTreeMap<TechId, NodeIndex>,
    id_tech_map: BTreeMap<TechId, Tech>,
    next_id: u8,
    initial_techs: BTreeSet<TechId>,
}

impl TechTree {
    fn new() -> Self {
        Self {
            graph: Graph::new(),
            id_node_map: BTreeMap::new(),
            id_tech_map: BTreeMap::new(),
            next_id: 0,
            initial_techs: BTreeSet::new(),
        }
    }

    fn next_id(&mut self) -> TechId {
        self.next_id += 1;
        TechId(self.next_id)
    }

    pub fn get(&self, tech_id: TechId) -> &Tech {
        self.id_tech_map.get(&tech_id).unwrap()
    }

    fn add(&mut self, tech: Tech) -> TechId {
        let id = self.next_id();
        let node_index = self.graph.add_node(id);
        self.id_node_map.insert(id, node_index);

        for dependency in &tech.dependencies {
            let dependency_node_index = self.id_node_map.get(dependency).unwrap();
            self.graph.add_edge(*dependency_node_index, node_index, ());
        }
        self.id_tech_map.insert(id, tech);

        id
    }

    #[allow(unused_variables)]
    pub fn generate(buildings: &BuildingTypes, units: &UnitTemplates) -> Self {
        let mut tree = Self::new();

        let root_id = tree.add(Tech {
            name: "Agriculture".into(),
            dependencies: vec![],
            position: (0.5, 0.0),
            cost: 0.0.into(),
            buildings: vec![
                buildings.get_by_name("Granary").clone(),
            ],
            units: vec![
                units.get_by_name("Settler").id,
                units.get_by_name("Warrior").id,
                units.get_by_name("Worker").id,
            ],
        });
        tree.initial_techs.insert(root_id);
        let a_id = tree.add(Tech {
            name: "A".into(),
            dependencies: vec![root_id],
            position: (0.4, 0.1),
            cost: 10.0.into(),
            buildings: vec![],
            units: vec![],
        });
        let b_id = tree.add(Tech {
            name: "B".into(),
            dependencies: vec![a_id],
            position: (0.4, 0.2),
            cost: 10.0.into(),
            buildings: vec![],
            units: vec![],
        });
        let c_id = tree.add(Tech {
            name: "C".into(),
            dependencies: vec![root_id, b_id],
            position: (0.6, 0.3),
            cost: 10.0.into(),
            buildings: vec![],
            units: vec![],
        });
        let d_id = tree.add(Tech {
            name: "D".into(),
            dependencies: vec![c_id],
            position: (0.6, 0.4),
            cost: 10.0.into(),
            buildings: vec![],
            units: vec![],
        });

        // TODO validate tree

        tree
    }

    pub fn all(&self) -> Vec<TechId> {
        self.id_tech_map.keys().map(|id| *id).collect()
    }

    pub fn initial_techs(&self) -> &BTreeSet<TechId> {
        &self.initial_techs
    }
}
