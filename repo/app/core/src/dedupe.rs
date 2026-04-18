use chrono::NaiveDate;
use strsim::normalized_levenshtein;

pub fn is_guided_merge_match(
    left_id: &str,
    right_id: &str,
    left_name: &str,
    right_name: &str,
    left_dob: NaiveDate,
    right_dob: NaiveDate,
) -> bool {
    if !left_id.trim().is_empty() && left_id == right_id {
        return true;
    }

    let name_similarity = normalized_levenshtein(
        &left_name.trim().to_lowercase(),
        &right_name.trim().to_lowercase(),
    );

    name_similarity >= 0.90 && left_dob == right_dob
}

pub fn similarity_score(left_name: &str, right_name: &str) -> f64 {
    normalized_levenshtein(
        &left_name.trim().to_lowercase(),
        &right_name.trim().to_lowercase(),
    )
}
