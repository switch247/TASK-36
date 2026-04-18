#[cfg(test)]
mod tests {
    use app_core::cleanse::{normalize_mmddyyyy, standardize_currency_to_usd, standardize_units};

    #[test]
    fn standardizes_units_to_metric_base() {
        let (value, unit) = standardize_units(2.0, "km").expect("unit conversion should succeed");
        assert_eq!(unit, "m");
        assert!((value - 2000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn standardizes_currency_to_usd() {
        let usd = standardize_currency_to_usd(100.0, "KES", 0.0078)
            .expect("currency conversion should succeed");
        assert_eq!(usd, "USD 0.78");
    }

    #[test]
    fn normalizes_mmddyyyy_to_iso() {
        let normalized =
            normalize_mmddyyyy("03/26/2026").expect("date normalization should succeed");
        assert_eq!(normalized, "2026-03-26");
    }

    #[test]
    fn rejects_invalid_calendar_date() {
        assert!(normalize_mmddyyyy("02/30/2026").is_err());
    }
}
