# zkid-crescent-lab

A minimal, reproducible lab around **zkID-style anonymous credentials** using **Microsoft's Crescent Credentials**.

This repo is intentionally small: it does _not_ re-implement credential schemes. Instead, it:

- vendors (clones) Crescent,
- runs Crescent's `zksetup / prove / show / verify` CLI steps, and
- records timings + artifacts so you can write a clean blog post with real data.

## Prereqs

- Rust toolchain (`rustup`)
- git + bash
- Crescent's circuit setup deps (see Crescent repo docs)

## Quickstart

### 1) Clone this repo and vendor Crescent

```bash
git clone https://github.com/reymom/zkid-crescent-lab.git
cd zkid-crescent-lab
./scripts/vendor_crescent.sh
```

This clones Crescent into `vendor/crescent-credentials`.

### 2) Run Crescent circuit setup

Pick a parameter set (recommended: `rs256-sd` first):

```bash
cd vendor/crescent-credentials/circuit_setup/scripts
./run_setup.sh rs256-sd
```

This should populate `vendor/crescent-credentials/creds/test-vectors/rs256-sd/`.

### 3) Sanity check Crescent

```bash
cd vendor/crescent-credentials/creds
cargo test --release
```

### 4) Run the lab harness (timings + logs)

From the root of _this_ repo:

```bash
cargo run --release -- run-all --param rs256-sd --show-iters 20
```

Outputs:

- `out/<run_id>/results.csv`
- `out/<run_id>/meta.json`
- `out/<run_id>/logs/<step>_<iter>_{stdout,stderr}.log`

## License

MIT for the harness code (this repo). Crescent code remains under its own license.
