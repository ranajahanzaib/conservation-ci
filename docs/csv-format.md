# CSV input

Canonical header: `t,x1,v1,x2,v2`. Use UTF-8, comma delimiters, standard CSV quoting, and finite numeric values. Header names are exact and case-sensitive; use TOML column mappings for other names. Additional unmapped columns are accepted and listed in `ignored_columns`.

Every row must contain all mapped fields and the same field count as the header. Empty, NaN, infinite, and non-numeric values are errors. At least two samples are required. Timestamps may be irregular and negative but must strictly increase. The analyzer neither sorts nor drops input records. Numerics are interpreted in SI; it does not infer or convert units.

Errors include physical source start lines and, for numeric errors, mapped column names when available. CSV parser errors carry the parser's record/line position. A multiline quoted record's source row is its starting physical line. Blank lines follow the csv crate's standard behavior and are not samples.

Limits: 32 MiB raw CSV, 250,000 samples. The CLI bounds file reading to limit+1 before rejecting. The API bounds the entire JSON request to 33 MiB; escaped CSV can reach that envelope before the raw CSV limit. Finite input that overflows derived energy, tolerances, time span, or diagnostics is rejected instead of exporting NaN metrics.

Export `--samples-csv` includes every sample's original state, source line, energy components, momentum, COM coordinates, signed/absolute errors, allowed errors, and individual violation flags. Missing allowed-error cells mean the check is not applicable.
