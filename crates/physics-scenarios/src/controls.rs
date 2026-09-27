//! One list of every control the scenarios can offer.
//!
//! A scenario does not invent a slider. It picks keys out of this table and
//! binds them to its own fields, so a control shared by two scenarios has the
//! same range, step and unit in both, and a new scenario reusing `mass` gets the
//! same slider everyone else has. Per-scenario wording and starting values are
//! overrides on the way out (`with_label`, `with_default`), never a second
//! range.

use physics_core::params::{ParamSpec, Tier};

/// Shorthand so the table below reads as a table. One argument per `ParamSpec`
/// field is the whole point here, so the arity lint does not apply.
#[allow(clippy::too_many_arguments)]
const fn ctl(
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    min: f64,
    max: f64,
    default: f64,
    step: f64,
    tier: Tier,
) -> ParamSpec {
    ParamSpec {
        key,
        label,
        unit,
        min,
        max,
        default,
        step,
        tier,
    }
}

/// Every control, in no particular order: the order a user sees is the order a
/// scenario lists them in.
///
/// Basic tier is the short list a simple panel shows: what the body is, what the
/// scenery is, and what starts the motion. Everything else, including gravity
/// and the initial-condition knobs, is advanced.
static CONTROLS: &[ParamSpec] = &[
    // Shared by every scenario.
    ctl("mass", "Mass", "kg", 0.01, 100.0, 1.0, 0.01, Tier::Basic),
    ctl(
        "gravity",
        "Gravity",
        "m/s^2",
        0.1,
        30.0,
        physics_core::G,
        0.01,
        Tier::Advanced,
    ),
    ctl(
        "initial_velocity",
        "Initial velocity",
        "m/s",
        -10.0,
        10.0,
        0.0,
        0.05,
        Tier::Advanced,
    ),
    // Springs.
    ctl(
        "stiffness",
        "Spring constant",
        "N/m",
        0.1,
        500.0,
        20.0,
        0.1,
        Tier::Basic,
    ),
    ctl(
        "initial_displacement",
        "Initial pull past equilibrium",
        "m",
        -0.5,
        0.5,
        0.05,
        0.005,
        Tier::Basic,
    ),
    ctl(
        "natural_length",
        "Natural length",
        "m",
        0.02,
        2.0,
        0.3,
        0.01,
        Tier::Advanced,
    ),
    ctl("damping", "Damping", "N s/m", 0.0, 20.0, 0.0, 0.01, Tier::Advanced),
    // Ramps.
    ctl("angle_deg", "Incline angle", "deg", 0.0, 89.0, 25.0, 0.5, Tier::Basic),
    ctl(
        "mu_static",
        "Static friction coefficient",
        "",
        0.0,
        2.0,
        0.35,
        0.01,
        Tier::Basic,
    ),
    ctl(
        "mu_kinetic",
        "Kinetic friction coefficient",
        "",
        0.0,
        2.0,
        0.25,
        0.01,
        Tier::Basic,
    ),
    ctl("length", "Ramp length", "m", 0.1, 20.0, 2.0, 0.05, Tier::Advanced),
    ctl(
        "initial_position",
        "Start distance from top",
        "m",
        0.0,
        20.0,
        0.0,
        0.05,
        Tier::Advanced,
    ),
    ctl(
        "applied_force",
        "Applied force along slope",
        "N",
        -200.0,
        200.0,
        0.0,
        0.5,
        Tier::Advanced,
    ),
];

/// Look up one control by key.
///
/// Panics on an unknown key. The table and the scenarios are compiled together,
/// so a miss is a typo in the source, not bad input at runtime.
pub fn control(key: &str) -> ParamSpec {
    match CONTROLS.iter().find(|c| c.key == key) {
        Some(spec) => *spec,
        None => panic!("no control named '{key}' in the catalog"),
    }
}

/// The whole table, for tests and for anything that wants to list what exists.
pub fn all() -> &'static [ParamSpec] {
    CONTROLS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique() {
        for (i, c) in CONTROLS.iter().enumerate() {
            assert!(
                !CONTROLS[..i].iter().any(|other| other.key == c.key),
                "duplicate control '{}'",
                c.key
            );
        }
    }

    #[test]
    fn every_range_holds_its_default() {
        for c in CONTROLS {
            assert!(c.min < c.max, "{} has an empty range", c.key);
            assert!(c.step > 0.0, "{} has a non-positive step", c.key);
            assert!(
                c.min <= c.default && c.default <= c.max,
                "{} default is outside its range",
                c.key
            );
        }
    }

    #[test]
    fn overrides_leave_the_range_alone() {
        let base = control("mass");
        let tuned = control("mass").with_default(0.25).with_label("Hanging mass");
        assert_eq!(tuned.default, 0.25);
        assert_eq!(tuned.label, "Hanging mass");
        assert_eq!((tuned.min, tuned.max, tuned.step), (base.min, base.max, base.step));
    }
}
