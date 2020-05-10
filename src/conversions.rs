use crate::parse::{Currency, Record, Records};
use chrono::NaiveDate;
use reqwest::{Client, Error, Response, StatusCode, Url};
use serde::Deserialize;
use std::collections::HashMap;
use stderrlog::new;
use std::process::exit;

static EBC_API_ADDR: &str = "https://api.exchangeratesapi.io";
static DATE_FMT: &str = "%Y-%m-%d";

#[derive(Deserialize, Debug)]
struct Rates {
    rates: HashMap<String, f32>,
}

pub async fn get_conversions(
    records: Records,
    exchange_to: Currency,
    date: Option<NaiveDate>,
) -> Result<Records, String> {
    let client = Client::new();

    let url = if let Some(date) = date {
        Url::parse(EBC_API_ADDR)
            .unwrap()
            .join(date.format(DATE_FMT).to_string().as_str())
            .unwrap()
    } else {
        Url::parse(EBC_API_ADDR).unwrap()
    };

    let res = client
        .get(url)
        .query(&[("base", exchange_to.to_string())])
        .send()
        .await
        .map_err(|err| format!("Error querying ebc: {:?}", err))?;

    let rates: Rates = res
        .json()
        .await
        .map_err(|err| format!("Invalid json: {:?}", err))?;

    let rates: Vec<f32> = records
        .currencies
        .iter()
        .map(|curr| rates.rates[&curr.to_string()])
        .collect();

    let mut new_records = Vec::new();

    for record in records.records {
        let savings = record
            .savings
            .iter()
            .enumerate()
            .map(|(i, s)| s / rates[i])
            .fold(0.0,|acc, x| acc + x);

        new_records.push(Record {
            date: record.date,
            savings: vec![savings],
        })
    }
    println!("{:?}", new_records);
    Ok(Records {
        records: new_records,
        currencies: vec![exchange_to],
    })
}
