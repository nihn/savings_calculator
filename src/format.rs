use crate::parse::Records;
use prettytable::{cell, format, Cell, Row, Table};

pub fn format_table(records: Records) -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    let mut titles = vec![cell!("Date")];
    titles.extend(
        records
            .currencies
            .iter()
            .map(|c| Cell::new(c.to_string().as_str())),
    );
    table.set_titles(Row::new(titles));
    for record in records.records {
        let mut cells = vec![cell!(record.date)];
        cells.extend(
            record
                .savings
                .iter()
                .map(|c| Cell::new(format!("{:.2}", c).as_str())),
        );
        table.add_row(Row::new(cells));
    }
    table
}
