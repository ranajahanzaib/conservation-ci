# JSON report schema 1

The canonical JSON report has `schema_version=1`, `tool="conservationci"`, `tool_version`, `model`, `regime`, `status`, parsed `config`, `sample_count`, `initial_time`, `final_time`, `input_sha256`, `config_sha256`, `ignored_columns`, `checks`, `failed_checks`, `violating_samples`, `first_violation`, `violation_count`, `violations`, `violations_truncated`, and `interpretation`.

Overall status is FAIL when at least one applicable check fails, PASS when at least one applicable check exists and all pass, and UNVERIFIED when none apply. Individual check status is PASS, FAIL, or NOT_APPLICABLE. Disabled and regime-inapplicable checks include explanatory reasons and no allowed_error. N/A metrics may still be calculated diagnostically; they are not evidence of passing a test.

`checks` is a stable ordered map with `energy`, `momentum`, and `energy_monotonicity`. Each includes status/reason/unit, initial/final value, allowed_error, max_absolute_error/maximum_time, max_normalized_error, zero_tolerance_exceeded, rms_error, final_deviation, signed_slope, early_rms, late_rms, first_violation_index/time, violating_samples, and longest_violation_streak. Early/late windows each contain floor(n/5) samples (minimum one); they are sample-count windows, not time-weighted intervals. RMS likewise weights exported samples equally. OLS uses actual irregular times. A slope is descriptive, not a drift or causal classifier.

Conservation errors are Qᵢ−Q₀. Monotonicity diagnostic errors are Eᵢ−Eᵢ₋₁, with a zero first entry. Its max absolute diagnostic error includes decreases even though only positive excess is a violation. Its allowed_error summary is the initial local allowance; per-sample CSV contains each local allowance.

A normalized conservation error is abs(error)/allowed. Zero allowed + zero error produces zero. Zero allowed + nonzero error produces null and zero_tolerance_exceeded=true. Unrepresentable normalized ratios also become null. No infinities or NaN enter JSON. Non-applicable normalized ratios are null. Monotonicity ratios use the positive part of the adjacent energy increase, so permitted decreases do not produce misleading failing ratios.

Each violation includes check, zero-based index, physical source row, time, observed value, reference, signed deviation and allowed error. Events are ordered by sample then energy, momentum, monotonicity. At most 1,000 event details are included; total counts and aggregate diagnostics always cover all samples. The dashboard shows the first 100. Use the full-sample CSV for complete evidence.

SHA-256 input hashes cover exact CSV bytes; configuration hashes cover the serialized parsed/default-expanded config. Whitespace changes to equivalent TOML do not change the config hash. Reports omit wall-clock timestamps and file paths. Tool/build/platform changes can alter last-bit floating-point behavior; reproducibility means identical input bytes, normalized config, and executable/platform.

HTTP `/api/analyze` wraps the report with `samples` (presentation subset), `indices` (original zero-based indices), `full_sample_count`, and backend-generated `markdown`. The standalone report does not include per-sample arrays. Export JSON uses the same report as CLI analysis.
