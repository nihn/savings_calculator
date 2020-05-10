use chrono::{NaiveDate, Utc};
use std::path::PathBuf;
use structopt::StructOpt;
use tokio;

mod conversions;
mod parse;
mod table;

#[derive(Debug, StructOpt)]
#[structopt(about = "Simple script to parse and combine savings in multiple currencies")]
struct SavingsCalc {
    #[structopt(subcommand)]
    cmd: Command,
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
    },
}

#[tokio::main]
async fn main() {
    let opt = SavingsCalc::from_args();

    match opt.cmd {
        Command::Table { records } => {
            table::format_table(records).printstd();
        }
        Command::Add { records } => println!("Not implemented!"),
        Command::Converse {
            records,
            date,
            currency,
        } => {
            let records = conversions::get_conversions(records, currency, date).await.unwrap();
        }
    };
}
