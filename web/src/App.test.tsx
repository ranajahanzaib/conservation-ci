// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import App from "./App";
vi.mock("recharts", () => {
  const Empty = () => null;
  const Box = ({ children }: { children?: import("react").ReactNode }) => (
    <div>{children}</div>
  );
  return {
    ResponsiveContainer: Box,
    LineChart: Box,
    ScatterChart: Box,
    CartesianGrid: Empty,
    Line: Empty,
    ReferenceLine: Empty,
    Tooltip: Empty,
    XAxis: Empty,
    YAxis: Empty,
    Brush: Empty,
    Scatter: Empty,
  };
});
const check = {
  status: "PASS",
  reason: "Declared model",
  unit: "J",
  initial: 0.15,
  final_value: 0.15,
  allowed_error: 0.001501,
  max_absolute_error: 1e-15,
  maximum_time: 0,
  max_normalized_error: 0.00001,
  zero_tolerance_exceeded: false,
  rms_error: 1e-16,
  final_deviation: 0,
  signed_slope: 0,
  early_rms: 0,
  late_rms: 0,
  first_violation_index: null,
  first_violation_time: null,
  violating_samples: 0,
  longest_violation_streak: 0,
};
const sample = {
  row: 2,
  t: 0,
  x1: 0,
  v1: 0.4,
  x2: 1.2,
  v2: -0.1,
  kinetic: 0.09,
  potential: 0.06,
  energy: 0.15,
  momentum: 0.2,
  energy_deviation: 0,
  momentum_deviation: 0,
  energy_allowed: 0.001501,
  momentum_allowed: 1e-8,
  energy_violation: false,
  momentum_violation: false,
};
let status = "PASS",
  fail = false;
function view() {
  return {
    report: {
      status,
      regime:
        status === "UNVERIFIED" ? "externally_driven" : "closed_conservative",
      sample_count: 2,
      initial_time: 0,
      final_time: 1,
      checks: {
        energy: {
          ...check,
          status: status === "UNVERIFIED" ? "NOT_APPLICABLE" : status,
        },
        momentum: {
          ...check,
          unit: "kg m/s",
          status: status === "UNVERIFIED" ? "NOT_APPLICABLE" : "PASS",
        },
      },
      failed_checks: status === "FAIL" ? 1 : 0,
      violating_samples: status === "FAIL" ? 1 : 0,
      violations:
        status === "FAIL"
          ? [
              {
                check: "energy",
                index: 1,
                row: 3,
                time: 1,
                observed: 0.5,
                reference: 0.15,
                deviation: 0.35,
                allowed: 0.001501,
              },
            ]
          : [],
      violation_count: status === "FAIL" ? 1 : 0,
      first_violation: null,
      input_sha256: "a".repeat(64),
      config_sha256: "b".repeat(64),
      ignored_columns: [],
      interpretation: ["Conservation does not prove physical correctness."],
    },
    samples: [sample, { ...sample, row: 3, t: 1, x1: 0.2, x2: 1.4 }],
    indices: [0, 1],
    full_sample_count: 2,
    markdown: "# Report",
  };
}
const fetchMock = vi.fn(async (url: string) => ({
  ok: !fail,
  text: async () =>
    JSON.stringify({ error: "row 3: timestamps must strictly increase" }),
  json: async () =>
    url.startsWith("/api/examples/")
      ? {
          csv: "t,x1,v1,x2,v2\n0,0,0.4,1.2,-0.1\n1,0.2,0.4,1.4,-0.1",
          config: "schema_version = 1",
        }
      : url === "/api/validate"
        ? { sample_count: 2 }
        : url.startsWith("/api/samples/")
          ? { ...sample, row: 3, t: 1 }
          : view(),
  blob: async () => new Blob(["report"]),
}));
beforeEach(() => {
  status = "PASS";
  fail = false;
  fetchMock.mockClear();
  vi.stubGlobal("fetch", fetchMock);
  vi.stubGlobal("matchMedia", () => ({ matches: false }));
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});
async function load() {
  await userEvent.click(
    screen.getByRole("button", { name: /Analyze the reference fixture/ }),
  );
  await screen.findByText("Within declared tolerances");
}
it("has an accessible empty state and no automatic requests", () => {
  render(<App />);
  expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent(
    "Trust the laws",
  );
  expect(screen.getByRole("button", { name: "Run analysis" })).toBeDisabled();
  expect(fetchMock).not.toHaveBeenCalled();
});
it("loads an example and analyzes through the API", async () => {
  render(<App />);
  await load();
  expect(fetchMock.mock.calls.map((c) => c[0])).toEqual([
    "/api/examples/analytic",
    "/api/analyze",
  ]);
  expect(
    screen.getByRole("button", { name: "Download json report" }),
  ).toBeEnabled();
  expect(screen.getByText("No violations detected")).toBeInTheDocument();
});
it("edits configuration and validates input", async () => {
  render(<App />);
  await load();
  await userEvent.click(screen.getByRole("button", { name: /^Configure/ }));
  fireEvent.change(screen.getByLabelText("Model configuration (TOML)"), {
    target: { value: "schema_version = 2" },
  });
  expect(screen.getByLabelText("Model configuration (TOML)")).toHaveValue(
    "schema_version = 2",
  );
  await userEvent.click(screen.getByRole("button", { name: "Validate input" }));
  expect(await screen.findByRole("status")).toHaveTextContent(
    "Input valid: 2 samples.",
  );
});
it("displays validation errors and recovers", async () => {
  render(<App />);
  await load();
  fail = true;
  await userEvent.click(screen.getByRole("button", { name: "Run analysis" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "row 3: timestamps must strictly increase",
  );
  fail = false;
  await userEvent.click(screen.getByRole("button", { name: "Run analysis" }));
  await waitFor(() =>
    expect(screen.queryByRole("alert")).not.toBeInTheDocument(),
  );
});
it("distinguishes an unverified run", async () => {
  status = "UNVERIFIED";
  render(<App />);
  await userEvent.click(
    screen.getByRole("button", { name: /External forcing/ }),
  );
  expect(
    await screen.findByText("Diagnostic run · unverified"),
  ).toBeInTheDocument();
  expect(
    screen.getByText("No conservation checks applied"),
  ).toBeInTheDocument();
  expect(screen.getAllByText("NOT APPLICABLE").length).toBeGreaterThanOrEqual(
    2,
  );
});
it("synchronizes failure selection with exact source sample", async () => {
  status = "FAIL";
  render(<App />);
  await userEvent.click(screen.getByRole("button", { name: /Explicit Euler/ }));
  await screen.findByText("Conservation tolerance exceeded");
  await userEvent.click(screen.getByRole("button", { name: /Observed 0.5/ }));
  await waitFor(() => expect(screen.getByText("row 3")).toBeInTheDocument());
  expect(fetchMock).toHaveBeenCalledWith("/api/samples/1", expect.anything());
});
it("switches plot modes and slider selection", async () => {
  render(<App />);
  await load();
  await userEvent.click(screen.getByRole("button", { name: "Momentum" }));
  expect(
    screen.getByRole("combobox", { name: "Plot value" }),
  ).toBeInTheDocument();
  fireEvent.change(screen.getByRole("slider", { name: "Selected sample" }), {
    target: { value: "1" },
  });
  expect(screen.getByText("row 3")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Phase space" }));
  expect(
    screen.getByRole("img", { name: "Relative phase-space scatter plot" }),
  ).toBeInTheDocument();
});
it("uploads CSV and opens configuration", async () => {
  render(<App />);
  const file = new File(["t,x1,v1,x2,v2\n0,0,0,1,0\n1,0,0,1,0"], "test.csv", {
    type: "text/csv",
  });
  Object.defineProperty(file, "text", {
    value: async () => "t,x1,v1,x2,v2\n0,0,0,1,0\n1,0,0,1,0",
  });
  await userEvent.upload(screen.getByLabelText("CSV upload"), file);
  expect(await screen.findByText("test.csv")).toBeInTheDocument();
  expect(
    screen.getByLabelText("Model configuration (TOML)"),
  ).toBeInTheDocument();
});
it("downloads backend reports", async () => {
  vi.stubGlobal("URL", {
    createObjectURL: vi.fn(() => "blob:test"),
    revokeObjectURL: vi.fn(),
  });
  const click = vi
    .spyOn(HTMLAnchorElement.prototype, "click")
    .mockImplementation(() => {});
  render(<App />);
  await load();
  await userEvent.click(
    screen.getByRole("button", { name: "Download csv report" }),
  );
  await waitFor(() => expect(click).toHaveBeenCalled());
  expect(fetchMock).toHaveBeenCalledWith("/api/reports/csv", expect.anything());
  click.mockRestore();
});
it("announces loading while a request is pending", async () => {
  fetchMock.mockImplementationOnce(() => new Promise(() => {}));
  render(<App />);
  await userEvent.click(
    screen.getByRole("button", { name: /Analyze the reference fixture/ }),
  );
  expect(screen.getByRole("status")).toHaveTextContent("Rust is validating");
  expect(
    screen.getByRole("button", { name: /Analyzing every sample/ }),
  ).toBeDisabled();
});
