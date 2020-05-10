use chrono::NaiveDate;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

static DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Debug)]
pub struct Record {
    pub date: NaiveDate,
    pub savings: Vec<f32>,
}

#[derive(Debug)]
pub struct Records {
    pub records: Vec<Record>,
    pub currencies: Vec<String>,
}

pub fn parse_from_str(file_path: &str) -> Result<Records, Box<dyn Error>> {
    let mut records = vec![];
    let mut rdr = csv::Reader::from_path(file_path)?;

    let headers = rdr.headers()?.clone();
    let currencies: Vec<String> = headers.iter().skip(1).map(String::from).collect();

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
