use crate::parse::{Currency, Record, Records};
use chrono::NaiveDate;
use futures::future::join_all;
use reqwest::{Client, Url};
use serde::Deserialize;
use std::collections::HashMap;

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

    let rates = if let Some(date) = date {
        vec![query(&client, &date, &exchange_to, &records.currencies).await?]
    } else {
        let futures = records
            .records
            .iter()
            .map(|rec| query(&client, &rec.date, &exchange_to, &records.currencies));
        join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<Vec<f32>>, String>>()?
    };

    let mut new_records = Vec::new();

    for (i, record) in records.records.iter().enumerate() {
        let date_rates: &Vec<f32> = if rates.len() == 1 {
            rates[0].as_ref()
        } else {
            rates[i].as_ref()
        };
        let savings = record
            .savings
            .iter()
            .enumerate()
            .map(|(i, s)| s / date_rates[i])
            .fold(0.0, |acc, x| acc + x);

        new_records.push(Record {
            date: record.date,
            savings: vec![savings],
        })
    }
    Ok(Records {
        records: new_records.clone(),
        currencies: vec![exchange_to],
        filepath: records.filepath,
    })
}

async fn query(
    client: &Client,
    date: &NaiveDate,
    exchange_to: &Currency,
    currencies: &Vec<Currency>,
) -> Result<Vec<f32>, String> {
    let url = Url::parse(EBC_API_ADDR)
        .unwrap()
        .join(date.format(DATE_FMT).to_string().as_str())
        .unwrap();

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

    Ok(currencies
        .iter()
        .map(|curr| rates.rates[&curr.to_string()])
        .collect())
}
