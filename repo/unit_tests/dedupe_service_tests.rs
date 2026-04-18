#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use app_services::dedupe_service::{DedupeCandidate, DedupeService};

    #[test]
    fn guided_merge_match_returns_true_for_same_id() {
        let dob = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let left = DedupeCandidate { id: "ID-123", name: "Alice Noor", dob };
        let right = DedupeCandidate { id: "ID-123", name: "Alice N.", dob };
        assert!(DedupeService::guided_merge_match(left, right));
    }

    #[test]
    fn guided_merge_similarity_reflects_close_names() {
        let score = DedupeService::guided_merge_similarity("Johnathan Doe", "Jonathan Doe");
        assert!(score >= 0.90);
    }

    #[test]
    fn should_surface_guided_merge_requires_match_signal() {
        let dob = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let left = DedupeCandidate { id: "", name: "Alice Noor", dob };
        let right = DedupeCandidate { id: "", name: "Completely Different", dob };
        let should_surface = DedupeService::should_surface_guided_merge(left, right).unwrap();
        assert!(!should_surface);
    }
}
