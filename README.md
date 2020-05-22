# savings_calculator

Simple rust script to calculate your savings balance from different currencies, all the data is read and stored in CSV file.

```
cargo run -- --help

savings_calc 0.1.0
Simple script to parse and combine savings in multiple currencies

USAGE:
    savings_calc [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --format <format>    Format of outputted data [default: Table]  [possible values: Table, Graph]

SUBCOMMANDS:
    add                Add data to our savings spreadsheet
    converse           Parse and converse into other currencies
    help               Prints this message or the help of the given subcommand(s)
    rolling-average    Calculate averages
    show               Parse our saving spreadsheet and display data
```
