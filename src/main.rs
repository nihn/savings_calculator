use chrono::{Duration, NaiveDate};
use clap;
use dialoguer::Confirm;
use std::collections::HashSet;
use structopt::StructOpt;
use tokio;

mod conversions;
mod format;
mod parse;
mod statistics;

#[derive(Debug, StructOpt)]
#[structopt(about = "Simple script to parse and combine savings in multiple currencies")]
struct SavingsCalc {
    #[structopt(subcommand)]
    cmd: Command,

    /// Format of outputted data
    #[structopt(long, possible_values = &format::Format::variants(), case_insensitive = true, default_value = "Table")]
    format: format::Format,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Add data to our savings spreadsheet
    Add {
        /// Input csv file
        #[structopt(parse(try_from_str = parse::parse_from_str))]
        records: parse::Records,
        /// Date of the entry, if nothing is passed today will be used
        #[structopt(short, long, default_value = "today", value_name = "YYYY-MM-DD", parse(try_from_str = parse::parse_date_from_str))]
        date: NaiveDate,

        /// Amount along with currency name, e.g. 123.45GBP
        #[structopt(short, long, required = true)]
        value: Vec<parse::Value>,

        /// Do not write file, only show what the result would look like
        #[structopt(long)]
        dry_run: bool,
    },
    /// Parse our saving spreadsheet and display data
    Show {
        /// Input csv file
        #[structopt(parse(try_from_str = parse::parse_from_str))]
        records: parse::Records,
    },
    /// Parse and converse into other currencies
    Converse {
        /// Input csv file
        #[structopt(parse(try_from_str = parse::parse_from_str))]
        records: parse::Records,

        /// Exchange rate for date, pass `today` for Today date
        #[structopt(short, long, value_name = "YYYY-MM-DD", parse(try_from_str = parse::parse_date_from_str))]
        date: Option<NaiveDate>,

        #[structopt(parse(try_from_str = parse::parse_currency_from_str))]
        currency: parse::Currency,

        /// Add deltas between entries
        #[structopt(short = "D", long)]
        delta: bool,
    },
    /// Calculate averages
    RollingAverage {
        /// Input csv file
        #[structopt(parse(try_from_str = parse::parse_from_str))]
        records: parse::Records,

        /// Over what period rolling average should be calculated
        #[structopt(default_value = "1 month", parse(try_from_str = parse::parse_duration_from_str))]
        period: Duration,

        /// Currency in which should averages be presented, if not passed due per currency averages
        #[structopt(short, long, parse(try_from_str = parse::parse_currency_from_str))]
        currency: Option<parse::Currency>,

        /// Exchange rate for date, pass `today` for Today date
        #[structopt(short = "E", long, value_name = "YYYY-MM-DD", parse(try_from_str = parse::parse_date_from_str))]
        exchange_rate_date: Option<NaiveDate>,

        /// Start date - first data point >= than this date will be used
        #[structopt(short, long, value_name = "YYYY-MM-DD", parse(try_from_str = parse::parse_date_from_str))]
        start_date: Option<NaiveDate>,

        /// End date - first data point <= than this date will be used
        #[structopt(short, long, value_name = "YYYY-MM-DD", parse(try_from_str = parse::parse_date_from_str))]
        end_date: Option<NaiveDate>,

        /// Show rolling average split into buckets, note that if there is not enough data points
        /// for given granurality results may be missing
        #[structopt(short, long, parse(try_from_str = parse::parse_duration_from_str))]
        buckets: Option<Duration>,

        /// Instead of doing per data point, calculate between first and last
        #[structopt(short = "S", long)]
        sum: bool,
    },
}

#[tokio::main]
async fn main() {
    let opt = SavingsCalc::from_args();
    match opt.cmd {
        Command::Show { records } => {
            format::present_results(records, opt.format);
        }
        Command::Add {
            mut records,
            date,
            value,
            dry_run,
        } => {
            let currencies: HashSet<_> = value.iter().map(|v| &v.currency).collect();
            if currencies.len() != value.len() {
                clap::Error::value_validation_auto("Duplicated currency passed!".into()).exit();
            }

            let new_currencies = currencies.into_iter().fold(vec![], |mut acc, x| {
                if !records.currencies.contains(x) {
                    acc.push(x);
                }
                acc
            });
            if !new_currencies.is_empty()
                && !Confirm::new()
                    .with_prompt(format!(
                        "Currencies {:?} are new, are you sure you want to add them?",
                        new_currencies
                    ))
                    .interact()
                    .unwrap()
            {
                clap::Error::with_description("Aborting!".into(), clap::ErrorKind::InvalidValue)
                    .exit();
            }
            if records.records.iter().any(|r| r.date == date) {
                if !Confirm::new()
                    .with_prompt(format!(
                        "Date {} already present in dataset, do you want to modify it?",
                        date
                    ))
                    .interact()
                    .unwrap()
                {
                    clap::Error::with_description(
                        "Aborting!".into(),
                        clap::ErrorKind::InvalidValue,
                    )
                    .exit();
                }
            }
            for value in value {
                records.set_value(&value, date);
            }
            if !dry_run {
                parse::update_csv_file(&records);
            }
            format::present_results(records, opt.format);
        }
        Command::Converse {
            records,
            date,
            currency,
            delta,
        } => {
            let mut records = conversions::get_conversions(records, currency, date)
                .await
                .unwrap();
            if delta {
                let deltas = records
                    .records
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(i, s)| parse::Record {
                        date: s.date,
                        savings: vec![
                            s.savings[0],
                            s.savings[0] - records.records[i - 1].savings[0],
                        ],
                    })
                    .collect();
                records = parse::Records {
                    currencies: vec![
                        records.currencies.remove(0),
                        parse::Currency("Delta".to_string()),
                    ],
                    records: deltas,
                    filepath: records.filepath,
                };
            }

            format::present_results(records, opt.format);
        }
        Command::RollingAverage {
            records,
            currency,
            period,
            exchange_rate_date,
            start_date,
            end_date,
            buckets,
            sum,
        } => {
            if let Some(buckets) = buckets {
                if buckets > period {
                    clap::Error::value_validation_auto(
                        "Buckets duration cannot be longer than period!".to_string(),
                    );
                }
            }

            let records = if let Some(currency) = currency {
                conversions::get_conversions(records, currency, exchange_rate_date)
                    .await
                    .unwrap()
            } else {
                records
            };
            let averages = statistics::calculate_rolling_average(
                records, period, sum, buckets, start_date, end_date,
            )
            .unwrap();
            format::present_results(averages, opt.format);
        }
    };
}
