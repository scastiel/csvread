use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppError {
  SelectParsingError(String),
  WhereParsingError(String),
  InvalidFieldInWhereClause(String),
  InvalidFieldInSelectClause(String),
}

impl Display for AppError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
    match &self {
      AppError::SelectParsingError(clause) => {
        formatter.write_fmt(format_args!("Error parsing the select clause: {}", clause))
      }
      AppError::WhereParsingError(clause) => {
        formatter.write_fmt(format_args!("Error parsing the where clause: {}", clause))
      }
      AppError::InvalidFieldInSelectClause(field) => {
        formatter.write_fmt(format_args!("Invalid field in select clause: {}.", field))
      }
      AppError::InvalidFieldInWhereClause(field) => {
        formatter.write_fmt(format_args!("Invalid field in where clause: {}.", field))
      }
    }
  }
}

impl Error for AppError {}
