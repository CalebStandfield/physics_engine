//! Lookup from scenario id to a live scenario.
//!
//! This is the whole cost of adding a scenario: one `use`, one entry in
//! `SCENARIOS`. Nothing in the core or the bindings changes.

use physics_core::scenario::Scenario;

use crate::incline::InclineScenario;
use crate::spring::SpringScenario;

type Factory = fn() -> Box<dyn Scenario>;

/// Every scenario the engine ships, in the order a UI should list them.
static SCENARIOS: &[(&str, Factory)] = &[
    ("spring", || Box::new(SpringScenario::new())),
    ("incline", || Box::new(InclineScenario::new())),
];

/// Ids of every available scenario.
pub fn ids() -> Vec<&'static str> {
    SCENARIOS.iter().map(|(id, _)| *id).collect()
}

/// Build one by id. `None` for an unknown id, so callers can fall back.
pub fn create(id: &str) -> Option<Box<dyn Scenario>> {
    SCENARIOS
        .iter()
        .find(|(known, _)| *known == id)
        .map(|(_, make)| make())
}

/// The scenario to show when nobody picks one.
pub fn default_scenario() -> Box<dyn Scenario> {
    let (_, make) = SCENARIOS[0];
    make()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_id_builds_and_reports_itself() {
        for id in ids() {
            let s = create(id).unwrap_or_else(|| panic!("{id} did not build"));
            assert_eq!(s.id(), id);
            assert!(!s.name().is_empty());
            assert!(!s.schema().is_empty());
        }
        assert!(create("nope").is_none());
    }

    #[test]
    fn every_schema_default_round_trips() {
        for id in ids() {
            let mut s = create(id).unwrap();
            for spec in s.schema() {
                assert_eq!(s.get_param(spec.key), Some(spec.default), "{id}.{}", spec.key);
                assert!(spec.min <= spec.default && spec.default <= spec.max, "{id}.{}", spec.key);
                s.set_param(spec.key, spec.default).unwrap();
            }
        }
    }
}
