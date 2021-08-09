use crate::errors::AppError;
use crate::query_parser::Query;
use crate::select_parser::SelectFields;
use clap::Clap;

#[derive(Clap, Debug)]
#[clap(
  version = "0.1.0",
  author = "Sébastien Castiel <sebastien@castiel.me>",
  about = "Efficient CSV reader for the console."
)]
pub struct Args {
  #[clap(about = "The CSV file to read.")]
  pub filename: String,
  #[clap(
    short = 's',
    long = "select",
    about = "List of columns to display, e.g \"Col1, [Column 2]\"."
  )]
  pub select: Option<String>,
  #[clap(
    short = 'w',
    long = "where",
    about = "Query to filter the data, e.g \"[My column] = 'the value'\"."
  )]
  pub where_: Option<String>,
  #[clap(
    short = 'c',
    long = "count",
    about = "Counts the number of rows instead of displaying them."
  )]
  pub count: bool,
}

impl Args {
  pub fn parse_query(&self) -> Result<Option<Query>, AppError> {
    match &self.where_ {
      Some(query) => match Query::parse(query) {
        Ok(query) => Ok(Some(query)),
        Err(_) => Err(AppError::WhereParsingError(query.clone())),
      },
      None => Ok(None),
    }
  }

  pub fn parse_select(&self) -> Result<Option<SelectFields>, AppError> {
    match &self.select {
      Some(select) => match SelectFields::parse(select) {
        Ok(select) => Ok(Some(select)),
        Err(_) => Err(AppError::SelectParsingError(select.clone())),
      },
      None => Ok(None),
    }
  }
}
