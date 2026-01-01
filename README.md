# zkid-crescent-lab

A small, reproducible benchmark harness for **Crescent Credentials** (Microsoft) — measuring the cost of generating and verifying **ZK presentations** from _real credential formats_ (JWT + mDL).

This repo does **not** re-implement Crescent. Instead, it:

- vendors `microsoft/crescent-credentials` under `vendor/`,
- runs Crescent's CLI steps (`zksetup`, `prove`, `show`, `verify`) in a consistent way,
- records timings + (optional) per-iteration samples + proof artifact sizes,
- writes clean CSV outputs and generates plots for a technical write-up.

## What you get

After a run you’ll have:

```text
out/run_<timestamp>Z/
  meta.json
  results.csv          # aggregated per step (p50/p95/mean etc.)
  samples.csv          # optional per-iteration rows (show/verify)
  param_table.md       # per-param metadata (claims, circuit IO, credential type, sig)
  param_table.csv
```

And after plotting:

```text
out/plots/
  merged_results.csv
  *.png
```

## Parameters used in this lab

This lab is set up around the Crescent parameter names:

- `rs256`: RSA-SHA256 JWT (hardcoded disclosure variant in Crescent)
- `rs256-sd`: RSA-SHA256 JWT (selective disclosure)
- `rs256-db`: device-bound RSA-SHA256 JWT (selective disclosure)
- `mdl1`: device-bound mDL (ECDSA-style, selective disclosure)

(These names come from Crescent's own test vectors / circuit setup flow.)

## Prerequisites

- Linux + bash + git
- Rust toolchain (pinned by `rust-toolchain.toml`)
- Python 3 (used only for setup scripts + plotting)

## Quickstart (end-to-end)

From repo root:

### 1) Vendor Crescent

```bash
./scripts/vendor_crescent.sh
```

This clones Crescent into `vendor/crescent-credentials/`.

### 2) Install Crescent build prerequisites

```bash
./scripts/setup_crescent.sh
```

(Optional sanity check: Crescent CLI builds)

```bash
cd vendor/crescent-credentials/creds
cargo build --release --bin crescent
ls -la target/release/crescent
```

### 3) Setup Python (project scripts)

This creates `.venv/` for any helper tooling used by the setup scripts.

```bash
./scripts/setup_python.sh
```

### 4) Generate / prepare test vectors

This runs Crescent’s circuit setup + test-vector generation for the parameters this lab benchmarks.

```bash
./scripts/setup_vectors.sh
```

### 5) Sanity test

```bash
./scripts/test_crescent.sh
```

## Running benchmarks

### Run a single end-to-end pass (one of each step)

```bash
cargo run --release -- run --param rs256-sd
```

### Benchmark `show` + `verify` N times (recommended)

This runs:

- `zksetup` once
- `prove` once
- `show` N times
- `verify` N times

Example:

```bash
cargo run --release -- bench --param rs256 --iters 30
cargo run --release -- bench --param rs256-sd --iters 30
cargo run --release -- bench --param rs256-db --iters 30
cargo run --release -- bench --param mdl1 --iters 30
```

### Outputs

Each command writes a new `out/run_<timestamp>Z/` folder with:

- `results.csv` (aggregated)
- `samples.csv` (raw per-iteration rows when enabled)
- `param_table.md` / `param_table.csv` (per-param metadata)
- failure logs (stdout/stderr) when a step fails (or when forced)

## Proof size measurement

After each successful `show`, the harness attempts to locate the newly produced presentation/proof artifact under the relevant Crescent `test-vectors/<param>/` directory (using a "new file since step start" heuristic and preferring filenames containing `presentation` or `proof`).

The artifact size (bytes) is recorded per iteration and aggregated into `results.csv`.

## Plotting (separate venv)

Plotting is intentionally isolated in `.venv-tools/` so plotting deps don't leak into setup tooling.

```bash
python3 -m venv .venv-tools
source .venv-tools/bin/activate
pip install -r tools/requirements-tools.txt
python3 tools/plot_suite.py --out out
```

This produces plots and a merged CSV under `out/plots/`.

## References

- Crescent Credentials repo:
  https://github.com/microsoft/crescent-credentials
- Crescent circuit setup docs:
  https://github.com/microsoft/crescent-credentials/blob/main/circuit_setup/README.md
- Paper: “Crescent: Stronger Privacy for Existing Credentials” (ePrint 2024/2013):
  https://eprint.iacr.org/2024/2013.pdf
- Christian Paquin write-up:
  https://christianpaquin.github.io/2024-12-19-crescent-creds.html

## License

- The benchmarking harness in this repo: MIT (see `LICENSE`)
- Vendored Crescent code remains under its own license in `vendor/crescent-credentials/`
