use anyhow::Result;
use chrono::NaiveDate;

use app_core::dedupe::{is_guided_merge_match, similarity_score};

pub struct DedupeCandidate<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub dob: NaiveDate,
}

pub struct DedupeService;

impl DedupeService {
    pub fn guided_merge_match(left: DedupeCandidate<'_>, right: DedupeCandidate<'_>) -> bool {
        is_guided_merge_match(left.id, right.id, left.name, right.name, left.dob, right.dob)
    }

    pub fn guided_merge_similarity(left_name: &str, right_name: &str) -> f64 {
        similarity_score(left_name, right_name)
    }

    pub fn should_surface_guided_merge(left: DedupeCandidate<'_>, right: DedupeCandidate<'_>) -> Result<bool> {
        let score = Self::guided_merge_similarity(left.name, right.name);
        Ok((score >= 0.90 && left.dob == right.dob) || (!left.id.is_empty() && left.id == right.id))
    }
}
