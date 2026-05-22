pub fn calculate_win_percent(cp: i32) -> f64 {
    50.0 + 50.0
        * (2.0
            / (1.0
                + (-0.00368 * cp as f64).exp())
            - 1.0)
}

pub fn calculate_accuracy(
    total_win_loss: f64,
    num_moves: u32,
) -> f64 {
    if num_moves == 0 {
        return 100.0;
    }

    let avg_wpl =
        total_win_loss / (num_moves as f64);

    let raw_accuracy =
        100.0 * (-avg_wpl / 15.0).exp();

    let clamped = raw_accuracy.clamp(0.0, 100.0);
    (clamped * 10.0).round() / 10.0
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

    #[test]
    fn test_accuracy_curve() {
        assert_eq!(
            calculate_accuracy(0.0, 40),
            100.0
        );

        let good_acc =
            calculate_accuracy(80.0, 40);
        assert!(
            good_acc > 80.0 && good_acc < 90.0
        );

        let bad_acc =
            calculate_accuracy(400.0, 40);
        assert!(bad_acc > 40.0 && bad_acc < 60.0);
    }
}
