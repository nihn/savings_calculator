use std::path::PathBuf;
use structopt::StructOpt;

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


fn main() {
    let opt = SavingsCalc::from_args();
    println!("{:?}", opt);
}
