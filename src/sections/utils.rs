pub fn decimals(num: f64, decimals: i32) -> f64 {
    // Negative decimals inherit the default decimal places value
    let power = if decimals >= 0 { decimals } else { 6 };
    let factor = 10f64.powi(power);
    (num * factor).round() / factor
}

pub fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let t = i as f64 / (n as f64 - 1.0);
            a + t * (b - a)
        })
        .collect()
}
