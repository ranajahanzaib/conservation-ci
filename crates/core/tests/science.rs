use conservationci_core::*;
fn cfg(id: &str) -> Config {
    Config::parse(&fixtures::config_text(id)).unwrap()
}
fn fixture(id: &str) -> Analysis {
    analyze(&fixtures::generate(id).unwrap(), &cfg(id)).unwrap()
}
#[test]
fn known_initial_values() {
    let s = State {
        row: 2,
        t: 0.,
        x1: 0.,
        v1: 0.4,
        x2: 1.2,
        v2: -0.1,
    };
    let q = quantities(&s, &cfg("analytic")).unwrap();
    for (a, b) in [
        (q.kinetic, 0.09),
        (q.potential, 0.06),
        (q.energy, 0.15),
        (q.momentum, 0.2),
        (q.com_velocity, 1. / 15.),
        (q.com_position, 0.8),
    ] {
        assert!((a - b).abs() < 1e-14, "{a} != {b}");
    }
}
#[test]
fn analytic_conservation_and_known_quarter_period() {
    let a = fixture("analytic");
    assert_eq!(a.report.status, "PASS");
    for s in a.samples {
        assert!((s.energy - 0.15).abs() < 1e-13);
        assert!((s.momentum - 0.2).abs() < 1e-14);
        assert!((s.com_position - (0.8 + s.state.t / 15.)).abs() < 1e-14);
    }
    assert!(a.report.checks["energy"].max_absolute_error < 1e-13);
}
#[test]
fn euler_fails_energy_only() {
    let a = fixture("euler");
    assert_eq!(a.report.checks["energy"].status, "FAIL");
    assert_eq!(a.report.checks["momentum"].status, "PASS");
    assert!(a.report.checks["energy"].late_rms > a.report.checks["energy"].early_rms * 5.);
    assert!(a.samples.last().unwrap().energy > a.samples[0].energy * 5.);
}
#[test]
fn verlet_bounded_under_declared_tolerance() {
    let a = fixture("verlet");
    assert_eq!(a.report.status, "PASS");
    assert!(a.report.checks["energy"].max_absolute_error > 1e-6);
    assert!(a.report.checks["energy"].max_absolute_error < 0.0001);
}
#[test]
fn independent_corruptions_and_source_evidence() {
    for (id, failed, passed) in [
        ("energy-corruption", "energy", "momentum"),
        ("momentum-corruption", "momentum", "energy"),
    ] {
        let a = fixture(id);
        assert_eq!(a.report.checks[failed].status, "FAIL");
        assert_eq!(a.report.checks[passed].status, "PASS");
        assert_eq!(a.report.checks[failed].first_violation_index, Some(250));
        assert_eq!(a.report.first_violation.as_ref().unwrap().row, 252);
        assert_eq!(a.report.first_violation.as_ref().unwrap().time, 5.);
        assert_eq!(a.report.violating_samples, 1);
        assert_eq!(a.report.failed_checks, 1);
        let max = a
            .samples
            .iter()
            .map(|s| {
                if failed == "energy" {
                    s.absolute_energy_deviation
                } else {
                    s.absolute_momentum_deviation
                }
            })
            .fold(0., f64::max);
        assert_eq!(a.report.checks[failed].max_absolute_error, max);
    }
}
#[test]
fn damping_is_not_energy_conservation() {
    let a = fixture("damped");
    assert_eq!(a.report.status, "PASS");
    assert_eq!(a.report.checks["energy"].status, "NOT_APPLICABLE");
    assert_eq!(a.report.checks["energy_monotonicity"].status, "PASS");
    assert!(a.samples.last().unwrap().energy < 0.01);
    assert_eq!(a.report.checks["momentum"].status, "PASS");
}
#[test]
fn driven_is_unverified() {
    let a = fixture("driven");
    assert_eq!(a.report.status, "UNVERIFIED");
    assert_eq!(a.report.failed_checks, 0);
    assert!(
        a.report
            .checks
            .values()
            .all(|c| c.status == "NOT_APPLICABLE")
    );
    assert!(a.report.violations.is_empty());
}
#[test]
fn equality_zero_and_near_zero_tolerances() {
    let mut c = cfg("analytic");
    c.checks.energy.enabled = false;
    c.checks.momentum.absolute_tolerance = 0.5;
    c.checks.momentum.relative_tolerance = 0.;
    let data = b"t,x1,v1,x2,v2\n0,0,0,1,0\n1,0,0.5,1,0\n";
    let a = analyze(data, &c).unwrap();
    assert_eq!(a.report.status, "PASS");
    assert_eq!(a.report.checks["momentum"].max_normalized_error, Some(1.));
    c.checks.momentum.absolute_tolerance = 0.;
    let a = analyze(data, &c).unwrap();
    assert_eq!(a.report.status, "FAIL");
    assert!(a.report.checks["momentum"].zero_tolerance_exceeded);
    assert!(a.report.checks["momentum"].max_normalized_error.is_none());
    serde_json::to_string(&a.report).unwrap();
    let a = analyze(b"t,x1,v1,x2,v2\n0,0,0,1,0\n1,0,0,1,0\n", &c).unwrap();
    assert_eq!(a.report.status, "PASS");
    assert_eq!(a.report.checks["momentum"].max_normalized_error, Some(0.));
    c.checks.momentum.absolute_tolerance = 1e-20;
    assert_eq!(
        analyze(b"t,x1,v1,x2,v2\n0,0,1e-30,1,0\n1,0,2e-30,1,0\n", &c)
            .unwrap()
            .report
            .status,
        "PASS"
    );
}
#[test]
fn disabled_checks() {
    let mut c = cfg("analytic");
    c.checks.energy.enabled = false;
    c.checks.momentum.enabled = false;
    assert_eq!(
        analyze(&fixtures::generate("euler").unwrap(), &c)
            .unwrap()
            .report
            .status,
        "UNVERIFIED"
    );
}
#[test]
fn irregular_sampling_and_extra_columns() {
    let data = b"t,x1,v1,x2,v2,unused\n0,0,0,1,0,a\n0.03,0,0,1,0,b\n4.8,0,0,1,0,c\n";
    let a = analyze(data, &cfg("analytic")).unwrap();
    assert_eq!(a.report.status, "PASS");
    assert_eq!(a.report.ignored_columns, vec!["unused"]);
    assert_eq!(a.samples[2].state.row, 4);
}
#[test]
fn reproducible_reports_and_exports() {
    for id in fixtures::IDS {
        let a = fixture(id);
        let b = fixture(id);
        assert_eq!(
            serde_json::to_vec(&a.report).unwrap(),
            serde_json::to_vec(&b.report).unwrap()
        );
        assert_eq!(a.report.schema_version, 1);
        assert_eq!(a.report.input_sha256.len(), 64);
        assert!(!a.report.markdown().is_empty());
        let csv = samples_csv(&a.samples).unwrap();
        let mut reader = csv::Reader::from_reader(csv.as_slice());
        assert_eq!(reader.records().count(), 1001);
    }
}
#[test]
fn correct_translation_and_velocity_shift_properties() {
    let c = cfg("analytic");
    for offset in [-100., -1., 0., 3., 100.] {
        for u in [-2., 0., 1.] {
            let s = State {
                row: 2,
                t: 0.,
                x1: 0.2,
                v1: 0.4,
                x2: 1.3,
                v2: -0.1,
            };
            let q = quantities(&s, &c).unwrap();
            let moved = State {
                x1: s.x1 + offset,
                x2: s.x2 + offset,
                ..s.clone()
            };
            let qm = quantities(&moved, &c).unwrap();
            assert!((qm.energy - q.energy).abs() < 1e-12);
            assert_eq!(qm.momentum, q.momentum);
            assert!((qm.relative_displacement - q.relative_displacement).abs() < 1e-13);
            let boosted = State {
                v1: s.v1 + u,
                v2: s.v2 + u,
                ..s
            };
            let qb = quantities(&boosted, &c).unwrap();
            assert!((qb.momentum - q.momentum - 3. * u).abs() < 1e-13);
        }
    }
}
#[test]
fn label_swap_with_oriented_rest_length_transformation() {
    // Reflect coordinates while swapping labels so positive rest length retains orientation.
    let c = cfg("analytic");
    let mut swapped = c.clone();
    swapped.masses.m1 = c.masses.m2;
    swapped.masses.m2 = c.masses.m1;
    let s = State {
        row: 2,
        t: 0.,
        x1: -0.2,
        v1: 0.4,
        x2: 1.4,
        v2: -0.1,
    };
    let r = State {
        row: 2,
        t: 0.,
        x1: -s.x2,
        v1: -s.v2,
        x2: -s.x1,
        v2: -s.v1,
    };
    assert!(
        (quantities(&s, &c).unwrap().energy - quantities(&r, &swapped).unwrap().energy).abs()
            < 1e-14
    );
}
#[test]
fn large_input_full_resolution_and_reduction() {
    let mut csv = String::from("t,x1,v1,x2,v2\n");
    for i in 0..100_000 {
        csv += &format!("{},0,0,1,0\n", i as f64 * 0.017);
    }
    let a = analyze(csv.as_bytes(), &cfg("analytic")).unwrap();
    assert_eq!(a.samples.len(), 100_000);
    let p = plot_indices(&a.samples);
    assert_eq!(p[0], 0);
    assert_eq!(*p.last().unwrap(), 99_999);
    assert!(p.len() < 4000);
}
#[test]
fn reduction_preserves_crossings_and_extrema() {
    let a = fixture("energy-corruption");
    let p = plot_indices(&a.samples);
    assert!(p.contains(&249) && p.contains(&250) && p.contains(&251));
}
#[test]
fn monotonicity_detects_increase_with_local_reference() {
    let mut c = cfg("damped");
    c.checks
        .energy_monotonicity
        .as_mut()
        .unwrap()
        .absolute_tolerance = 0.01;
    c.checks
        .energy_monotonicity
        .as_mut()
        .unwrap()
        .relative_tolerance = 0.;
    let a = analyze(b"t,x1,v1,x2,v2\n0,0,0,1,0\n1,0,0,1.2,0\n2,0,0,1.1,0\n", &c).unwrap();
    assert_eq!(a.report.checks["energy"].status, "NOT_APPLICABLE");
    assert_eq!(a.report.checks["energy_monotonicity"].status, "FAIL");
    assert_eq!(a.report.first_violation.unwrap().time, 1.);
    assert_eq!(a.report.violating_samples, 1);
}
#[test]
fn explicit_reference_scale_is_dimensionful_and_optional() {
    let mut c = cfg("analytic");
    c.checks.energy.enabled = false;
    c.checks.momentum.absolute_tolerance = 0.;
    c.checks.momentum.relative_tolerance = 0.1;
    c.checks.momentum.reference_scale = Some(2.);
    let a = analyze(b"t,x1,v1,x2,v2\n0,0,0,1,0\n1,0,0.1,1,0\n", &c).unwrap();
    assert_eq!(a.report.status, "PASS");
    assert_eq!(a.report.checks["momentum"].allowed_error, Some(0.2));
}
#[test]
fn generator_refuses_existing_files_and_is_repeatable() {
    let dir = std::env::temp_dir().join(format!("conservationci-generator-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    fixtures::write_all(&dir).unwrap();
    let bytes = std::fs::read(dir.join("analytic.csv")).unwrap();
    assert_eq!(bytes, fixtures::generate("analytic").unwrap());
    assert!(fixtures::write_all(&dir).is_err());
    assert_eq!(bytes, std::fs::read(dir.join("analytic.csv")).unwrap());
    std::fs::remove_dir_all(dir).unwrap();
}
