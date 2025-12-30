import argparse
from pathlib import Path

import pandas as pd
import matplotlib.pyplot as plt


def load_runs(pattern: str) -> pd.DataFrame:
    files = sorted(Path(".").glob(pattern))
    if not files:
        raise SystemExit(f"No results.csv found for pattern: {pattern}")

    dfs = []
    for f in files:
        df = pd.read_csv(f)
        df["run_dir"] = str(f.parent)
        dfs.append(df)

    out = pd.concat(dfs, ignore_index=True)

    # keep the newest run per (param, step) by run_id lexicographic timestamp
    out = out.sort_values(["param", "step", "run_id"]).groupby(["param", "step"], as_index=False).tail(1)
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--pattern",
        default="out/run_*/results.csv",
        help="Glob for run result CSVs (default: out/run_*/results.csv)",
    )
    ap.add_argument(
        "--out",
        default="out/plots",
        help="Output directory for merged CSV + plots",
    )
    args = ap.parse_args()

    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    df = load_runs(args.pattern)
    df.to_csv(out_dir / "merged_results.csv", index=False)

    # 1) Mean runtime by step, grouped by param
    pivot_mean = df.pivot(index="step", columns="param", values="mean_ms")
    ax = pivot_mean.plot(kind="bar")
    ax.set_ylabel("mean_ms")
    ax.set_title("Mean runtime by step (latest run per param)")
    fig = ax.get_figure()
    fig.tight_layout()
    fig.savefig(out_dir / "mean_ms_by_step.png", dpi=150)
    plt.close(fig)

    # 2) Show/verify latency focus: p50/p95
    for step in ["show", "verify", "prove", "zksetup"]:
        sdf = df[df["step"] == step].set_index("param")
        if sdf.empty:
            continue
        plot_df = sdf[["p50_ms", "p95_ms", "mean_ms"]]
        ax = plot_df.plot(kind="bar")
        ax.set_ylabel("ms")
        ax.set_title(f"{step}: p50/p95/mean (latest run per param)")
        fig = ax.get_figure()
        fig.tight_layout()
        fig.savefig(out_dir / f"{step}_p50_p95_mean.png", dpi=150)
        plt.close(fig)

    # 3) Failures (should be zero, but print if not)
    fails = df[df["fail"] > 0][["param", "step", "iters", "ok", "fail", "run_dir"]]
    if not fails.empty:
        print("Non-zero failures detected:")
        print(fails.to_string(index=False))
    else:
        print("All selected runs have fail=0.")

    print(f"Wrote: {(out_dir / 'merged_results.csv').as_posix()}")
    print(f"Wrote plots under: {out_dir.as_posix()}")


if __name__ == "__main__":
    main()