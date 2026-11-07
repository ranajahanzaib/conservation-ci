# Architecture and local API

The Cargo workspace separates `conservationci-core` from the adapters. Core contains strict config/CSV validation, f64 invariant evaluation, check applicability, diagnostics, deterministic reporting, SHA-256 provenance, and fixture generation. It depends on neither Tokio nor Axum. The CLI controls I/O and exit codes; the server calls the same `analyze` function. React consumes serialized values; it does no authoritative physics or check classification.

The analyzer evaluates all n samples and uses O(n) memory bounded by 250,000 samples and 32 MiB input. Limits are compile-time public constants in the core. The JSON envelope is 33 MiB and requests are limited to two concurrent operations. Compute is offloaded to blocking worker tasks. There is no persistent storage, path argument in browser requests, shell execution, authentication, or external service.

| Method / route                        | Contract                                                                   |
| ------------------------------------- | -------------------------------------------------------------------------- |
| GET /api/health                       | `{ "status": "ok" }`                                                       |
| GET /api/examples                     | Fixed safe example IDs                                                     |
| GET /api/examples/{id}                | Embedded generated CSV and TOML text                                       |
| POST /api/analyze                     | `{ "csv": "...", "config": "TOML..." }`; returns report + plotting samples |
| POST /api/validate                    | Same body; validation only, returns sample count and ignored columns       |
| POST /api/reports/{json,markdown,csv} | Same body; canonical JSON, Markdown, or full-sample CSV                    |
| POST /api/samples/{index}             | Same body; exact full-resolution sample at zero-based index                |

Invalid config/input and malformed JSON return structured `{ "error": "..." }` with HTTP 422; oversized JSON returns 413; concurrent-request saturation returns 429. Unknown routes use the static service. API operations are stateless and re-analyze the submitted data, so downloads and exact sample lookups do not depend on mutable session storage. This trades some repeated CPU work for a small architecture.

Charts use up to 400 sample-count buckets, keeping extrema of energy, momentum, kinetic and potential energy plus first/last samples and first 1,000 threshold boundaries. Actual times are preserved. Every crossing still contributes to the full report and CSV. This bounds chart work even for alternating-threshold data; later crossings can be omitted visually, so the UI discloses that limit. No smoothing or downsampled calculation affects check status. Phase-space and playback share this presentation subset; selected failure events can request exact source samples separately.

The local server binds IPv4 loopback only. Vite's development proxy keeps requests same-origin. Production uses the built static frontend directory passed to `serve --web`. No cloud hosting or remote deployment is part of this project.
