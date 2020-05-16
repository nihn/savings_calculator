use crate::parse::{Record, Records};
use chrono::{Duration, NaiveDate};
use futures::StreamExt;

pub fn calculate_rolling_average(
    records: Records,
    period: Duration,
    sum: bool,
    start_date: Option<NaiveDate>,
) -> Result<Records, String> {
    let currencies = records.currencies.clone();
    let mut records = match start_date {
        Some(date) => records.records_newer_than(date),
        None => records.records,
    };
    if sum {
        records = vec![records.remove(0), records.remove(records.len() - 1)];
    }

    let days = period.num_days() as f32;

    let mut result = vec![Record {
        date: records[0].date,
        savings: records[0].savings.iter().map(|_| 0.0).collect(),
    }];
    let ref first_record = records[0];

    for record in records.iter().skip(1) {
        let days_passed = (record.date - first_record.date).num_days() as f32;
        let per_period_savings = record
            .savings
            .iter()
            .enumerate()
            .map(|(i, s)| (s - first_record.savings[i]) / days_passed * days);

        result.push(Record {
            date: record.date,
            savings: per_period_savings.collect(),
        });
    }
    Ok(Records {
        currencies,
        records: result,
    })
}
