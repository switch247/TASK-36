use anyhow::Result;
use chrono::NaiveDate;
use serde_json::Value;
use sqlx::MySqlPool;

use app_core::cleanse::normalize_mmddyyyy;
use app_core::crypto::{decrypt_dob, encrypt_dob};

use crate::dedupe_service::{DedupeCandidate, DedupeService};

pub struct CandidateService {
    pub pool: MySqlPool,
    pub aes_key: [u8; 32],
}

impl CandidateService {
    pub fn new(pool: MySqlPool, aes_key: [u8; 32]) -> Self {
        Self { pool, aes_key }
    }

    pub async fn create_candidate(
        &self,
        candidate_id: &str,
        date_of_birth: &str,
        national_id: &str,
        scanned_barcode: &str,
        metadata_json: &str,
        created_by: &str,
    ) -> Result<()> {
        let encrypted_dob = encrypt_dob(date_of_birth, &self.aes_key)?;

        sqlx::query(
            r#"INSERT INTO candidates (id, encrypted_dob, national_id, scanned_barcode, metadata, created_by)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(candidate_id)
        .bind(encrypted_dob)
        .bind(national_id)
        .bind(scanned_barcode)
        .bind(metadata_json)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub fn normalize_dob_mmddyyyy(&self, date_of_birth: &str) -> Result<String> {
        Ok(normalize_mmddyyyy(date_of_birth)?)
    }

    pub async fn find_exact_duplicate(
        &self,
        national_id: &str,
        scanned_barcode: &str,
    ) -> Result<Option<String>> {
        let existing = sqlx::query_scalar::<_, String>(
            "SELECT id FROM candidates WHERE national_id = ? OR scanned_barcode = ? LIMIT 1",
        )
        .bind(national_id)
        .bind(scanned_barcode)
        .fetch_optional(&self.pool)
        .await?;
        Ok(existing)
    }

    pub async fn find_guided_merge_duplicate(
        &self,
        national_id: &str,
        normalized_dob_iso: &str,
        candidate_name: Option<&str>,
    ) -> Result<Option<(String, f64)>> {
        let Some(incoming_name) = candidate_name.map(str::trim).filter(|n| !n.is_empty()) else {
            return Ok(None);
        };
        let incoming_dob = NaiveDate::parse_from_str(normalized_dob_iso, "%Y-%m-%d")?;

        let rows = sqlx::query_as::<_, (String, String, String, Value)>(
            "SELECT id, national_id, encrypted_dob, metadata FROM candidates",
        )
        .fetch_all(&self.pool)
        .await?;

        for (id, existing_national_id, encrypted_dob, metadata) in rows {
            let existing_dob_raw = match decrypt_dob(&encrypted_dob, &self.aes_key) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let existing_dob = match parse_any_supported_dob(&existing_dob_raw) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let existing_name = metadata
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or_default();
            if existing_name.is_empty() {
                continue;
            }

            let similarity = DedupeService::guided_merge_similarity(incoming_name, existing_name);
            let matches = DedupeService::guided_merge_match(
                DedupeCandidate {
                    id: national_id,
                    name: incoming_name,
                    dob: incoming_dob,
                },
                DedupeCandidate {
                    id: &existing_national_id,
                    name: existing_name,
                    dob: existing_dob,
                },
            );
            if matches {
                return Ok(Some((id, similarity)));
            }
        }

        Ok(None)
    }
}

fn parse_any_supported_dob(input: &str) -> Result<NaiveDate> {
    if let Ok(date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return Ok(date);
    }
    Ok(NaiveDate::parse_from_str(input, "%m/%d/%Y")?)
}
