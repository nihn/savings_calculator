use crate::parse::{Currency, Records};
use chrono::NaiveDate;
use reqwest::{Client, Error, Response, StatusCode, Url};

static EBC_API_ADDR: &str = "https://api.exchangeratesapi.io";
static DATE_FMT: &str = "%Y-%m-%d";

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

    println!("{}", url);
    let res = client
        .get(url)
        .query(&[("base", exchange_to.to_string())])
        .send()
        .await
        .map_err(|err| format!("Error querying ebc: {:?}", err))?;
    println!("{:?}", res.text().await);
    Ok(records)
}
