"""
Purpose: Generate meaningful benchmark plots from out/run_*/results.csv and samples.csv.

Design:
- Treat zksetup/prove as one-time "offline" costs (often n=1 in bench mode).
- Treat show/verify as "online" costs with real distributions (n=iters).
- Produce boxplots for show/verify and proof size, plus tradeoff + amortization plots.

Usage:
  python3 tools/plot_suite.py --out out
"""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import Dict, List, Tuple

import pandas as pd
import matplotlib.pyplot as plt


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", type=str, default="out", help="Output directory containing run_* folders")
    ap.add_argument("--mode", choices=["latest-per-param", "all-runs"], default="latest-per-param")
    return ap.parse_args()


def find_run_dirs(out_dir: Path) -> List[Path]:
    return sorted([p for p in out_dir.glob("run_*") if p.is_dir()])


def read_csv_if_exists(path: Path) -> pd.DataFrame | None:
    if not path.is_file():
        return None
    return pd.read_csv(path)


def run_id_from_dir(run_dir: Path) -> str:
    return run_dir.name


def choose_latest_run_per_param(results_all: pd.DataFrame) -> Dict[str, str]:
    """
    Return mapping param -> run_id (dir name), picking lexicographically max run_id.
    Your run_id format run_YYYYMMDDTHHMMSSZ sorts correctly.
    """
    latest: Dict[str, str] = {}
    for param, g in results_all.groupby("param"):
        rid = sorted(g["run_id"].unique())[-1]
        latest[param] = rid
    return latest


def ensure_plots_dir(out_dir: Path) -> Path:
    p = out_dir / "plots"
    p.mkdir(parents=True, exist_ok=True)
    return p


def boxplot_by_param(
    df: pd.DataFrame,
    value_col: str,
    title: str,
    ylabel: str,
    out_path: Path,
) -> None:
    # Keep params in deterministic order
    params = sorted(df["param"].unique())
    data = [df[df["param"] == p][value_col].dropna().tolist() for p in params]

    plt.figure()
    plt.boxplot(data, labels=params, showfliers=True)
    plt.title(title)
    plt.ylabel(ylabel)
    plt.xticks(rotation=90)
    plt.tight_layout()
    plt.savefig(out_path)
    plt.close()


def scatter_show_vs_proof(df_show: pd.DataFrame, out_path: Path) -> None:
    plt.figure()
    for p in sorted(df_show["param"].unique()):
        g = df_show[df_show["param"] == p]
        # artifact_bytes may be missing for some iterations; drop them
        gg = g.dropna(subset=["artifact_bytes"])
        if gg.empty:
            continue
        plt.scatter(gg["artifact_bytes"], gg["duration_ms"], label=p, alpha=0.8)
    plt.title("show latency vs proof size (per-iteration)")
    plt.xlabel("proof size (bytes)")
    plt.ylabel("show duration (ms)")
    plt.legend()
    plt.tight_layout()
    plt.savefig(out_path)
    plt.close()


def amortized_cost_curve(results_latest: pd.DataFrame, out_path: Path) -> None:
    """
    Plot amortized end-to-end cost per presentation:
      amortized(n) = online_p50 + offline / n
    where offline = zksetup_mean + prove_mean (typically n=1),
          online_p50 = show_p50 + verify_p50 (n=iters).
    """
    # N values (log-ish)
    ns = [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000]

    plt.figure()
    for param in sorted(results_latest["param"].unique()):
        g = results_latest[results_latest["param"] == param]

        def get(step: str, col: str) -> float | None:
            gg = g[g["step"] == step]
            if gg.empty:
                return None
            v = gg.iloc[0][col]
            try:
                return float(v)
            except Exception:
                return None

        zk = get("zksetup", "mean_ms")
        pr = get("prove", "mean_ms")
        sh = get("show", "p50_ms")
        ve = get("verify", "p50_ms")
        if None in (zk, pr, sh, ve):
            continue

        offline = zk + pr
        online = sh + ve
        ys = [online + offline / n for n in ns]
        plt.plot(ns, ys, marker="o", label=param)

    plt.xscale("log")
    plt.title("Amortized cost per presentation: (show+verify)p50 + (zksetup+prove)/n")
    plt.xlabel("n presentations (log scale)")
    plt.ylabel("amortized ms / presentation")
    plt.legend()
    plt.tight_layout()
    plt.savefig(out_path)
    plt.close()


def offline_online_breakdown(results_latest: pd.DataFrame, out_path: Path) -> None:
    """
    Bar chart: offline (zksetup+prove mean) vs online (show+verify mean).
    """
    params = sorted(results_latest["param"].unique())

    offline_vals: List[float] = []
    online_vals: List[float] = []

    for p in params:
        g = results_latest[results_latest["param"] == p]

        def get(step: str, col: str) -> float:
            gg = g[g["step"] == step]
            if gg.empty:
                return 0.0
            return float(gg.iloc[0][col])

        offline = get("zksetup", "mean_ms") + get("prove", "mean_ms")
        online = get("show", "mean_ms") + get("verify", "mean_ms")
        offline_vals.append(offline)
        online_vals.append(online)

    x = list(range(len(params)))

    plt.figure()
    plt.bar(x, offline_vals, label="offline (zksetup+prove mean)")
    plt.bar(x, online_vals, bottom=offline_vals, label="online (show+verify mean)")
    plt.title("Offline vs online cost breakdown (latest run per param)")
    plt.ylabel("ms")
    plt.xticks(x, params, rotation=90)
    plt.legend()
    plt.tight_layout()
    plt.savefig(out_path)
    plt.close()


def main() -> None:
    args = parse_args()
    out_dir = Path(args.out)
    plots_dir = ensure_plots_dir(out_dir)

    run_dirs = find_run_dirs(out_dir)
    if not run_dirs:
        raise SystemExit(f"No run_* folders found under: {out_dir}")

    results_frames = []
    samples_frames = []

    for rd in run_dirs:
        results = read_csv_if_exists(rd / "results.csv")
        if results is None:
            continue
        results["run_id"] = results.get("run_id", run_id_from_dir(rd))
        results["run_dir"] = str(rd)
        results_frames.append(results)

        samples = read_csv_if_exists(rd / "samples.csv")
        if samples is not None:
            samples["run_id"] = samples.get("run_id", run_id_from_dir(rd))
            samples["run_dir"] = str(rd)
            samples_frames.append(samples)

    if not results_frames:
        raise SystemExit("No results.csv found under any run_* folder")

    results_all = pd.concat(results_frames, ignore_index=True)
    samples_all = pd.concat(samples_frames, ignore_index=True) if samples_frames else pd.DataFrame()

    if args.mode == "latest-per-param":
        latest = choose_latest_run_per_param(results_all)
        keep_run_ids = set(latest.values())

        results_latest = results_all[results_all["run_id"].isin(keep_run_ids)].copy()
        # Keep only the chosen run per param
        results_latest = results_latest.sort_values(["param", "run_id"]).groupby(["param", "step"]).tail(1)

        if not samples_all.empty:
            samples_latest = samples_all[samples_all["run_id"].isin(keep_run_ids)].copy()
            samples_latest = samples_latest.sort_values(["param", "run_id"]).groupby(["param"]).apply(
                lambda g: g[g["run_id"] == sorted(g["run_id"].unique())[-1]]
            ).reset_index(drop=True)
        else:
            samples_latest = pd.DataFrame()
    else:
        results_latest = results_all.copy()
        samples_latest = samples_all.copy()

    # Write merged CSVs for blog/debugging
    (plots_dir / "merged_results.csv").write_text(results_latest.to_csv(index=False))
    if not samples_latest.empty:
        (plots_dir / "merged_samples.csv").write_text(samples_latest.to_csv(index=False))

    # Only plot distributions where we have real samples
    if samples_latest.empty:
        raise SystemExit("No samples.csv found. Re-run with samples enabled (your runs already have it).")

    # Normalize column names we rely on
    # samples.csv is expected to contain: param, step, duration_ms, artifact_bytes (optional)
    for col in ["duration_ms", "artifact_bytes"]:
        if col not in samples_latest.columns:
            samples_latest[col] = pd.NA

    df_show = samples_latest[samples_latest["step"] == "show"].copy()
    df_verify = samples_latest[samples_latest["step"] == "verify"].copy()

    # 1) Boxplots: show / verify
    boxplot_by_param(
        df_show,
        value_col="duration_ms",
        title="show latency distribution (per-iteration)",
        ylabel="ms",
        out_path=plots_dir / "show_boxplot.png",
    )

    boxplot_by_param(
        df_verify,
        value_col="duration_ms",
        title="verify latency distribution (per-iteration)",
        ylabel="ms",
        out_path=plots_dir / "verify_boxplot.png",
    )

    # 2) Proof size distribution (bytes) from show iterations
    df_proof = df_show.dropna(subset=["artifact_bytes"]).copy()
    if not df_proof.empty:
        boxplot_by_param(
            df_proof,
            value_col="artifact_bytes",
            title="proof size distribution (bytes) — artifacts emitted by show",
            ylabel="bytes",
            out_path=plots_dir / "proof_size_boxplot.png",
        )

        # 3) Scatter: show latency vs proof size
        scatter_show_vs_proof(df_show, plots_dir / "show_vs_proof_scatter.png")

    # 4) “Impressive” plot: amortization curve (offline cost / n)
    amortized_cost_curve(results_latest, plots_dir / "amortized_cost_curve.png")

    # 5) Breakdown: offline vs online stacked
    offline_online_breakdown(results_latest, plots_dir / "offline_online_breakdown.png")

    print(f"[ok] wrote plots under: {plots_dir}")


if __name__ == "__main__":
    main()
