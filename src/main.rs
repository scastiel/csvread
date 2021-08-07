use crate::errors::AppError;
use crate::query_parser::Query;
use clap::Clap;
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

struct FieldComparisonWithColumnIndex {
  col_pos: usize,
  value: String,
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

  let field_comp = get_field_comp(&args, &headers)?;

  let mut table = Table::new(&row_spec(&headers));
  table.add_row(headers_row(&headers));

  for record in reader.records() {
    let record = record?;
    if !should_display_record(&record, &field_comp) {
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

fn get_field_comp(
  args: &Args,
  headers: &csv::StringRecord,
) -> Result<Option<FieldComparisonWithColumnIndex>, AppError> {
  if let Some(query) = args.parse_query()? {
    return Ok(Some(FieldComparisonWithColumnIndex {
      col_pos: find_col_pos(headers.into_iter(), &query.field)?,
      value: String::from(query.value),
    }));
  }
  return Ok(None);
}

fn should_display_record(
  record: &csv::StringRecord,
  field_comp: &Option<FieldComparisonWithColumnIndex>,
) -> bool {
  if let Some(FieldComparisonWithColumnIndex { col_pos, ref value }) = *field_comp {
    return record.get(col_pos).unwrap() == value;
  }
  return true;
}

fn find_col_pos(headers: csv::StringRecordIter, header: &str) -> Result<usize, AppError> {
  let mut i: usize = 0;
  for h in headers {
    if h == header {
      return Ok(i);
    }
    i += 1;
  }
  Err(AppError::InvalidFieldInWhereClause(String::from(header)))
}
