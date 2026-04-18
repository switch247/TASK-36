use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use sqlx::MySqlPool;

use app_core::cleanse::{normalize_mmddyyyy, standardize_currency_to_usd, standardize_units};
use app_models::entities::ZipCityRow;

pub struct CleansingService {
    pool: MySqlPool,
}

pub struct NormalizedRecord {
    pub normalized_value: f64,
    pub normalized_unit: String,
    pub normalized_amount_usd: String,
    pub normalized_date: String,
}

impl CleansingService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub fn normalize_record(
        &self,
        value: f64,
        unit: &str,
        amount: f64,
        currency: &str,
        fx_rate_to_usd: f64,
        date_mmddyyyy: &str,
    ) -> Result<NormalizedRecord> {
        let (normalized_value, normalized_unit) = standardize_units(value, unit)?;
        let normalized_amount_usd = standardize_currency_to_usd(amount, currency, fx_rate_to_usd)?;
        let normalized_date = normalize_mmddyyyy(date_mmddyyyy)?;

        Ok(NormalizedRecord {
            normalized_value,
            normalized_unit,
            normalized_amount_usd,
            normalized_date,
        })
    }

    pub async fn validate_zip_city(&self, zip: &str, city: &str) -> Result<bool> {
        let record = sqlx::query_as::<_, ZipCityRow>(
            r#"SELECT zip_code, city, state, country
               FROM zip_city_reference
               WHERE zip_code = ? LIMIT 1"#,
        )
        .bind(zip)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record
            .map(|r| r.city.trim().eq_ignore_ascii_case(city.trim()))
            .unwrap_or(false))
    }

    pub fn is_room_capacity_outlier(capacity: i32, all_capacities: &[i32]) -> Result<bool> {
        if all_capacities.is_empty() {
            return Err(anyhow!("capacity baseline cannot be empty"));
        }
        let avg =
            all_capacities.iter().map(|v| *v as f64).sum::<f64>() / all_capacities.len() as f64;
        Ok((capacity as f64) > (avg * 3.0))
    }

    pub fn parse_dob(date_mmddyyyy: &str) -> Result<NaiveDate> {
        let iso = normalize_mmddyyyy(date_mmddyyyy)?;
        Ok(NaiveDate::parse_from_str(&iso, "%Y-%m-%d")?)
    }
}
