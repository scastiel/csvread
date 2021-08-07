use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppError {
  WhereParsingError(String),
  InvalidFieldInWhereClause(String),
}

impl Display for AppError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
    match &self {
      AppError::WhereParsingError(clause) => {
        formatter.write_fmt(format_args!("Error parsing the where clause: {}", clause))
      }
      AppError::InvalidFieldInWhereClause(field) => {
        formatter.write_fmt(format_args!("Invalid field in where clause: {}.", field))
      }
    }
  }
}

impl Error for AppError {}
