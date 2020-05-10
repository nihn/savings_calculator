use chrono::NaiveDate;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

static DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Debug)]
pub struct Record {
    date: NaiveDate,
    savings: HashMap<String, f32>,
}

pub fn parse(file: PathBuf) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut res = vec![];
    let mut rdr = csv::Reader::from_path(file)?;

    let headers = rdr.headers()?.clone();
    let currencies: Vec<&str> = headers.iter().skip(1).collect();

    for result in rdr.records() {
        let result_ = result?;
        let mut result_iter = result_.into_iter();
        let mut savings = HashMap::<String, f32>::new();

        let date = result_iter.next().expect("Empty row found!");

        for (i, column) in result_iter.enumerate() {
            assert!(currencies.len() > i);
            let amount: f32 = column.parse()?;
            savings.insert(currencies[i].to_string(), amount);
        }
        res.push(Record {
            date: NaiveDate::parse_from_str(date, DATE_FORMAT)?,
            savings,
        })
    }
    Ok(res)
}
