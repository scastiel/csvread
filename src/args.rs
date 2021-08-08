use crate::errors::AppError;
use crate::query_parser::Query;
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
    short = 'w',
    long = "where",
    about = "Query to filter the data, e.g \"[My column] = 'the value'\"."
  )]
  pub where_: Option<String>,
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
}
