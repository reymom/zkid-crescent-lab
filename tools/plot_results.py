import argparse
from pathlib import Path

import pandas as pd
import matplotlib.pyplot as plt


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", required=True, help="Path to run dir, e.g. out/run_20251229T235030Z")
    args = ap.parse_args()

    run_dir = Path(args.run)
    csv_path = run_dir / "results.csv"
    if not csv_path.exists():
        raise SystemExit(f"missing: {csv_path}")

    df = pd.read_csv(csv_path)

    # Keep only successful rows for timing plots (still show fail counts separately).
    ok_df = df[df["ok"] > 0].copy()

    # 1) One figure: mean_ms by (param, step)
    pivot = ok_df.pivot(index="step", columns="param", values="mean_ms")
    ax = pivot.plot(kind="bar")
    ax.set_ylabel("mean_ms")
    ax.set_title(f"Mean runtime by step ({run_dir.name})")
    fig = ax.get_figure()
    fig.tight_layout()
    fig.savefig(run_dir / "mean_ms_by_step.png", dpi=150)
    plt.close(fig)

    # 2) One figure per step: p50/p95 as bars
    for step in ok_df["step"].unique():
        sdf = ok_df[ok_df["step"] == step].set_index("param")[["p50_ms", "p95_ms"]]
        ax = sdf.plot(kind="bar")
        ax.set_ylabel("ms")
        ax.set_title(f"{step}: p50/p95 ({run_dir.name})")
        fig = ax.get_figure()
        fig.tight_layout()
        fig.savefig(run_dir / f"{step}_p50_p95.png", dpi=150)
        plt.close(fig)

    # 3) Failures table (printed)
    fail_df = df[["param", "step", "iters", "fail"]].sort_values(["fail", "param"], ascending=[False, True])
    print("Failures:")
    print(fail_df.to_string(index=False))


if __name__ == "__main__":
    main()
