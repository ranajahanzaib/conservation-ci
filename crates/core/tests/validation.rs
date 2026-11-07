use conservationci_core::*;
fn cfg() -> Config {
    Config::parse(&fixtures::config_text("analytic")).unwrap()
}
#[test]
fn invalid_csv_contracts() {
    for data in [
        "t,x1,v1,x2\n0,0,0,1\n1,0,0,1\n",
        "t,x1,v1,x2,v2\n",
        "t,x1,v1,x2,v2\n0,0,0,1,0\n0,0,0,1,0\n",
        "t,x1,v1,x2,v2\n1,0,0,1,0\n0,0,0,1,0\n",
        "t,x1,v1,x2,v2\n0,0,0,1,0\n1,0,0\n",
        "t,x1,v1,x2,v2,t\n0,0,0,1,0,0\n1,0,0,1,0,1\n",
    ] {
        assert!(analyze(data.as_bytes(), &cfg()).is_err(), "{data}");
    }
}
#[test]
fn numeric_rows_are_exact() {
    for value in ["", "NaN", "inf", "-inf", "nope", "1e999"] {
        let data = format!("t,x1,v1,x2,v2\n0,0,0,1,0\n1,{value},0,1,0\n");
        let error = analyze(data.as_bytes(), &cfg()).err().unwrap().to_string();
        assert!(error.contains("row 3"), "{error}");
    }
}
#[test]
fn invalid_models_and_tolerances() {
    for field in 0..10 {
        let mut c = cfg();
        match field {
            0 => c.masses.m1 = 0.,
            1 => c.masses.m2 = -1.,
            2 => c.spring.stiffness = 0.,
            3 => c.spring.rest_length = -1.,
            4 => c.checks.energy.absolute_tolerance = -1.,
            5 => c.checks.momentum.relative_tolerance = f64::NAN,
            6 => c.model = "unknown".into(),
            7 => c.columns.x1 = "absent".into(),
            8 => c.columns.x1 = c.columns.v1.clone(),
            _ => c.checks.energy.reference_scale = Some(0.),
        };
        assert!(analyze(b"t,x1,v1,x2,v2\n0,0,0,1,0\n1,0,0,1,0\n", &c).is_err());
    }
}
#[test]
fn invalid_config_syntax_assumptions_and_unknown_fields() {
    for s in [
        "not toml".to_owned(),
        fixtures::config_text("analytic").replace("two_mass_spring_1d", "other"),
        fixtures::config_text("analytic")
            + "\n[checks.energy_monotonicity]\nabsolute_tolerance=0.1\n",
        fixtures::config_text("analytic").replace("units = \"SI\"", "units = \"CGS\""),
        fixtures::config_text("analytic") + "\n[unsupported]\nfoo=1\n",
    ] {
        assert!(Config::parse(&s).is_err());
    }
}
#[test]
fn derived_overflow_rejected() {
    assert!(analyze(b"t,x1,v1,x2,v2\n0,0,1e308,1,0\n1,0,0,1,0\n", &cfg()).is_err());
}
#[test]
fn oversized_input() {
    assert!(read_csv(&vec![b' '; MAX_BYTES + 1], &cfg()).is_err());
}
#[test]
fn mapping_is_used() {
    let mut c = cfg();
    c.columns.time = "seconds".into();
    let a = analyze(b"seconds,x1,v1,x2,v2\n0,0,0,1,0\n1,0,0,1,0\n", &c).unwrap();
    assert_eq!(a.report.status, "PASS");
}
#[test]
fn extreme_time_and_diagnostic_ranges_do_not_leak_nan() {
    let c = cfg();
    assert!(analyze(b"t,x1,v1,x2,v2\n-1e308,0,0,1,0\n1e308,0,0,1,0\n", &c).is_err());
    let result = analyze(b"t,x1,v1,x2,v2\n0,0,0,1,0\n1e-320,0,1,1,0\n", &c);
    assert!(result.is_err());
}
