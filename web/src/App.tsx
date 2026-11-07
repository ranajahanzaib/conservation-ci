import {
  Activity,
  ArrowDownToLine,
  ArrowRight,
  CheckCircle2,
  ChevronDown,
  FileUp,
  FlaskConical,
  Pause,
  Play,
  Settings2,
  ShieldCheck,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import {
  Brush,
  CartesianGrid,
  Line,
  LineChart,
  ReferenceLine,
  ResponsiveContainer,
  Scatter,
  ScatterChart,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { csvHeaders } from "./csv";
import type { Check, Sample, View } from "./types";
const examples = [
  ["analytic", "Analytic reference", "Exact solution · expected pass"],
  ["euler", "Explicit Euler", "Accumulating energy error"],
  ["verlet", "Velocity Verlet", "Bounded numerical energy error"],
  [
    "energy-corruption",
    "Energy corruption",
    "One perturbed sample · momentum preserved",
  ],
  [
    "momentum-corruption",
    "Momentum corruption",
    "COM reversal · energy preserved",
  ],
  ["damped", "Internal damping", "Mechanical energy loss is expected"],
  ["driven", "External forcing", "Conservation checks not applicable"],
];
const fmt = (n: number | null | undefined) =>
  n == null
    ? "—"
    : Math.abs(n) > 0 && Math.abs(n) < 0.0001
      ? n.toExponential(4)
      : n.toLocaleString("en-US", { maximumFractionDigits: 7 });
async function response<T>(url: string, body?: unknown): Promise<T> {
  const r = await fetch(
    url,
    body
      ? {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body),
        }
      : undefined,
  );
  if (!r.ok) {
    const text = await r.text();
    try {
      throw new Error((JSON.parse(text) as { error: string }).error || text);
    } catch (e) {
      if (e instanceof SyntaxError) throw new Error(text);
      throw e;
    }
  }
  return r.json() as Promise<T>;
}
function Badge({ status }: { status: string }) {
  return (
    <span className={`badge ${status.toLowerCase()}`}>
      {status === "PASS" ? <CheckCircle2 size={12} /> : <Activity size={12} />}{" "}
      {status.replaceAll("_", " ")}
    </span>
  );
}
function Metric({ name, check }: { name: string; check: Check }) {
  return (
    <article className="metric">
      <div className="flex">
        <h3>{name}</h3>
        <Badge status={check.status} />
      </div>
      <p className="metric-value">
        {fmt(check.max_absolute_error)} <small>{check.unit}</small>
      </p>
      <p>Maximum absolute deviation</p>
      <div className="metric-bottom">
        <span>
          Allowed <b>{fmt(check.allowed_error)}</b>
        </span>
        <span>
          First crossing <b>{fmt(check.first_violation_time)} s</b>
        </span>
      </div>
    </article>
  );
}
export default function App() {
  const [csv, setCsv] = useState("");
  const [config, setConfig] = useState("");
  const [filename, setFilename] = useState("No trajectory selected");
  const [selectedExample, setSelectedExample] = useState("analytic");
  const [view, setView] = useState<View | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [tab, setTab] = useState<"energy" | "momentum" | "phase">("energy");
  const [mode, setMode] = useState("total");
  const [selected, setSelected] = useState(0);
  const [exactSample, setExactSample] = useState<Sample | null>(null);
  const [playing, setPlaying] = useState(false);
  const [showConfig, setShowConfig] = useState(false);
  const [mapping, setMapping] = useState<Record<string, string>>({});
  const input = useRef<HTMLInputElement>(null);
  const requestId = useRef(0);
  const active = useRef<{ csv: string; config: string } | null>(null);
  const run = async (data: { csv: string; config: string }) => {
    const id = ++requestId.current;
    setBusy(true);
    setError("");
    setMessage("");
    try {
      const result = await response<View>("/api/analyze", data);
      if (id === requestId.current) {
        setView(result);
        active.current = data;
        setSelected(0);
        setExactSample(null);
        setPlaying(false);
      }
    } catch (e) {
      if (id === requestId.current)
        setError(String(e instanceof Error ? e.message : e));
    } finally {
      if (id === requestId.current) setBusy(false);
    }
  };
  const load = async (id: string) => {
    setSelectedExample(id);
    setBusy(true);
    setError("");
    try {
      const data = await response<{ csv: string; config: string }>(
        `/api/examples/${id}`,
      );
      setCsv(data.csv);
      setConfig(data.config);
      setFilename(`${id}.csv`);
      setMapping({});
      await run(data);
    } catch (e) {
      setError(String(e));
      setBusy(false);
    }
  };
  // No automatic requests: the empty state makes dataset selection explicit.
  useEffect(() => {
    if (!playing || !view) return;
    const timer = window.setInterval(
      () =>
        setSelected((i) => {
          if (i >= view.samples.length - 1) {
            setPlaying(false);
            return i;
          }
          return i + 1;
        }),
      60,
    );
    return () => clearInterval(timer);
  }, [playing, view]);
  const upload = async (file: File) => {
    if (file.size > 32 * 1024 * 1024) {
      setError("File exceeds the 32 MiB input limit.");
      return;
    }
    try {
      setCsv(await file.text());
      setFilename(file.name);
      setView(null);
      active.current = null;
      setExactSample(null);
      setPlaying(false);
      setError("");
      setMapping({});
      setMessage("Trajectory loaded. Review the model and run analysis.");
      setShowConfig(true);
      if (!config) {
        const data = await response<{ config: string }>(
          "/api/examples/analytic",
        );
        setConfig(data.config);
      }
    } catch (e) {
      setError(String(e));
    }
  };
  const mappedConfig = () => {
    if (!Object.keys(mapping).length) return config;
    const section = config.match(/\[columns\][\s\S]*?(?=\n\[|$)/)?.[0];
    let updated = section || "[columns]\n";
    for (const [key, value] of Object.entries(mapping)) {
      const assignment = `${key} = ${JSON.stringify(value)}`;
      const pattern = new RegExp(`^\\s*${key}\\s*=.*$`, "m");
      updated = pattern.test(updated)
        ? updated.replace(pattern, () => assignment)
        : updated.trimEnd() + "\n" + assignment + "\n";
    }
    return section
      ? config.replace(section, () => updated)
      : config + "\n" + updated;
  };

  const download = async (format: string) => {
    if (!active.current) return;
    try {
      const r = await fetch(`/api/reports/${format}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(active.current),
      });
      if (!r.ok) throw new Error(await r.text());
      const url = URL.createObjectURL(await r.blob());
      const a = document.createElement("a");
      a.href = url;
      a.download = `conservationci.${format === "markdown" ? "md" : format}`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(String(e));
    }
  };
  const s = exactSample || view?.samples[selected];
  const report = view?.report;
  const ec = report?.checks.energy;
  const mc = report?.checks.momentum;
  const selectTime = (t: number) => {
    if (view) {
      let lo = 0,
        hi = view.samples.length - 1;
      while (lo < hi) {
        const mid = (lo + hi) >> 1;
        if (view.samples[mid].t < t) lo = mid + 1;
        else hi = mid;
      }
      setSelected(lo);
      setExactSample(null);
      setPlaying(false);
    }
  };
  const key =
    tab === "momentum"
      ? mode === "signed"
        ? "momentum_deviation"
        : mode === "absolute"
          ? "absolute_momentum_deviation"
          : "momentum"
      : mode === "signed"
        ? "energy_deviation"
        : mode === "absolute"
          ? "absolute_energy_deviation"
          : "energy";
  const check = tab === "momentum" ? mc : ec;
  const applicable = check?.status !== "NOT_APPLICABLE";
  const monotonicity = report?.checks.energy_monotonicity;
  const baseline =
    mode === "signed" || mode === "absolute" ? 0 : check?.initial;
  const chartSamples =
    exactSample && view && !view.samples.some((p) => p.t === exactSample.t)
      ? [...view.samples, exactSample].sort((a, b) => a.t - b.t)
      : view?.samples;
  const chartData = chartSamples?.map((p) => ({
    ...p,
    marker: (tab === "momentum" ? p.momentum_violation : p.energy_violation)
      ? p[key]
      : null,
  }));
  const headers = csvHeaders(csv);
  return (
    <div className="app">
      <aside className="sidebar">
        <a className="brand" href="#">
          <span className="brand-icon">
            <Activity size={21} />
          </span>
          Conservation<span>CI</span>
        </a>
        <div className="workspace-label">
          LOCAL WORKSPACE <span className="live-dot" />
        </div>
        <a className="nav active" href="#analysis">
          <Activity size={17} />
          Trajectory analysis
        </a>
        <button className="nav" onClick={() => setShowConfig(!showConfig)}>
          <Settings2 size={17} />
          Model configuration
        </button>
        <div className="sidebar-section">EXAMPLE TRAJECTORIES</div>
        <div className="examples">
          {examples.map(([id, title, description]) => (
            <button
              disabled={busy}
              className={
                selectedExample === id && view ? "example selected" : "example"
              }
              key={id}
              onClick={() => void load(id)}
            >
              <FlaskConical size={15} />
              <span>
                {title}
                <small>{description}</small>
              </span>
              <ArrowRight size={13} />
            </button>
          ))}
        </div>
        <div className="sidebar-note">
          <ShieldCheck size={20} />
          <b>Your data stays here.</b>
          <p>
            Local analysis. No accounts, uploads to a cloud, or external
            services.
          </p>
          <code>two_mass_spring_1d</code>
        </div>
        <div className="sidebar-footer">
          CONSERVATIONCI <span>v0.1.0</span>
        </div>
      </aside>
      <main id="analysis">
        <header className="topbar">
          <span>
            Workspace <span className="slash">/</span>{" "}
            <b>Trajectory analysis</b>
          </span>
          <span className="local">
            <span className="live-dot" /> Local Rust engine
          </span>
        </header>
        <div className="content">
          <div className="page-title">
            <div className="eyebrow">SIMULATION CORRECTNESS / 01</div>
            <h1>
              Trust the laws.
              <br className="mobile-break" /> Inspect the evidence.
            </h1>
            <p>
              Independent conservation checks for your exported physics
              trajectories.
            </p>
          </div>
          <section
            className="import-bar"
            aria-label="Trajectory input"
            onDragOver={(e) => e.preventDefault()}
            onDrop={(e) => {
              e.preventDefault();
              const f = e.dataTransfer.files[0];
              if (f) void upload(f);
            }}
          >
            <div className="file-icon">
              <FileUp size={22} />
            </div>
            <div className="file-info">
              <b>{filename}</b>
              <small>CSV trajectory · t, x1, v1, x2, v2 · SI units</small>
            </div>
            <input
              ref={input}
              aria-label="CSV upload"
              type="file"
              accept=".csv,text/csv"
              hidden
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) void upload(f);
              }}
            />
            <button
              className="secondary"
              onClick={() => input.current?.click()}
            >
              Import CSV
            </button>
            <button
              className="secondary"
              onClick={() => setShowConfig(!showConfig)}
            >
              <Settings2 size={14} /> Configure <ChevronDown size={13} />
            </button>
            <button
              className="primary"
              disabled={!csv || busy}
              onClick={() => void run({ csv, config: mappedConfig() })}
            >
              {busy ? "Analyzing every sample…" : "Run analysis"}
              <ArrowRight size={15} />
            </button>
          </section>
          {showConfig && (
            <section className="config-panel">
              <div>
                <h2>Declared physical model</h2>
                <p>
                  Oriented spring: extension = x2 − x1 − L0. The regime
                  determines applicable checks.
                </p>
                <label htmlFor="config">Model configuration (TOML)</label>
                <textarea
                  id="config"
                  spellCheck={false}
                  value={config}
                  onChange={(e) => setConfig(e.target.value)}
                  rows={20}
                />
              </div>
              <div>
                <h3>CSV column mapping</h3>
                <p>
                  The backend validates all fields and rows. Extra columns are
                  ignored and reported.
                </p>
                {["time", "x1", "v1", "x2", "v2"].map((k) => (
                  <label key={k}>
                    {k}
                    <select
                      value={mapping[k] || { time: "t" }[k] || k}
                      onChange={(e) =>
                        setMapping({ ...mapping, [k]: e.target.value })
                      }
                    >
                      {headers.map((h, i) => (
                        <option key={i} value={h}>
                          {h}
                        </option>
                      ))}
                    </select>
                  </label>
                ))}
                <button
                  className="secondary"
                  disabled={busy || !csv}
                  onClick={async () => {
                    setError("");
                    try {
                      const r = await response<{ sample_count: number }>(
                        "/api/validate",
                        { csv, config: mappedConfig() },
                      );
                      setMessage(
                        `Input valid: ${r.sample_count.toLocaleString()} samples.`,
                      );
                    } catch (e) {
                      setError(String(e));
                    }
                  }}
                >
                  Validate input
                </button>
              </div>
            </section>
          )}
          {error && (
            <div role="alert" className="error">
              {error}
            </div>
          )}
          {message && (
            <p role="status" className="notice">
              {message}
            </p>
          )}
          {busy && (
            <p role="status" className="notice">
              Rust is validating the complete trajectory and evaluating all
              applicable checks.
            </p>
          )}
          {!view && !busy && (
            <section className="empty">
              <div className="empty-symbol">
                <Activity size={38} />
              </div>
              <div className="eyebrow">EVIDENCE STARTS WITH A TRAJECTORY</div>
              <h2>
                Does your simulation conserve
                <br />
                what its model says it should?
              </h2>
              <p>
                Import a CSV and declare your model, or explore a deterministic
                reference trajectory.
              </p>
              <button className="primary" onClick={() => void load("analytic")}>
                Analyze the reference fixture <ArrowRight size={16} />
              </button>
              <div className="empty-facts">
                <span>01 / Full-resolution analysis</span>
                <span>02 / Explicit tolerances</span>
                <span>03 / Traceable failures</span>
              </div>
            </section>
          )}
          {report && view && ec && mc && s && (
            <>
              <section className={`run-summary ${report.status.toLowerCase()}`}>
                <div className="summary-icon">
                  <ShieldCheck size={24} />
                </div>
                <div>
                  <div className="flex">
                    <h2>
                      {report.status === "PASS"
                        ? "Within declared tolerances"
                        : report.status === "FAIL"
                          ? "Conservation tolerance exceeded"
                          : "Diagnostic run · unverified"}
                    </h2>
                    <Badge status={report.status} />
                  </div>
                  <p>
                    {report.regime.replaceAll("_", " ")} <span>·</span>{" "}
                    {report.sample_count.toLocaleString()} samples{" "}
                    <span>·</span>{" "}
                    {fmt(report.final_time - report.initial_time)} s duration{" "}
                    <span>·</span> {report.failed_checks} failed checks
                  </p>
                </div>
                <div className="exports">
                  <span>EXPORT REPORT</span>
                  {["json", "markdown", "csv"].map((f) => (
                    <button
                      aria-label={`Download ${f} report`}
                      title={`Download ${f} report`}
                      key={f}
                      onClick={() => void download(f)}
                    >
                      <ArrowDownToLine size={13} />
                      {f === "markdown" ? "MD" : f.toUpperCase()}
                    </button>
                  ))}
                </div>
              </section>
              <div className="metrics">
                <Metric name="Mechanical energy" check={ec} />
                <Metric name="Linear momentum" check={mc} />
                <article className="metric compact">
                  <div className="flex">
                    <h3>Failure evidence</h3>
                    <span className="mini-label">FULL RESOLUTION</span>
                  </div>
                  <p className="metric-value">
                    {report.violating_samples.toLocaleString()}{" "}
                    <small>samples</small>
                  </p>
                  <p>
                    {report.first_violation
                      ? `First at t = ${fmt(report.first_violation.time)} s · CSV row ${report.first_violation.row}`
                      : "No applicable threshold crossings"}
                  </p>
                  <div className="metric-bottom">
                    <span>
                      Model <b>Two-mass spring / 1D</b>
                    </span>
                  </div>
                </article>
              </div>
              <section className="chart-panel">
                <div className="panel-heading">
                  <div>
                    <span className="eyebrow">INVARIANT DIAGNOSTICS</span>
                    <h2>
                      {tab === "phase"
                        ? "Relative phase space"
                        : tab === "energy"
                          ? "Mechanical energy"
                          : "Linear momentum"}
                    </h2>
                  </div>
                  <div
                    className="tabs"
                    role="group"
                    aria-label="Diagnostic plot"
                  >
                    {(["energy", "momentum", "phase"] as const).map((t) => (
                      <button
                        className={tab === t ? "chosen" : ""}
                        key={t}
                        onClick={() => {
                          setTab(t);
                          setMode("total");
                        }}
                      >
                        {t === "phase"
                          ? "Phase space"
                          : t[0].toUpperCase() + t.slice(1)}
                      </button>
                    ))}
                  </div>
                </div>
                <div className="chart-controls">
                  <p>
                    {tab === "phase"
                      ? "Relative displacement (m) against relative velocity (m/s)"
                      : applicable
                        ? "Initial reference and explicitly configured tolerance boundaries"
                        : tab === "energy" &&
                            monotonicity &&
                            monotonicity.status !== "NOT_APPLICABLE"
                          ? `Adjacent energy non-increase: ${monotonicity.status}. Local tolerances apply to each preceding sample; energy conservation is not assumed.`
                          : "Diagnostic only — constant-value conservation is not assumed"}
                  </p>
                  {tab !== "phase" && (
                    <select
                      aria-label="Plot value"
                      value={mode}
                      onChange={(e) => setMode(e.target.value)}
                    >
                      <option value="total">Total {tab}</option>
                      {tab === "energy" && (
                        <option value="components">Kinetic + potential</option>
                      )}
                      <option value="signed">Signed deviation</option>
                      <option value="absolute">Absolute deviation</option>
                    </select>
                  )}
                </div>
                <div
                  className="chart"
                  role="img"
                  aria-label={
                    tab === "phase"
                      ? "Relative phase-space scatter plot"
                      : `${tab} against time. Select a point to inspect its source sample. Use the sample slider for keyboard selection.`
                  }
                >
                  {tab === "phase" ? (
                    <ResponsiveContainer width="100%" height="100%">
                      <ScatterChart
                        margin={{ top: 20, right: 25, bottom: 25, left: 10 }}
                      >
                        <CartesianGrid strokeDasharray="3 5" />
                        <XAxis
                          type="number"
                          dataKey="relative_displacement"
                          name="Relative displacement"
                          unit=" m"
                        />
                        <YAxis
                          type="number"
                          dataKey="relative_velocity"
                          name="Relative velocity"
                          unit=" m/s"
                        />
                        <Tooltip />
                        <Scatter
                          data={view.samples}
                          fill="#538678"
                          isAnimationActive={false}
                        />
                      </ScatterChart>
                    </ResponsiveContainer>
                  ) : (
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart
                        data={chartData}
                        margin={{ top: 20, right: 25, bottom: 10, left: 15 }}
                        onClick={(state) => {
                          if (state.activeLabel != null)
                            selectTime(Number(state.activeLabel));
                        }}
                      >
                        <CartesianGrid
                          vertical={false}
                          strokeDasharray="3 5"
                          stroke="#e5e8e2"
                        />
                        <XAxis
                          dataKey="t"
                          type="number"
                          domain={["dataMin", "dataMax"]}
                          tickFormatter={(v) => `${fmt(Number(v))}s`}
                          minTickGap={50}
                        />
                        <YAxis
                          domain={["auto", "auto"]}
                          tickFormatter={(v) => Number(v).toExponential(2)}
                          width={80}
                        />
                        <Tooltip
                          formatter={(v) => fmt(Number(v))}
                          labelFormatter={(v) => `t = ${fmt(Number(v))} s`}
                        />
                        {mode === "components" ? (
                          <>
                            <Line
                              dataKey="kinetic"
                              stroke="#b78648"
                              dot={false}
                              isAnimationActive={false}
                            />
                            <Line
                              dataKey="potential"
                              stroke="#6083a4"
                              dot={false}
                              isAnimationActive={false}
                            />
                          </>
                        ) : (
                          <>
                            <Line
                              dataKey={key}
                              name={`${tab} (${tab === "energy" ? "J" : "kg m/s"})`}
                              stroke="#347361"
                              strokeWidth={2}
                              dot={false}
                              isAnimationActive={false}
                            />
                            <Line
                              dataKey="marker"
                              name="Tolerance exceeded"
                              stroke="none"
                              dot={{ r: 2, fill: "#c25236", stroke: "none" }}
                              isAnimationActive={false}
                            />
                            {applicable && check?.allowed_error != null && (
                              <>
                                <ReferenceLine
                                  y={baseline}
                                  stroke="#95a09a"
                                  strokeDasharray="4 4"
                                />
                                <ReferenceLine
                                  y={(baseline || 0) + check.allowed_error}
                                  stroke="#c0925c"
                                  strokeDasharray="5 5"
                                />
                                {mode !== "absolute" && (
                                  <ReferenceLine
                                    y={(baseline || 0) - check.allowed_error}
                                    stroke="#c0925c"
                                    strokeDasharray="5 5"
                                  />
                                )}
                              </>
                            )}
                          </>
                        )}
                        <ReferenceLine
                          x={s.t}
                          stroke="#263e35"
                          label={{
                            value: "Selected",
                            position: "insideTopRight",
                            fontSize: 10,
                          }}
                        />
                        <Brush
                          dataKey="t"
                          height={22}
                          stroke="#9caea4"
                          travellerWidth={8}
                          tickFormatter={(v) => `${fmt(Number(v))}s`}
                        />
                      </LineChart>
                    </ResponsiveContainer>
                  )}
                </div>
                <div className="chart-foot">
                  <span>
                    <i /> Rust-calculated samples{" "}
                    <span className="legend-band">
                      --- Tolerance where applicable
                    </span>
                  </span>
                  <span>
                    {view.samples.length.toLocaleString()} plotted /{" "}
                    {view.full_sample_count.toLocaleString()} analyzed · extrema
                    + first 1,000 crossing boundaries
                  </span>
                </div>
              </section>
              <div className="bottom-grid">
                <section className="trajectory-panel">
                  <div className="panel-heading">
                    <div>
                      <span className="eyebrow">TRAJECTORY INSPECTOR</span>
                      <h2>Two bodies. One spring.</h2>
                    </div>
                    <code>t = {fmt(s.t)} s</code>
                  </div>
                  <Trajectory sample={s} samples={view.samples} />
                  <div className="scrubber">
                    <button
                      aria-label={
                        playing ? "Pause trajectory" : "Play trajectory"
                      }
                      onClick={() => {
                        if (
                          !playing &&
                          matchMedia("(prefers-reduced-motion: reduce)").matches
                        ) {
                          setMessage(
                            "Reduced motion is enabled. Use the sample slider to inspect the trajectory.",
                          );
                          return;
                        }
                        setExactSample(null);
                        setPlaying(!playing);
                      }}
                    >
                      {playing ? <Pause size={15} /> : <Play size={15} />}
                    </button>
                    <input
                      aria-label="Selected sample"
                      type="range"
                      min={0}
                      max={view.samples.length - 1}
                      value={selected}
                      onChange={(e) => {
                        setSelected(Number(e.target.value));
                        setExactSample(null);
                        setPlaying(false);
                      }}
                    />
                    <span>row {s.row}</span>
                  </div>
                  <table>
                    <caption>
                      Selected source state and invariant values
                    </caption>
                    <thead>
                      <tr>
                        <th>Quantity</th>
                        <th>Mass 1</th>
                        <th>Mass 2</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr>
                        <th>Position / m</th>
                        <td>{fmt(s.x1)}</td>
                        <td>{fmt(s.x2)}</td>
                      </tr>
                      <tr>
                        <th>Velocity / m/s</th>
                        <td>{fmt(s.v1)}</td>
                        <td>{fmt(s.v2)}</td>
                      </tr>
                      <tr>
                        <th>Energy / J</th>
                        <td colSpan={2}>{fmt(s.energy)}</td>
                      </tr>
                      <tr>
                        <th>Momentum / kg m/s</th>
                        <td colSpan={2}>{fmt(s.momentum)}</td>
                      </tr>
                      <tr>
                        <th>Allowed energy error / J</th>
                        <td colSpan={2}>{fmt(s.energy_allowed)}</td>
                      </tr>
                      <tr>
                        <th>Allowed momentum error / kg m/s</th>
                        <td colSpan={2}>{fmt(s.momentum_allowed)}</td>
                      </tr>
                    </tbody>
                  </table>
                  <button
                    className="copy"
                    onClick={() =>
                      void navigator.clipboard
                        .writeText(JSON.stringify(s, null, 2))
                        .then(() => setMessage("Selected sample copied."))
                        .catch(() =>
                          setError(
                            "Clipboard unavailable. Download the full-sample CSV.",
                          ),
                        )
                    }
                  >
                    Copy selected sample diagnostics
                  </button>
                </section>
                <section className="violations-panel">
                  <div className="panel-heading">
                    <div>
                      <span className="eyebrow">THRESHOLD CROSSINGS</span>
                      <h2>
                        Failure timeline <small>{report.violation_count}</small>
                      </h2>
                    </div>
                  </div>
                  {report.violations.length === 0 ? (
                    <div className="no-failures">
                      <CheckCircle2 size={27} />
                      <h3>
                        {report.status === "UNVERIFIED"
                          ? "No conservation checks applied"
                          : "No violations detected"}
                      </h3>
                      <p>
                        {report.status === "UNVERIFIED"
                          ? "No work or impulse budget was supplied. This run is diagnostic only."
                          : "Every exported sample remained within its applicable tolerance."}
                      </p>
                    </div>
                  ) : (
                    <div className="violation-list">
                      {report.violations.slice(0, 100).map((v, i) => (
                        <button
                          key={i}
                          className="violation"
                          onClick={async () => {
                            selectTime(v.time);
                            setTab(
                              v.check === "momentum" ? "momentum" : "energy",
                            );
                            if (active.current) {
                              try {
                                setExactSample(
                                  await response<Sample>(
                                    `/api/samples/${v.index}`,
                                    active.current,
                                  ),
                                );
                              } catch (e) {
                                setError(String(e));
                              }
                            }
                          }}
                        >
                          <div className="flex">
                            <b>{v.check.replaceAll("_", " ")}</b>
                            <code>t = {fmt(v.time)} s</code>
                          </div>
                          <p>
                            CSV row {v.row} · deviation {fmt(v.deviation)} ·
                            allowed {fmt(v.allowed)}
                          </p>
                          <span>
                            Observed {fmt(v.observed)} / reference{" "}
                            {fmt(v.reference)} <ArrowRight size={13} />
                          </span>
                        </button>
                      ))}
                      {report.violation_count > 100 && (
                        <p>
                          Showing the first 100 events. JSON includes up to
                          1,000; CSV exports every sample.
                        </p>
                      )}
                    </div>
                  )}
                </section>
              </div>
              <section className="interpretation">
                <ShieldCheck size={20} />
                <div>
                  <h3>What this evidence tells you</h3>
                  {report.interpretation.map((p) => (
                    <p key={p}>{p}</p>
                  ))}
                  {report.ignored_columns.length > 0 && (
                    <p>
                      Ignored CSV columns: {report.ignored_columns.join(", ")}
                    </p>
                  )}
                  <details>
                    <summary>Numerical diagnostics and reproducibility</summary>
                    <table>
                      <thead>
                        <tr>
                          <th>Check</th>
                          <th>Outcome</th>
                          <th>RMS</th>
                          <th>Early RMS</th>
                          <th>Late RMS</th>
                          <th>Signed OLS slope / s</th>
                          <th>Longest violation streak</th>
                        </tr>
                      </thead>
                      <tbody>
                        {Object.entries(report.checks).map(([name, c]) => (
                          <tr key={name}>
                            <th>{name}</th>
                            <td>
                              <Badge status={c.status} />
                            </td>
                            <td>{fmt(c.rms_error)}</td>
                            <td>{fmt(c.early_rms)}</td>
                            <td>{fmt(c.late_rms)}</td>
                            <td>{fmt(c.signed_slope)}</td>
                            <td>{c.longest_violation_streak}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                    <p>
                      OLS slope describes signed error, and does not establish
                      monotonic drift or identify an integrator.
                    </p>
                    <code>
                      Input SHA-256: {report.input_sha256}
                      <br />
                      Config SHA-256: {report.config_sha256}
                    </code>
                  </details>
                </div>
              </section>
            </>
          )}
          <footer>
            Catch broken physics before it ships.
            <span>
              Independent analysis. Explicit assumptions. Reproducible evidence.
            </span>
          </footer>
        </div>
      </main>
    </div>
  );
}
function Trajectory({
  sample: s,
  samples,
}: {
  sample: Sample;
  samples: Sample[];
}) {
  let min = Infinity,
    max = -Infinity;
  for (const p of samples) {
    min = Math.min(min, p.x1, p.x2);
    max = Math.max(max, p.x1, p.x2);
  }
  const scale = (x: number) => 45 + ((x - min) / (max - min || 1)) * 510;
  const a = scale(s.x1),
    b = scale(s.x2);
  const points = Array.from(
    { length: 33 },
    (_, i) =>
      `${a + ((b - a) * i) / 32},${i === 0 || i === 32 ? 78 : 78 + (i % 2 ? 9 : -9)}`,
  ).join(" ");
  return (
    <svg
      className="spring-view"
      viewBox="0 0 600 150"
      role="img"
      aria-label={`Mass 1 at ${fmt(s.x1)} meters, mass 2 at ${fmt(s.x2)} meters`}
    >
      <line x1="25" y1="115" x2="575" y2="115" stroke="#dce3db" />
      {Array.from({ length: 12 }, (_, i) => (
        <line
          key={i}
          x1={25 + i * 50}
          y1="110"
          x2={25 + i * 50}
          y2="120"
          stroke="#dce3db"
        />
      ))}
      <polyline points={points} fill="none" stroke="#82958c" strokeWidth="2" />
      <rect x={a - 16} y="62" width="32" height="32" rx="7" fill="#347361" />
      <rect x={b - 21} y="57" width="42" height="42" rx="8" fill="#b5864b" />
      <text x={a} y="47" textAnchor="middle" fill="#347361">
        m₁
      </text>
      <text x={b} y="42" textAnchor="middle" fill="#986c36">
        m₂
      </text>
      <text x="25" y="141" fill="#738177" fontSize="11">
        {fmt(min)} m
      </text>
      <text x="575" y="141" textAnchor="end" fill="#738177" fontSize="11">
        {fmt(max)} m
      </text>
    </svg>
  );
}
