export interface Sample {
  row: number;
  t: number;
  x1: number;
  v1: number;
  x2: number;
  v2: number;
  kinetic: number;
  potential: number;
  energy: number;
  momentum: number;
  com_position: number;
  com_velocity: number;
  relative_displacement: number;
  relative_velocity: number;
  energy_deviation: number;
  momentum_deviation: number;
  absolute_energy_deviation: number;
  absolute_momentum_deviation: number;
  energy_allowed: number | null;
  momentum_allowed: number | null;
  monotonicity_allowed: number | null;
  energy_violation: boolean;
  momentum_violation: boolean;
  monotonicity_violation: boolean;
}
export interface Check {
  status: string;
  reason: string;
  unit: string;
  initial: number;
  final_value: number;
  allowed_error: number | null;
  max_absolute_error: number;
  maximum_time: number;
  max_normalized_error: number | null;
  zero_tolerance_exceeded: boolean;
  rms_error: number;
  final_deviation: number;
  signed_slope: number;
  early_rms: number;
  late_rms: number;
  first_violation_index: number | null;
  first_violation_time: number | null;
  violating_samples: number;
  longest_violation_streak: number;
}
export interface Violation {
  check: string;
  index: number;
  row: number;
  time: number;
  observed: number;
  reference: number;
  deviation: number;
  allowed: number;
}
export interface Report {
  status: string;
  model: string;
  regime: string;
  sample_count: number;
  initial_time: number;
  final_time: number;
  checks: Record<string, Check>;
  failed_checks: number;
  violating_samples: number;
  violations: Violation[];
  violation_count: number;
  violations_truncated: boolean;
  first_violation: Violation | null;
  input_sha256: string;
  config_sha256: string;
  ignored_columns: string[];
  interpretation: string[];
}
export interface View {
  report: Report;
  samples: Sample[];
  indices: number[];
  full_sample_count: number;
  markdown: string;
}
