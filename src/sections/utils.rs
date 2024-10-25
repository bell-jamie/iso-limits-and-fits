pub fn decimal_places(num: f64, decimals: i32) -> f64 {
    // Negative decimals inherit the default decimal places value
    let power = if decimals >= 0 { decimals } else { 6 };
    let factor = 10f64.powi(power);
    (num * factor).round() / factor
}
