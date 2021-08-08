use crate::errors::AppError;
use crate::query_parser::Query;
use clap::Clap;
use std::collections::HashMap;
use std::error::Error;
use tabular::{Row, Table};

mod errors;
mod query_parser;

#[derive(Clap, Debug)]
#[clap(
  version = "0.1.0",
  author = "Sébastien Castiel <sebastien@castiel.me>",
  about = "Efficient CSV reader for the console."
)]
struct Args {
  #[clap(about = "The CSV file to read.")]
  filename: String,
  #[clap(
    short = 'w',
    long = "where",
    about = "Query to filter the data, e.g \"[My column] = 'the value'\"."
  )]
  where_: Option<String>,
}

impl Args {
  fn parse_query(&self) -> Result<Option<Query>, AppError> {
    match &self.where_ {
      Some(query) => match Query::parse(query) {
        Ok(query) => Ok(Some(query)),
        Err(_) => Err(AppError::WhereParsingError(query.clone())),
      },
      None => Ok(None),
    }
  }
}

fn main() {
  if let Err(err) = run() {
    eprintln!("{}", err);
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();

  let mut reader = csv::ReaderBuilder::new()
    .has_headers(true)
    .from_path(&args.filename)?;

  let headers = reader.headers()?;

  let mut header_positions: HashMap<String, usize> = HashMap::new();
  for (i, header) in headers.iter().enumerate() {
    header_positions.insert(String::from(header), i);
  }

  let mut table = Table::new(&row_spec(&headers));
  table.add_row(headers_row(&headers));

  let query = args.parse_query()?;

  for record in reader.records() {
    let record = record?;
    if !should_display_record(&record, &query, &header_positions)? {
      continue;
    }
    table.add_row(row_for_record(&record));
  }

  print!("{}", table);

  Ok(())
}

fn row_spec(headers: &csv::StringRecord) -> String {
  headers
    .into_iter()
    .map(|_| String::from("{:<}"))
    .collect::<Vec<String>>()
    .join(" ")
}

fn headers_row(headers: &csv::StringRecord) -> Row {
  let mut headers_row = Row::new();
  for header in headers {
    headers_row.add_cell(&header);
  }
  return headers_row;
}

fn row_for_record(record: &csv::StringRecord) -> Row {
  let mut row = Row::new();
  for cell in record.iter() {
    row.add_cell(cell);
  }
  return row;
}

fn should_display_record(
  record: &csv::StringRecord,
  query: &Option<Query>,
  header_positions: &HashMap<String, usize>,
) -> Result<bool, AppError> {
  match query {
    Some(Query::Comparison(field, value)) => match header_positions.get(field) {
      Some(&col_pos) => return Ok(record.get(col_pos).unwrap() == value),
      None => return Err(AppError::InvalidFieldInWhereClause(field.clone())),
    },
    Some(Query::OrCombination(left, right)) => Ok(
      should_display_record(&record, &Some(*left.clone()), &header_positions)?
        || should_display_record(&record, &Some(*right.clone()), &header_positions)?,
    ),
    Some(Query::AndCombination(left, right)) => Ok(
      should_display_record(&record, &Some(*left.clone()), &header_positions)?
        && should_display_record(&record, &Some(*right.clone()), &header_positions)?,
    ),
    _ => Ok(true),
  }
}
