use crate::*;
pub const IDS: [&str; 7] = [
    "analytic",
    "euler",
    "verlet",
    "energy-corruption",
    "momentum-corruption",
    "damped",
    "driven",
];
pub fn config_text(id: &str) -> String {
    format!(
        "schema_version = 1\nmodel = \"two_mass_spring_1d\"\nregime = \"{}\"\nunits = \"SI\"\n[masses]\nm1 = 1.0\nm2 = 2.0\n[spring]\nstiffness = 3.0\nrest_length = 1.0\n[checks.energy]\nabsolute_tolerance = 0.000001\nrelative_tolerance = 0.01\n[checks.momentum]\nabsolute_tolerance = 0.00000001\nrelative_tolerance = 0.000001\n{}",
        if id == "damped" {
            "internal_dissipation"
        } else if id == "driven" {
            "externally_driven"
        } else {
            "closed_conservative"
        },
        if id == "damped" {
            "[checks.energy_monotonicity]\nabsolute_tolerance = 0.00000001\nrelative_tolerance = 0.000001\n"
        } else {
            ""
        }
    )
}
pub fn generate(id: &str) -> Result<Vec<u8>> {
    if !IDS.contains(&id) {
        return Err(Error::Invalid("Unknown fixture".into()));
    }
    let mut w = csv::Writer::from_writer(Vec::new());
    w.write_record(["t", "x1", "v1", "x2", "v2"])?;
    let (mut x1, mut v1, mut x2, mut v2) = (0., 0.4, 1.2, -0.1);
    let dt = 0.02;
    let omega = 4.5_f64.sqrt();
    for i in 0..=1000 {
        let t = i as f64 * dt;
        let (mut a, mut b, mut d, mut e) = if id == "euler" || id == "verlet" {
            (x1, v1, x2, v2)
        } else {
            let (r, u) = if id == "damped" {
                let gamma = 0.15;
                let wd = (4.5_f64 - gamma * gamma).sqrt();
                let b = (-0.5 + gamma * 0.2) / wd;
                let r = (-gamma * t).exp() * (0.2 * (wd * t).cos() + b * (wd * t).sin());
                let u = -gamma * r
                    + (-gamma * t).exp() * wd * (-0.2 * (wd * t).sin() + b * (wd * t).cos());
                (r, u)
            } else {
                (
                    0.2 * (omega * t).cos() - 0.5 / omega * (omega * t).sin(),
                    -0.2 * omega * (omega * t).sin() - 0.5 * (omega * t).cos(),
                )
            };
            let com = 0.8 + t / 15.;
            (
                com - 2. / 3. * (1. + r),
                1. / 15. - 2. / 3. * u,
                com + 1. / 3. * (1. + r),
                1. / 15. + u / 3.,
            )
        };
        if id == "energy-corruption" && i == 250 {
            b += 0.2;
            e -= 0.1;
        }
        if id == "momentum-corruption" && i == 250 {
            b -= 2. / 15.;
            e -= 2. / 15.;
        }
        if id == "driven" {
            a += 0.05 * t * t;
            d += 0.05 * t * t;
            b += 0.1 * t;
            e += 0.1 * t;
        }
        w.write_record([t, a, b, d, e].map(|v| format!("{v:.17}")))?;
        let f = 3. * (x2 - x1 - 1.);
        if id == "euler" {
            x1 += dt * v1;
            x2 += dt * v2;
            v1 += dt * f;
            v2 -= dt * f / 2.;
        } else if id == "verlet" {
            x1 += dt * v1 + 0.5 * dt * dt * f;
            x2 += dt * v2 - 0.25 * dt * dt * f;
            let nf = 3. * (x2 - x1 - 1.);
            v1 += 0.5 * dt * (f + nf);
            v2 -= 0.25 * dt * (f + nf);
        }
    }
    w.into_inner().map_err(|e| Error::Io(e.into_error()))
}
pub fn write_all(output: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(output)?;
    for id in IDS {
        for (name, bytes) in [
            (format!("{id}.csv"), generate(id)?),
            (format!("{id}.toml"), config_text(id).into_bytes()),
        ] {
            let path = output.join(name);
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)?;
            f.write_all(&bytes)?;
        }
    }
    Ok(())
}
