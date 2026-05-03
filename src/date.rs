use crate::error::CliError;
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueDateInput {
    pub milliseconds: String,
    pub has_time: bool,
}

fn invalid_date_error(date_str: &str) -> CliError {
    CliError::ClientError {
        message: format!(
            "Invalid date '{}'. Use YYYY-MM-DD or RFC3339 datetime format.",
            date_str
        ),
        status: 0,
    }
}

pub fn parse_date_only(date_str: &str) -> Result<String, CliError> {
    let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| invalid_date_error(date_str))?;
    let dt = naive.and_hms_opt(0, 0, 0).unwrap().and_utc();
    Ok(dt.timestamp_millis().to_string())
}

pub fn parse_due_date(date_str: &str) -> Result<DueDateInput, CliError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        let utc = dt.with_timezone(&Utc);
        return Ok(DueDateInput {
            milliseconds: utc.timestamp_millis().to_string(),
            has_time: true,
        });
    }

    Ok(DueDateInput {
        milliseconds: parse_date_only(date_str)?,
        has_time: false,
    })
}

pub fn date_to_ms(date_str: &str) -> Result<String, CliError> {
    parse_date_only(date_str)
}

fn format_ms(ms: i64, has_time: bool) -> Option<String> {
    let dt = DateTime::from_timestamp_millis(ms)?;
    Some(if has_time {
        dt.to_rfc3339_opts(SecondsFormat::Millis, true)
    } else {
        dt.format("%Y-%m-%d").to_string()
    })
}

pub fn format_due_date_value(
    value: Option<&serde_json::Value>,
    due_date_time: bool,
) -> Option<String> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => {
            if let Ok(ms) = s.parse::<i64>() {
                format_ms(ms, due_date_time).or(Some(s.clone()))
            } else {
                Some(s.clone())
            }
        }
        Some(serde_json::Value::Number(n)) => {
            n.as_i64().and_then(|ms| format_ms(ms, due_date_time))
        }
        _ => None,
    }
}
