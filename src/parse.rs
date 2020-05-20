use chrono::{Duration, NaiveDate, Utc};
use humantime;
use simple_error::{bail, SimpleError, SimpleResult};
use std::error::Error;
use std::fmt;
use std::num::ParseFloatError;

static DATE_FORMAT: &str = "%Y-%m-%d";
static TODAY: &str = "today";

#[derive(Debug, Clone)]
pub struct Record {
    pub date: NaiveDate,
    pub savings: Vec<f32>,
}

#[derive(Debug)]
pub struct Records {
    pub records: Vec<Record>,
    pub currencies: Vec<Currency>,
    pub filepath: String,
}

impl Records {
    pub fn records_newer_older_than(
        self,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
    ) -> Vec<Record> {
        let records = if let Some(start) = start {
            self.records
                .to_vec()
                .into_iter()
                .filter(|r| r.date >= start)
                .collect()
        } else {
            self.records
        };

        if let Some(end) = end {
            records
                .to_vec()
                .into_iter()
                .filter(|r| r.date <= end)
                .collect()
        } else {
            records
        }
    }

    /// Update records with new Value, if date is alraedy present in records,
    /// overwrite the given currency amount, if not add new Record with the new
    /// currency amount and other currencies copied from previous record.
    /// If new currency is added fill previous dates with 0 and next datess with
    /// the same amount.
    pub fn set_value(&mut self, val: &Value, date: NaiveDate) {
        let new_currency = !self.currencies.contains(&val.currency);
        if new_currency {
            self.currencies.push(val.currency.clone());
            for record in self.records.iter_mut() {
                record.savings.push(0.0);
            }
        }
        let idx = self
            .currencies
            .iter()
            .position(|c| c == &val.currency)
            .unwrap();

        let mut last = 0;
        for (i, record) in self.records.iter_mut().enumerate() {
            if record.date == date {
                record.savings[idx] = val.amount;
                break;
            } else if record.date > date {
                let mut savings = self.records[i - 1].savings.clone();
                savings[idx] = val.amount;
                self.records.insert(i, Record { date, savings });
                break;
            }
            last = i + 1;
        }

        if last == self.records.len() - 1 {
            let mut savings = self.records[last].savings.clone();
            savings[idx] = val.amount;
            self.records.push(Record { date, savings });
        } else if new_currency {
            while last < self.records.len() - 1 {
                self.records[last + 1].savings[idx] = self.records[last].savings[idx];
                last += 1;
            }
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Currency(pub String);

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Currency {
    fn new(value: &str) -> SimpleResult<Self> {
        if value.len() != 3 {
            Err(SimpleError::new("Currency code has to have 3 characters!"))
        } else if value.chars().any(|c| !c.is_alphabetic()) {
            Err(SimpleError::new(
                "Currency code has to have only alphabetic characters!",
            ))
        } else {
            Ok(Currency(value.to_uppercase()))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Value {
    pub amount: f32,
    pub currency: Currency,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.currency)
    }
}

impl std::str::FromStr for Value {
    type Err = SimpleError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let currency = Currency::new(&value[value.len() - 3..value.len()])?;
        let amount = &value[..value.len() - 3];
        let amount: f32 = match amount.parse() {
            Ok(res) => res,
            Err(err) => {
                return Err(SimpleError::new(format!(
                    "{} is not a valid float value!",
                    amount
                )))
            }
        };

        Ok(Value { amount, currency })
    }
}

pub fn parse_from_str(filepath: &str) -> Result<Records, Box<dyn Error>> {
    let mut records = vec![];
    let mut rdr = csv::Reader::from_path(filepath)?;

    let headers = rdr.headers()?.clone();
    let currencies: Vec<_> = headers
        .into_iter()
        .skip(1)
        .map(|c| Currency::new(c))
        .collect::<SimpleResult<Vec<Currency>>>()?;

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
        filepath: filepath.to_string(),
    })
}

pub fn update_csv_file(records: &Records) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(&records.filepath)?;
    let mut header = vec!["Date".to_string()];
    header.extend(records.currencies.iter().map(|c| c.0.clone()));

    wtr.write_record(header)?;

    for record in records.records.iter() {
        let mut row = vec![record.date.format(DATE_FORMAT).to_string()];
        row.extend(record.savings.iter().map(|s| s.to_string()));
        wtr.write_record(row)?;
    }
    wtr.flush()?;
    Ok(())
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
    Ok(Currency::new(currency)?)
}

pub fn parse_duration_from_str(duration: &str) -> Result<Duration, Box<dyn Error>> {
    let duration = Duration::from_std(humantime::parse_duration(duration)?)?;

    if duration < Duration::days(1) {
        bail!("Periods lesser than 1 days are not supported!");
    }

    Ok(duration)
}
