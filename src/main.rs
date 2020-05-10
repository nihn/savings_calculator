use std::path::PathBuf;
use structopt::StructOpt;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, StructOpt)]
#[structopt(about = "Simple script to parse and combine savings in multiple currencies")]
struct SavingsCalc {
    #[structopt(subcommand)]
    cmd: Command
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Add data to our savings spreadsheet
    Add {
        /// Input csv file
        #[structopt(parse(from_os_str))]
        input: PathBuf,
    },
    /// Parse our shaving spreadsheet and display data
    Show {
        /// Input csv file
        #[structopt(parse(from_os_str))]
        input: PathBuf,
    }
}


#[derive(Debug)]
struct Record {
    date: String,
    savings: HashMap<String, f32>,
}

fn parse(file: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file)?;
    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here..
        let record = result?;
        println!("{:?}", record);
    }
    Ok(())
}

fn main() {
    let opt = SavingsCalc::from_args();
    let input_file = match opt.cmd {
        Command::Add{ input } => {
            input
        },
        Command::Show{ input} => {
            input
        }
    };
    let parsed = parse(input_file);
    println!("{:?}", parsed);
}
