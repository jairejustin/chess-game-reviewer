pub fn calculate_win_percent(cp: i32) -> f64 {
    50.0 + 50.0
        * (2.0
            / (1.0
                + (-0.00368 * cp as f64).exp())
            - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_win_percent_bounds() {
        let win_0 = calculate_win_percent(0);
        assert!((win_0 - 50.0).abs() < 0.1, "0 cp should be exactly 50% win probability");

        let win_high =
            calculate_win_percent(1000);
        assert!(win_high > 95.0, "High positive cp should yield near 100% win probability");

        let win_low =
            calculate_win_percent(-1000);
        assert!(win_low < 5.0, "High negative cp should yield near 0% win probability");
    }
}
