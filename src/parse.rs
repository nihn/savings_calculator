use chrono::{NaiveDate, Utc};
use simple_error::bail;
use std::error::Error;
use std::fmt;
use std::iter::Cycle;

static DATE_FORMAT: &str = "%Y-%m-%d";
static TODAY: &str = "today";

#[derive(Debug)]
pub struct Record {
    pub date: NaiveDate,
    pub savings: Vec<f32>,
}

#[derive(Debug)]
pub struct Records {
    pub records: Vec<Record>,
    pub currencies: Vec<Currency>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Currency(String);

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn parse_from_str(file_path: &str) -> Result<Records, Box<dyn Error>> {
    let mut records = vec![];
    let mut rdr = csv::Reader::from_path(file_path)?;

    let headers = rdr.headers()?.clone();
    let currencies: Vec<Currency> = headers.iter().skip(1).map(|c| Currency(c.to_string())).collect();

    for result in rdr.records() {
        let result_ = result?;
        let mut result_iter = result_.into_iter();
        let mut savings = Vec::new();

        let date = result_iter.next().expect("Empty row found!");

        for (i, column) in result_iter.enumerate() {
            assert!(currencies.len() > i);
            let amount: f32 = column.parse()?;
            savings.push(amount);
        }
        records.push(Record {
            date: NaiveDate::parse_from_str(date, DATE_FORMAT)?,
            savings,
        })
    }
    Ok(Records {
        records,
        currencies,
    })
}

pub fn parse_date_from_str(date: &str) -> Result<NaiveDate, Box<dyn Error>> {
    if date == TODAY {
        return Ok(Utc::today().naive_local());
    }
    let parsed = NaiveDate::parse_from_str(date, DATE_FORMAT)?;
    if parsed > Utc::today().naive_local() {
        bail!("{:?} is in the future!", parsed);
    }
    Ok(parsed)
}

pub fn parse_currency_from_str(currency: &str) -> Result<Currency, Box<dyn Error>> {
    if currency.len() != 3 {
        bail!("Currency should be in three letter format, e.g. GBP, USD");
    }
    Ok(Currency(currency.to_uppercase()))
}
