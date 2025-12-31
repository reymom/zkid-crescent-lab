pub fn summarize_u128(durations: &[u128]) -> (u128, u128, u128, u128, u128) {
    if durations.is_empty() {
        return (0, 0, 0, 0, 0);
    }

    let mut v = durations.to_vec();
    v.sort_unstable();

    let min = v[0];
    let max = v[v.len() - 1];
    let mean = (v.iter().sum::<u128>()) / (v.len() as u128);

    let p50 = quantile_sorted_u128(&v, 0.50);
    let p95 = quantile_sorted_u128(&v, 0.95);

    (min, p50, p95, mean, max)
}

pub fn summarize_u64(values: &[u64]) -> (u64, u64, u64) {
    // returns (p50, mean, max) to keep it simple for the blog
    if values.is_empty() {
        return (0, 0, 0);
    }

    let mut v = values.to_vec();
    v.sort_unstable();

    let max = v[v.len() - 1];
    let mean = (v.iter().map(|x| *x as u128).sum::<u128>() / (v.len() as u128)) as u64;
    let p50 = quantile_sorted_u64(&v, 0.50);

    (p50, mean, max)
}

fn quantile_sorted_u128(sorted: &[u128], q: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let n = sorted.len() as f64;
    let idx = ((q * (n - 1.0)).round() as usize).min(sorted.len() - 1);
    sorted[idx]
}

fn quantile_sorted_u64(sorted: &[u64], q: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let n = sorted.len() as f64;
    let idx = ((q * (n - 1.0)).round() as usize).min(sorted.len() - 1);
    sorted[idx]
}
