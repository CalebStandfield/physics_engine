//! Self-describing scenario parameters.
//!
//! A scenario declares a table of parameters once. Everything else (the
//! bindings, and later the UI) reads that table instead of hardcoding a list of
//! knobs, so a new scenario needs no changes downstream.

use serde::{Deserialize, Serialize};

/// Everything a UI needs to render one input, and the engine needs to validate
/// it. Const-constructible so scenarios can declare their table as a `static`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ParamSpec {
    /// Stable machine name, e.g. `"mass"`.
    pub key: &'static str,
    /// Label to show the user, e.g. `"Mass"`.
    pub label: &'static str,
    /// Unit string, e.g. `"kg"`. Empty for dimensionless parameters.
    pub unit: &'static str,
    /// Smallest accepted value, inclusive.
    pub min: f64,
    /// Largest accepted value, inclusive.
    pub max: f64,
    /// Value the scenario starts with.
    pub default: f64,
    /// Suggested slider increment.
    pub step: f64,
}

/// Why a parameter write was rejected.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamError {
    /// No parameter with that key in this scenario.
    UnknownKey(String),
    /// Value was NaN or infinite.
    NotFinite(&'static str),
    /// Value fell outside `[min, max]`.
    OutOfRange {
        key: &'static str,
        value: f64,
        min: f64,
        max: f64,
    },
}

impl std::fmt::Display for ParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamError::UnknownKey(key) => write!(f, "unknown parameter '{key}'"),
            ParamError::NotFinite(key) => write!(f, "parameter '{key}' must be a finite number"),
            ParamError::OutOfRange {
                key,
                value,
                min,
                max,
            } => write!(f, "parameter '{key}' = {value} is outside [{min}, {max}]"),
        }
    }
}

impl std::error::Error for ParamError {}

/// One parameter bound to a field of the scenario's parameter struct.
///
/// The accessors are plain function pointers, so reading a parameter during a
/// step costs a field access rather than a string lookup.
pub struct ParamDef<P: 'static> {
    pub spec: ParamSpec,
    pub get: fn(&P) -> f64,
    pub set: fn(&mut P, f64),
}

/// Declare a parameter bound to a field. Keeps the table readable:
///
/// ```ignore
/// param!(SpringParams, mass, "mass", "Mass", "kg", 0.01, 50.0, 0.25, 0.01)
/// ```
#[macro_export]
macro_rules! param {
    ($ty:ty, $field:ident, $key:expr, $label:expr, $unit:expr, $min:expr, $max:expr, $default:expr, $step:expr) => {
        $crate::params::ParamDef::<$ty> {
            spec: $crate::params::ParamSpec {
                key: $key,
                label: $label,
                unit: $unit,
                min: $min,
                max: $max,
                default: $default,
                step: $step,
            },
            get: |p: &$ty| p.$field,
            set: |p: &mut $ty, v: f64| p.$field = v,
        }
    };
}

/// Read one parameter by key. `None` means the key is not in the table.
pub fn get<P>(defs: &[ParamDef<P>], params: &P, key: &str) -> Option<f64> {
    defs.iter()
        .find(|d| d.spec.key == key)
        .map(|d| (d.get)(params))
}

/// Write one parameter by key, rejecting unknown keys and out-of-range values.
pub fn set<P>(
    defs: &[ParamDef<P>],
    params: &mut P,
    key: &str,
    value: f64,
) -> Result<(), ParamError> {
    let def = defs
        .iter()
        .find(|d| d.spec.key == key)
        .ok_or_else(|| ParamError::UnknownKey(key.to_string()))?;

    if !value.is_finite() {
        return Err(ParamError::NotFinite(def.spec.key));
    }
    if value < def.spec.min || value > def.spec.max {
        return Err(ParamError::OutOfRange {
            key: def.spec.key,
            value,
            min: def.spec.min,
            max: def.spec.max,
        });
    }

    (def.set)(params, value);
    Ok(())
}

/// Just the specs, for handing a schema to the frontend.
pub fn specs<P>(defs: &[ParamDef<P>]) -> Vec<ParamSpec> {
    defs.iter().map(|d| d.spec).collect()
}

/// Build a parameter struct from the declared defaults. Lets a scenario's
/// `Default` impl and its published schema come from one source.
pub fn defaults<P: Default>(defs: &[ParamDef<P>]) -> P {
    let mut p = P::default();
    for def in defs {
        (def.set)(&mut p, def.spec.default);
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, PartialEq, Debug)]
    struct Demo {
        mass: f64,
        angle: f64,
    }

    fn defs() -> Vec<ParamDef<Demo>> {
        vec![
            param!(Demo, mass, "mass", "Mass", "kg", 0.1, 10.0, 2.0, 0.1),
            param!(Demo, angle, "angle", "Angle", "deg", 0.0, 90.0, 30.0, 1.0),
        ]
    }

    #[test]
    fn defaults_come_from_the_table() {
        let d = defs();
        let p = defaults(&d);
        assert_eq!(p, Demo { mass: 2.0, angle: 30.0 });
    }

    #[test]
    fn set_then_get_round_trips() {
        let d = defs();
        let mut p = defaults(&d);
        set(&d, &mut p, "mass", 5.5).unwrap();
        assert_eq!(get(&d, &p, "mass"), Some(5.5));
    }

    #[test]
    fn bad_writes_are_rejected() {
        let d = defs();
        let mut p = defaults(&d);
        assert!(matches!(
            set(&d, &mut p, "nope", 1.0),
            Err(ParamError::UnknownKey(_))
        ));
        assert!(matches!(
            set(&d, &mut p, "mass", 999.0),
            Err(ParamError::OutOfRange { .. })
        ));
        assert!(matches!(
            set(&d, &mut p, "mass", f64::NAN),
            Err(ParamError::NotFinite(_))
        ));
        // A rejected write leaves the old value alone.
        assert_eq!(get(&d, &p, "mass"), Some(2.0));
    }

    #[test]
    fn specs_keep_table_order() {
        let d = defs();
        let keys: Vec<_> = specs(&d).iter().map(|s| s.key).collect();
        assert_eq!(keys, vec!["mass", "angle"]);
    }
}
