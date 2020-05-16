use crate::parse::{Record, Records};
use chrono::{Duration, NaiveDate};

pub fn calculate_rolling_average(
    records: Records,
    period: Duration,
    sum: bool,
    buckets: Option<Duration>,
    start_date: Option<NaiveDate>,
) -> Result<Records, String> {
    let currencies = records.currencies.clone();
    let records = match start_date {
        Some(date) => records.records_newer_than(date),
        None => records.records,
    };

    let days = period.num_days() as f32;

    let records_groups = if let Some(buckets) = buckets {
        let mut end = records[0].date + buckets;
        let mut result = vec![];
        let mut current = vec![];

        for record in records {
            if record.date > end {
                result.push(current);
                current = vec![];
                end = end + buckets;
                continue;
            }
            current.push(record);
        }
        result
    } else {
        vec![records]
    };

    let result = records_groups
        .into_iter()
        .map(|records| calculate_records(records, days, sum))
        .flatten()
        .collect();

    Ok(Records {
        currencies,
        records: result,
    })
}

fn calculate_records(mut records: Vec<Record>, days: f32, sum: bool) -> Vec<Record> {
    if records.len() < 2 {
        return vec![];
    }
    if sum {
        records = vec![records.remove(0), records.remove(records.len() - 1)];
    }

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
    result
}
