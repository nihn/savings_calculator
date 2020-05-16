use chrono::{Duration, NaiveDate};
use crate::parse::{Record, Records};

pub fn calculate_rolling_average(
    records: Records,
    period: Duration,
    presentation_period: Duration,
    start_date: Option<NaiveDate>,
) -> Result<Vec<Record>, String> {
    Ok(vec![])
}
