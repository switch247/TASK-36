#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use app_core::dedupe::{is_guided_merge_match, similarity_score};

    #[test]
    fn matches_on_exact_id_even_with_name_variance() {
        let dob = NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid test date");
        assert!(is_guided_merge_match(
            "ID-123",
            "ID-123",
            "Amina Noor",
            "Amina N.",
            dob,
            dob
        ));
    }

    #[test]
    fn matches_on_similarity_threshold_and_dob() {
        let dob = NaiveDate::from_ymd_opt(1999, 12, 31).expect("valid test date");
        let score = similarity_score("Johnathan Doe", "Jonathan Doe");
        assert!(score >= 0.90);
        assert!(is_guided_merge_match(
            "",
            "",
            "Johnathan Doe",
            "Jonathan Doe",
            dob,
            dob
        ));
    }

    #[test]
    fn rejects_when_similarity_below_threshold() {
        let dob = NaiveDate::from_ymd_opt(1999, 12, 31).expect("valid test date");
        let score = similarity_score("Jane Doe", "Completely Different");
        assert!(score < 0.90);
        assert!(!is_guided_merge_match(
            "",
            "",
            "Jane Doe",
            "Completely Different",
            dob,
            dob
        ));
    }
}
