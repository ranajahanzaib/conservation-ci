use crate::{Error, Result};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub model: String,
    pub regime: Regime,
    pub units: String,
    pub masses: Masses,
    pub spring: Spring,
    #[serde(default)]
    pub columns: Columns,
    #[serde(default)]
    pub checks: Checks,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    ClosedConservative,
    InternalDissipation,
    ExternallyDriven,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Masses {
    pub m1: f64,
    pub m2: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spring {
    pub stiffness: f64,
    pub rest_length: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Columns {
    pub time: String,
    pub x1: String,
    pub v1: String,
    pub x2: String,
    pub v2: String,
}
impl Default for Columns {
    fn default() -> Self {
        Self {
            time: "t".into(),
            x1: "x1".into(),
            v1: "v1".into(),
            x2: "x2".into(),
            v2: "v2".into(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Tolerance {
    pub enabled: bool,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
    pub reference_scale: Option<f64>,
}
impl Default for Tolerance {
    fn default() -> Self {
        Self {
            enabled: true,
            absolute_tolerance: 1e-8,
            relative_tolerance: 1e-6,
            reference_scale: None,
        }
    }
}
impl Tolerance {
    pub fn allowed(&self, q: f64) -> f64 {
        self.absolute_tolerance
            + self.relative_tolerance * q.abs().max(self.reference_scale.unwrap_or(0.))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Checks {
    pub energy: Tolerance,
    pub momentum: Tolerance,
    pub energy_monotonicity: Option<Tolerance>,
}
impl Default for Checks {
    fn default() -> Self {
        Self {
            energy: Tolerance {
                absolute_tolerance: 1e-6,
                relative_tolerance: 0.01,
                ..Tolerance::default()
            },
            momentum: Tolerance::default(),
            energy_monotonicity: None,
        }
    }
}
impl Config {
    pub fn parse(s: &str) -> Result<Self> {
        let c: Self = toml::from_str(s).map_err(|e| Error::Invalid(e.to_string()))?;
        c.validate()?;
        Ok(c)
    }
    pub fn validate(&self) -> Result<()> {
        let bad = |s: &str| Err(Error::Invalid(s.into()));
        if self.schema_version != 1 || self.model != "two_mass_spring_1d" || self.units != "SI" {
            return bad("Require schema_version=1, model=two_mass_spring_1d and units=SI");
        }
        for (name, v) in [
            ("m1", self.masses.m1),
            ("m2", self.masses.m2),
            ("stiffness", self.spring.stiffness),
        ] {
            if !v.is_finite() || v <= 0. {
                return bad(&format!("{name} must be finite and positive"));
            }
        }
        if !self.spring.rest_length.is_finite()
            || self.spring.rest_length < 0.
            || !(self.masses.m1 + self.masses.m2).is_finite()
        {
            return bad("Invalid rest length or total mass");
        }
        for t in [&self.checks.energy, &self.checks.momentum]
            .into_iter()
            .chain(self.checks.energy_monotonicity.iter())
        {
            if [t.absolute_tolerance, t.relative_tolerance]
                .iter()
                .any(|v| !v.is_finite() || *v < 0.)
                || t.reference_scale.is_some_and(|v| !v.is_finite() || v <= 0.)
            {
                return bad(
                    "Tolerances must be finite and nonnegative; reference_scale must be positive",
                );
            }
        }
        if self.checks.energy_monotonicity.is_some() && self.regime != Regime::InternalDissipation {
            return bad("energy_monotonicity requires internal_dissipation");
        }
        let names = self.columns.names();
        let mut seen = std::collections::HashSet::new();
        if names.iter().any(|s| s.is_empty() || !seen.insert(*s)) {
            return bad("Column mappings must be nonempty and distinct");
        }
        Ok(())
    }
}
impl Columns {
    pub fn names(&self) -> [&str; 5] {
        [&self.time, &self.x1, &self.v1, &self.x2, &self.v2]
    }
}
