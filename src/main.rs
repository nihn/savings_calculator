use chrono::{Duration, NaiveDate, Utc};
use clap;
use clap::arg_enum;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio;

mod conversions;
mod parse;
mod statistics;
mod format;

arg_enum! {
    #[derive(Debug)]
    enum Format {
        Table,
        Graph,
    }
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Simple script to parse and combine savings in multiple currencies")]
struct SavingsCalc {
    #[structopt(subcommand)]
    cmd: Command,

    /// Format of outputted data
    #[structopt(long, possible_values = &Format::variants(), case_insensitive = true, default_value = "Table")]
    format: Format,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Add data to our savings spreadsheet
    Add {
        /// Input csv file
        #[structopt(parse(try_from_str = parse::parse_from_str))]
        records: parse::Records,
    },
    /// Parse our shaving spreadsheet and display data
    Table {
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
        #[structopt(short, long, value_name = "YYYY-MM-DD", parse(try_from_str = parse::parse_date_from_str))]
        exchange_rate_date: Option<NaiveDate>,

        /// Start date - first data point >= than this date will be used
        #[structopt(short, long, value_name = "YYYY-MM-DD", parse(try_from_str = parse::parse_date_from_str))]
        start_date: Option<NaiveDate>,

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
        Command::Table { records } => {
            format::format_table(records).printstd();
        }
        Command::Add { records } => println!("Not implemented!"),
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
                };
            }

            format::format_table(records).printstd();
        }
        Command::RollingAverage {
            records,
            currency,
            period,
            exchange_rate_date,
            start_date,
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
            let averages =
                statistics::calculate_rolling_average(records, period, sum, buckets, start_date)
                    .unwrap();
            format::format_table(averages).printstd();
        }
    };
}
