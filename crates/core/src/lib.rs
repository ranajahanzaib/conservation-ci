pub mod config;
pub mod fixtures;
pub mod report;
pub use config::*;
pub use report::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    TooLarge(String),
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
pub const MAX_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_SAMPLES: usize = 250_000;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub row: u64,
    pub t: f64,
    pub x1: f64,
    pub v1: f64,
    pub x2: f64,
    pub v2: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    #[serde(flatten)]
    pub state: State,
    pub kinetic: f64,
    pub potential: f64,
    pub energy: f64,
    pub momentum: f64,
    pub com_position: f64,
    pub com_velocity: f64,
    pub relative_displacement: f64,
    pub relative_velocity: f64,
    pub energy_deviation: f64,
    pub momentum_deviation: f64,
    pub absolute_energy_deviation: f64,
    pub absolute_momentum_deviation: f64,
    pub energy_allowed: Option<f64>,
    pub momentum_allowed: Option<f64>,
    pub monotonicity_allowed: Option<f64>,
    pub energy_violation: bool,
    pub momentum_violation: bool,
    pub monotonicity_violation: bool,
}
pub fn quantities(s: &State, c: &Config) -> Result<Sample> {
    let kinetic = 0.5 * c.masses.m1 * s.v1 * s.v1 + 0.5 * c.masses.m2 * s.v2 * s.v2;
    let r = s.x2 - s.x1 - c.spring.rest_length;
    let potential = 0.5 * c.spring.stiffness * r * r;
    let momentum = c.masses.m1 * s.v1 + c.masses.m2 * s.v2;
    let mass = c.masses.m1 + c.masses.m2;
    let com_position = (c.masses.m1 * s.x1 + c.masses.m2 * s.x2) / mass;
    let com_velocity = momentum / mass;
    if [
        kinetic,
        potential,
        kinetic + potential,
        momentum,
        com_position,
        com_velocity,
        s.v2 - s.v1,
    ]
    .iter()
    .any(|v| !v.is_finite())
    {
        return Err(Error::Invalid(format!(
            "row {}: derived quantity overflow",
            s.row
        )));
    }
    Ok(Sample {
        state: s.clone(),
        kinetic,
        potential,
        energy: kinetic + potential,
        momentum,
        com_position,
        com_velocity,
        relative_displacement: r,
        relative_velocity: s.v2 - s.v1,
        energy_deviation: 0.,
        momentum_deviation: 0.,
        absolute_energy_deviation: 0.,
        absolute_momentum_deviation: 0.,
        energy_allowed: None,
        momentum_allowed: None,
        monotonicity_allowed: None,
        energy_violation: false,
        momentum_violation: false,
        monotonicity_violation: false,
    })
}
pub fn read_csv(bytes: &[u8], c: &Config) -> Result<(Vec<State>, Vec<String>)> {
    c.validate()?;
    if bytes.len() > MAX_BYTES {
        return Err(Error::Invalid(format!("Input exceeds {MAX_BYTES} bytes")));
    }
    let mut reader = csv::ReaderBuilder::new().from_reader(bytes);
    let headers = reader.headers()?.clone();
    let mut indexes = Vec::new();
    for name in c.columns.names() {
        let matching: Vec<_> = headers
            .iter()
            .enumerate()
            .filter(|(_, s)| *s == name)
            .map(|(i, _)| i)
            .collect();
        if matching.len() != 1 {
            return Err(Error::Invalid(format!(
                "Column {name:?} must occur exactly once"
            )));
        }
        indexes.push(matching[0]);
    }
    let ignored = headers
        .iter()
        .enumerate()
        .filter(|(i, _)| !indexes.contains(i))
        .map(|(_, s)| s.to_string())
        .collect();
    let mut states: Vec<State> = Vec::new();
    for record in reader.records() {
        let record = record?;
        let row = record
            .position()
            .map_or(states.len() as u64 + 2, |p| p.line());
        let mut v = [0.; 5];
        for (j, i) in indexes.iter().enumerate() {
            v[j] = record[*i].parse::<f64>().map_err(|_| {
                Error::Invalid(format!(
                    "row {row}, column {}: expected finite number",
                    headers[*i].to_owned()
                ))
            })?;
            if !v[j].is_finite() {
                return Err(Error::Invalid(format!(
                    "row {row}: NaN and infinity are forbidden"
                )));
            }
        }
        if states.last().is_some_and(|s| v[0] <= s.t) {
            return Err(Error::Invalid(format!(
                "row {row}: timestamps must strictly increase"
            )));
        }
        states.push(State {
            row,
            t: v[0],
            x1: v[1],
            v1: v[2],
            x2: v[3],
            v2: v[4],
        });
        if states.len() > MAX_SAMPLES {
            return Err(Error::Invalid(format!("Maximum {MAX_SAMPLES} samples")));
        }
    }
    if states.len() < 2 {
        return Err(Error::Invalid("At least two samples required".into()));
    }
    Ok((states, ignored))
}
pub fn fingerprint(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub struct Analysis {
    pub report: Report,
    pub samples: Vec<Sample>,
}
pub fn analyze(bytes: &[u8], config: &Config) -> Result<Analysis> {
    let (states, ignored) = read_csv(bytes, config)?;
    let mut samples: Vec<_> = states
        .iter()
        .map(|s| quantities(s, config))
        .collect::<Result<_>>()?;
    let span = samples[samples.len() - 1].state.t - samples[0].state.t;
    if !span.is_finite() || span <= 0. {
        return Err(Error::Invalid(
            "Timestamp span exceeds finite numerical range".into(),
        ));
    }
    let e0 = samples[0].energy;
    let p0 = samples[0].momentum;
    let ea = (config.regime == Regime::ClosedConservative && config.checks.energy.enabled)
        .then(|| config.checks.energy.allowed(e0));
    let pa = (config.regime != Regime::ExternallyDriven && config.checks.momentum.enabled)
        .then(|| config.checks.momentum.allowed(p0));
    for i in 0..samples.len() {
        let previous = if i == 0 { e0 } else { samples[i - 1].energy };
        let s = &mut samples[i];
        s.energy_deviation = s.energy - e0;
        s.momentum_deviation = s.momentum - p0;
        s.absolute_energy_deviation = s.energy_deviation.abs();
        s.absolute_momentum_deviation = s.momentum_deviation.abs();
        s.energy_allowed = ea;
        s.momentum_allowed = pa;
        s.monotonicity_allowed = config
            .checks
            .energy_monotonicity
            .as_ref()
            .filter(|t| t.enabled)
            .map(|t| t.allowed(previous));
        if [s.energy_deviation, s.momentum_deviation]
            .iter()
            .any(|v| !v.is_finite())
            || [ea, pa, s.monotonicity_allowed]
                .into_iter()
                .flatten()
                .any(|v| !v.is_finite())
        {
            return Err(Error::Invalid(format!(
                "row {}: deviation or tolerance overflow",
                s.state.row
            )));
        }
        s.energy_violation = ea.is_some_and(|a| s.absolute_energy_deviation > a);
        s.momentum_violation = pa.is_some_and(|a| s.absolute_momentum_deviation > a);
        s.monotonicity_violation = i > 0
            && s.monotonicity_allowed
                .is_some_and(|a| s.energy - previous > a);
    }
    let report = Report::build(bytes, config, &samples, ignored)?;
    Ok(Analysis { report, samples })
}
