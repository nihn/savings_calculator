use crate::parse::Records;
use chrono::{Date, Duration, NaiveDate, TimeZone, Utc};
use clap::arg_enum;
use float_ord::FloatOrd;
use plotters::coord::IntoMonthly;
use plotters::prelude::*;
use prettytable::{cell, format, Cell, Row, Table};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io;
use tempfile::Builder;
use webbrowser;

arg_enum! {
    #[derive(Debug)]
    pub enum Format {
        Table,
        Graph,
    }
}

static COLORS: [RGBColor; 6] = [RED, BLACK, YELLOW, BLUE, CYAN, MAGENTA];

pub fn present_results(records: Records, format: Format) {
    match format {
        Format::Table => print_table(records),
        Format::Graph => plot_graph(records),
    }
}

pub fn print_table(records: Records) {
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
    table.printstd();
}

fn to_date(date: &NaiveDate) -> Date<Utc> {
    Utc.from_local_date(date).unwrap()
}

pub fn plot_graph(records: Records) {
    let file = Builder::new().suffix(".png").tempfile().unwrap();
    let root = BitMapBackend::new(file.path().into(), (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let offset = Duration::weeks(4);
    let start: Date<_> = to_date(&records.records[0].date) - offset;
    let end: Date<_> = to_date(&records.records[records.records.len() - 1].date) + offset;
    let max_val = records
        .records
        .iter()
        .map(|r| r.savings.iter().map(|s| FloatOrd(*s)).max().unwrap())
        .max()
        .unwrap()
        .0;

    let min_val = records
        .records
        .iter()
        .map(|r| r.savings.iter().map(|s| FloatOrd(*s)).min().unwrap())
        .min()
        .unwrap()
        .0;
    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .set_label_area_size(LabelAreaPosition::Left, (5i32).percent_width())
        .set_label_area_size(LabelAreaPosition::Bottom, (10i32).percent_height())
        .build_ranged((start..end).monthly(), min_val as f64..max_val as f64)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    let mut series: Vec<Vec<(Date<Utc>, f64)>> =
        records.currencies.iter().map(|_| Vec::new()).collect();
    for record in records.records {
        for (i, saving) in record.savings.iter().enumerate() {
            series[i].push((to_date(&record.date), *saving as f64));
        }
    }

    for (i, s) in series.into_iter().enumerate() {
        let color = &COLORS[i % COLORS.len()];
        chart
            .draw_series(LineSeries::new(s, color))
            .unwrap()
            .label(records.currencies[i].to_string())
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color)
            });
    }
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
    root.present().unwrap();
    webbrowser::open(format!("file://{}", file.path().to_str().unwrap()).as_str()).unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}
