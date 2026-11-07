use crate::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Check {
    pub status: String,
    pub reason: String,
    pub unit: String,
    pub initial: f64,
    pub final_value: f64,
    pub allowed_error: Option<f64>,
    pub max_absolute_error: f64,
    pub maximum_time: f64,
    pub max_normalized_error: Option<f64>,
    pub zero_tolerance_exceeded: bool,
    pub rms_error: f64,
    pub final_deviation: f64,
    pub signed_slope: f64,
    pub early_rms: f64,
    pub late_rms: f64,
    pub first_violation_index: Option<usize>,
    pub first_violation_time: Option<f64>,
    pub violating_samples: usize,
    pub longest_violation_streak: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Violation {
    pub check: String,
    pub index: usize,
    pub row: u64,
    pub time: f64,
    pub observed: f64,
    pub reference: f64,
    pub deviation: f64,
    pub allowed: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub tool: String,
    pub tool_version: String,
    pub model: String,
    pub regime: Regime,
    pub status: String,
    pub input_sha256: String,
    pub config_sha256: String,
    pub config: Config,
    pub sample_count: usize,
    pub initial_time: f64,
    pub final_time: f64,
    pub ignored_columns: Vec<String>,
    pub checks: std::collections::BTreeMap<String, Check>,
    pub failed_checks: usize,
    pub violating_samples: usize,
    pub first_violation: Option<Violation>,
    pub violations: Vec<Violation>,
    pub violation_count: usize,
    pub violations_truncated: bool,
    pub interpretation: Vec<String>,
}
fn rms(v: impl Iterator<Item = f64>) -> f64 {
    let mut n = 0.;
    let mut scale = 0.;
    let mut sum = 1.;
    for x in v.map(f64::abs) {
        n += 1.;
        if x != 0. {
            if scale < x {
                sum = 1. + sum * (scale / x).powi(2);
                scale = x;
            } else {
                sum += (x / scale).powi(2);
            }
        }
    }
    if n == 0. {
        0.
    } else {
        scale * (sum / n).sqrt()
    }
}
impl Check {
    fn build(s: &[Sample], kind: usize, enabled: bool, reason: &str) -> Self {
        let value = |x: &Sample| if kind == 1 { x.momentum } else { x.energy };
        let deviation = |i: usize| {
            if kind == 0 {
                s[i].energy_deviation
            } else if kind == 1 {
                s[i].momentum_deviation
            } else if i == 0 {
                0.
            } else {
                s[i].energy - s[i - 1].energy
            }
        };
        let allowed = |i: usize| {
            if kind == 0 {
                s[i].energy_allowed
            } else if kind == 1 {
                s[i].momentum_allowed
            } else {
                s[i].monotonicity_allowed
            }
        };
        let failed = |i: usize| {
            if kind == 0 {
                s[i].energy_violation
            } else if kind == 1 {
                s[i].momentum_violation
            } else {
                s[i].monotonicity_violation
            }
        };
        let mut max = 0.;
        let mut mi = 0;
        let mut ratio: f64 = 0.;
        let mut zero = false;
        let mut count = 0;
        let mut first = None;
        let mut streak = 0;
        let mut longest = 0;
        for i in 0..s.len() {
            let d = deviation(i).abs();
            if d > max {
                max = d;
                mi = i;
            }
            if let Some(a) = allowed(i) {
                let normalized_deviation = if kind == 2 { deviation(i).max(0.) } else { d };
                if a == 0. {
                    zero |= normalized_deviation > 0.;
                } else {
                    ratio = ratio.max(normalized_deviation / a);
                }
            }
            if failed(i) {
                count += 1;
                first.get_or_insert(i);
                streak += 1;
                longest = longest.max(streak);
            } else {
                streak = 0;
            }
        }
        let n = s.len() as f64;
        let span = s[s.len() - 1].state.t - s[0].state.t;
        // Regress normalized time to avoid squaring large timestamps.
        let times: Vec<_> = s
            .iter()
            .map(|x| (x.state.t - s[0].state.t) / span)
            .collect();
        let mt = times.iter().sum::<f64>() / n;
        let md = (0..s.len()).map(|i| deviation(i) / n).sum::<f64>();
        let den = times.iter().map(|t| (t - mt).powi(2)).sum::<f64>();
        let slope = times
            .iter()
            .enumerate()
            .map(|(i, t)| (t - mt) * (deviation(i) - md))
            .sum::<f64>()
            / den
            / span;
        let window = (s.len() / 5).max(1);
        Self {
            status: if !enabled {
                "NOT_APPLICABLE"
            } else if count > 0 {
                "FAIL"
            } else {
                "PASS"
            }
            .into(),
            reason: reason.into(),
            unit: if kind == 1 { "kg m/s" } else { "J" }.into(),
            initial: value(&s[0]),
            final_value: value(&s[s.len() - 1]),
            allowed_error: allowed(0),
            max_absolute_error: max,
            maximum_time: s[mi].state.t,
            max_normalized_error: if !enabled || zero || !ratio.is_finite() {
                None
            } else {
                Some(ratio)
            },
            zero_tolerance_exceeded: zero,
            rms_error: rms((0..s.len()).map(deviation)),
            final_deviation: deviation(s.len() - 1),
            signed_slope: slope,
            early_rms: rms((0..window).map(deviation)),
            late_rms: rms((s.len() - window..s.len()).map(deviation)),
            first_violation_index: first,
            first_violation_time: first.map(|i| s[i].state.t),
            violating_samples: count,
            longest_violation_streak: longest,
        }
    }
}
impl Report {
    pub fn build(
        bytes: &[u8],
        c: &Config,
        s: &[Sample],
        ignored_columns: Vec<String>,
    ) -> Result<Self> {
        let energy = s[0].energy_allowed.is_some();
        let momentum = s[0].momentum_allowed.is_some();
        let mono = s[0].monotonicity_allowed.is_some();
        let mut checks = std::collections::BTreeMap::new();
        checks.insert(
            "energy".into(),
            Check::build(
                s,
                0,
                energy,
                if energy {
                    "Energy conservation under the declared closed model"
                } else {
                    "Energy conservation disabled or not justified by this regime"
                },
            ),
        );
        checks.insert(
            "momentum".into(),
            Check::build(
                s,
                1,
                momentum,
                if momentum {
                    "No declared external net force"
                } else {
                    "Momentum conservation disabled or external impulse unaccounted for"
                },
            ),
        );
        checks.insert("energy_monotonicity".into(),Check::build(s,2,mono,"Adjacent mechanical energy non-increase under declared passive internal dissipation; not an energy-conservation check"));
        for check in checks.values() {
            if ![
                check.signed_slope,
                check.rms_error,
                check.early_rms,
                check.late_rms,
                check.max_absolute_error,
                check.final_deviation,
            ]
            .iter()
            .all(|v| v.is_finite())
            {
                return Err(Error::Invalid("Diagnostic arithmetic exceeded finite range; rescale exported units or time span".into()));
            }
        }
        let mut violations = Vec::new();
        let mut violation_count = 0;
        let mut violating_samples = 0;
        for (i, x) in s.iter().enumerate() {
            if x.energy_violation || x.momentum_violation || x.monotonicity_violation {
                violating_samples += 1;
            }
            for (name, fail, observed, reference, allowed) in [
                (
                    "energy",
                    x.energy_violation,
                    x.energy,
                    s[0].energy,
                    x.energy_allowed,
                ),
                (
                    "momentum",
                    x.momentum_violation,
                    x.momentum,
                    s[0].momentum,
                    x.momentum_allowed,
                ),
                (
                    "energy_monotonicity",
                    x.monotonicity_violation,
                    x.energy,
                    if i == 0 { x.energy } else { s[i - 1].energy },
                    x.monotonicity_allowed,
                ),
            ] {
                if fail {
                    violation_count += 1;
                    if violations.len() < 1000 {
                        violations.push(Violation {
                            check: name.into(),
                            index: i,
                            row: x.state.row,
                            time: x.state.t,
                            observed,
                            reference,
                            deviation: observed - reference,
                            allowed: allowed.unwrap_or(0.),
                        });
                    }
                }
            }
        }
        let failed_checks = checks.values().filter(|v| v.status == "FAIL").count();
        let status = if failed_checks > 0 {
            "FAIL"
        } else if !energy && !momentum && !mono {
            "UNVERIFIED"
        } else {
            "PASS"
        };
        let mut interpretation=vec!["Conservation within tolerance does not prove overall physical correctness. Failures do not uniquely identify a simulator defect. The initial sampled state is the reference.".into()];
        for (name, check) in &checks {
            if let Some(t) = check.first_violation_time {
                interpretation.push(format!(
                    "{name} exceeds its configured tolerance beginning at t={t} s."
                ));
            }
        }
        if c.regime == Regime::ExternallyDriven {
            interpretation.push("External work and impulse budgets are unavailable; energy and momentum conservation were not applied.".into());
        }
        if c.regime == Regime::InternalDissipation {
            interpretation.push("Declining mechanical energy is permitted under internal dissipation; total momentum remains testable.".into());
        }
        Ok(Self {
            schema_version: 1,
            tool: "conservationci".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            model: c.model.clone(),
            regime: c.regime,
            status: status.into(),
            input_sha256: fingerprint(bytes),
            config_sha256: fingerprint(&serde_json::to_vec(c)?),
            config: c.clone(),
            sample_count: s.len(),
            initial_time: s[0].state.t,
            final_time: s[s.len() - 1].state.t,
            ignored_columns,
            checks,
            failed_checks,
            violating_samples,
            first_violation: violations.first().cloned(),
            violation_count,
            violations_truncated: violation_count > violations.len(),
            violations,
            interpretation,
        })
    }
    pub fn markdown(&self) -> String {
        let mut out = format!(
            "# ConservationCI — {}\n\nModel: `{}` · Regime: `{:?}`\n\n{} samples, t={}–{} s. {} failed checks.\n\n| Check | Status | Initial | Final | Max deviation | Allowed | First violation (s) |\n|---|---|---:|---:|---:|---:|---:|\n",
            self.status,
            self.model,
            self.regime,
            self.sample_count,
            self.initial_time,
            self.final_time,
            self.failed_checks
        );
        for (name, c) in &self.checks {
            out += &format!(
                "| {name} | {} | {:.9e} | {:.9e} | {:.9e} | {} | {} |\n",
                c.status,
                c.initial,
                c.final_value,
                c.max_absolute_error,
                c.allowed_error
                    .map(|v| format!("{v:.9e}"))
                    .unwrap_or_else(|| "N/A".into()),
                c.first_violation_time
                    .map(|v| format!("{v:.9e}"))
                    .unwrap_or_else(|| "—".into())
            );
        }
        if let Some(v) = &self.first_violation {
            out += &format!(
                "\n## First failure evidence\n\n{}: t={} s, CSV source row {}, sample index {}.\n\nObserved: {:.12e}; reference: {:.12e}; signed deviation: {:.12e}; allowed deviation: {:.12e}.\n",
                v.check, v.time, v.row, v.index, v.observed, v.reference, v.deviation, v.allowed
            );
        }
        for p in &self.interpretation {
            out += &format!("\n{p}\n");
        }
        out += &format!(
            "\nInput SHA-256: `{}`\n\nConfig SHA-256: `{}`\n",
            self.input_sha256, self.config_sha256
        );
        out
    }
}
pub fn samples_csv(samples: &[Sample]) -> Result<Vec<u8>> {
    let mut w = csv::Writer::from_writer(Vec::new());
    w.write_record([
        "row",
        "t",
        "x1",
        "v1",
        "x2",
        "v2",
        "kinetic",
        "potential",
        "energy",
        "momentum",
        "com_position",
        "com_velocity",
        "energy_deviation",
        "momentum_deviation",
        "absolute_energy_deviation",
        "absolute_momentum_deviation",
        "energy_allowed",
        "momentum_allowed",
        "monotonicity_allowed",
        "energy_violation",
        "momentum_violation",
        "monotonicity_violation",
    ])?;
    for s in samples {
        w.write_record([
            s.state.row.to_string(),
            s.state.t.to_string(),
            s.state.x1.to_string(),
            s.state.v1.to_string(),
            s.state.x2.to_string(),
            s.state.v2.to_string(),
            s.kinetic.to_string(),
            s.potential.to_string(),
            s.energy.to_string(),
            s.momentum.to_string(),
            s.com_position.to_string(),
            s.com_velocity.to_string(),
            s.energy_deviation.to_string(),
            s.momentum_deviation.to_string(),
            s.absolute_energy_deviation.to_string(),
            s.absolute_momentum_deviation.to_string(),
            s.energy_allowed.map(|v| v.to_string()).unwrap_or_default(),
            s.momentum_allowed
                .map(|v| v.to_string())
                .unwrap_or_default(),
            s.monotonicity_allowed
                .map(|v| v.to_string())
                .unwrap_or_default(),
            s.energy_violation.to_string(),
            s.momentum_violation.to_string(),
            s.monotonicity_violation.to_string(),
        ])?;
    }
    w.into_inner().map_err(|e| Error::Io(e.into_error()))
}
/// Presentation only: bucket extrema plus all crossing boundaries. Crossings can exceed the budget;
/// retain the first 1000 crossing boundaries. Full-resolution reports retain total counts.
pub fn plot_indices(s: &[Sample]) -> Vec<usize> {
    let mut set = std::collections::BTreeSet::from([0, s.len() - 1]);
    let bucket = s.len().div_ceil(400);
    for start in (0..s.len()).step_by(bucket) {
        let end = (start + bucket).min(s.len());
        for field in 0..4 {
            for max in [false, true] {
                let index = (start..end).min_by(|&a, &b| {
                    let get = |i: usize| match field {
                        0 => s[i].energy,
                        1 => s[i].momentum,
                        2 => s[i].kinetic,
                        _ => s[i].potential,
                    };
                    let order = get(a).total_cmp(&get(b));
                    if max { order.reverse() } else { order }
                });
                if let Some(i) = index {
                    set.insert(i);
                }
            }
        }
    }
    let mut crossings = 0;
    for i in 1..s.len() {
        if (
            s[i].energy_violation,
            s[i].momentum_violation,
            s[i].monotonicity_violation,
        ) != (
            s[i - 1].energy_violation,
            s[i - 1].momentum_violation,
            s[i - 1].monotonicity_violation,
        ) {
            if crossings < 1000 {
                set.insert(i - 1);
                set.insert(i);
            }
            crossings += 1;
        }
    }
    set.into_iter().collect()
}
