use clap::Clap;
use std::error::Error;
use tabular::{Row, Table};

#[derive(Clap, Debug)]
#[clap(
  version = "0.1.0",
  author = "Sébastien Castiel <sebastien@castiel.me>",
  about = "Efficient CSV reader for the console."
)]
struct Args {
  #[clap(about = "The CSV file to read.")]
  filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();

  let mut reader = csv::ReaderBuilder::new()
    .has_headers(true)
    .from_path(args.filename)?;
  let headers = reader.headers()?;

  let row_spec = headers
    .iter()
    .map(|_| String::from("{:<}"))
    .collect::<Vec<String>>()
    .join(" ");
  let mut table = Table::new(&row_spec);

  let mut header_row = Row::new();
  for header in headers {
    header_row.add_cell(&header);
  }
  table.add_row(header_row);

  for record in reader.records() {
    let record = record?;
    let mut row = Row::new();
    for cell in record.iter() {
      row.add_cell(cell);
    }
    table.add_row(row);
  }

  print!("{}", table);

  Ok(())
}
