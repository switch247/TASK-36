use chrono::NaiveDate;
use regex::Regex;

use crate::errors::CoreError;

pub fn standardize_units(value: f64, unit: &str) -> Result<(f64, String), CoreError> {
    let canonical = unit.trim().to_lowercase();
    match canonical.as_str() {
        "m" | "meter" | "meters" => Ok((value, "m".to_string())),
        "km" | "kilometer" | "kilometers" => Ok((value * 1000.0, "m".to_string())),
        "cm" | "centimeter" | "centimeters" => Ok((value / 100.0, "m".to_string())),
        "kg" | "kilogram" | "kilograms" => Ok((value, "kg".to_string())),
        "g" | "gram" | "grams" => Ok((value / 1000.0, "kg".to_string())),
        "lb" | "lbs" | "pound" | "pounds" => Ok((value * 0.45359237, "kg".to_string())),
        _ => Err(CoreError::NormalizationError(format!(
            "unsupported unit: {unit}"
        ))),
    }
}

pub fn standardize_currency_to_usd(
    amount: f64,
    currency: &str,
    fx_rate_to_usd: f64,
) -> Result<String, CoreError> {
    if fx_rate_to_usd <= 0.0 {
        return Err(CoreError::NormalizationError(
            "fx_rate_to_usd must be positive".to_string(),
        ));
    }

    let normalized_currency = currency.trim().to_uppercase();
    let usd_value = if normalized_currency == "USD" {
        amount
    } else {
        amount * fx_rate_to_usd
    };

    Ok(format!("USD {:.2}", usd_value))
}

pub fn normalize_mmddyyyy(input: &str) -> Result<String, CoreError> {
    let re = Regex::new(r"^(\d{2})/(\d{2})/(\d{4})$")
        .map_err(|e| CoreError::NormalizationError(e.to_string()))?;
    let captures = re
        .captures(input)
        .ok_or_else(|| CoreError::NormalizationError("date must match MM/DD/YYYY".to_string()))?;

    let month: u32 = captures
        .get(1)
        .ok_or_else(|| CoreError::NormalizationError("missing month".to_string()))?
        .as_str()
        .parse()
        .map_err(|_| CoreError::NormalizationError("invalid month".to_string()))?;
    let day: u32 = captures
        .get(2)
        .ok_or_else(|| CoreError::NormalizationError("missing day".to_string()))?
        .as_str()
        .parse()
        .map_err(|_| CoreError::NormalizationError("invalid day".to_string()))?;
    let year: i32 = captures
        .get(3)
        .ok_or_else(|| CoreError::NormalizationError("missing year".to_string()))?
        .as_str()
        .parse()
        .map_err(|_| CoreError::NormalizationError("invalid year".to_string()))?;

    let date = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| CoreError::NormalizationError("invalid calendar date".to_string()))?;
    Ok(date.format("%Y-%m-%d").to_string())
}
