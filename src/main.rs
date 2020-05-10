use std::path::PathBuf;
use structopt::StructOpt;

mod parse;

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
        #[structopt(parse(from_os_str))]
        input: PathBuf,
    },
    /// Parse our shaving spreadsheet and display data
    Show {
        /// Input csv file
        #[structopt(parse(from_os_str))]
        input: PathBuf,
    },
}

fn main() {
    let opt = SavingsCalc::from_args();
    let input_file = match opt.cmd {
        Command::Add { input } => input,
        Command::Show { input } => input,
    };
    let parsed = parse::parse(input_file).unwrap();
    println!("{:?}", parsed);
}
