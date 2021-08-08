use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::sequence::tuple;
use nom::IResult;

pub type ParsingError<'a> = nom::Err<nom::error::Error<&'a str>>;

#[derive(Debug, PartialEq, Clone)]
pub enum Query {
  Comparison(String, String),
  Combination(Box<Query>, Box<Query>),
}

impl Query {
  pub fn parse(query: &str) -> Result<Query, ParsingError> {
    match Self::parse_query(&query) {
      Ok((_, query)) => Ok(query),
      Err(err) => Err(err),
    }
  }

  fn parse_query(input: &str) -> IResult<&str, Query> {
    return alt((Self::parse_combination, Self::parse_comparison))(input);
  }

  fn parse_combination(input: &str) -> IResult<&str, Query> {
    let space = take_while(|c| c == ' ');
    let or = tag("or");
    let (input, (left, _, _, _, right)) = tuple((
      Self::parse_comparison,
      &space,
      &or,
      &space,
      Self::parse_comparison,
    ))(input)?;
    Ok((input, Query::Combination(Box::new(left), Box::new(right))))
  }

  fn parse_comparison(input: &str) -> IResult<&str, Query> {
    let space = take_while(|c| c == ' ');
    let equals = tag("=");
    let quote = tag("'");
    let value = take_while(|c| c != '\'');
    let (input, (field, _, _, _, value, _, _)) = tuple((
      Self::parse_field,
      equals,
      &space,
      &quote,
      value,
      &quote,
      &space,
    ))(input)?;
    Ok((
      input,
      Query::Comparison(String::from(field), String::from(value)),
    ))
  }

  fn parse_field(input: &str) -> IResult<&str, &str> {
    return alt((
      Self::parse_field_with_brackets,
      Self::parse_field_without_brackets,
    ))(input);
  }

  fn parse_field_without_brackets(input: &str) -> IResult<&str, &str> {
    let space = take_while(|c| c == ' ');
    let field = take_while1(|c| c != ' ' && c != '=');
    let (input, (_, field, _)) = tuple((&space, &field, &space))(input)?;
    Ok((input, field))
  }

  fn parse_field_with_brackets(input: &str) -> IResult<&str, &str> {
    let space = take_while(|c| c == ' ');
    let field = take_while1(|c| c != ']');
    let (input, (_, _, field, _, _)) = tuple((&space, tag("["), &field, tag("]"), &space))(input)?;
    Ok((input, field))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn comparison_with_spaces_inside() {
    assert_eq!(
      Ok(Query::Comparison(
        String::from("my_field"),
        String::from("my_value")
      )),
      Query::parse("my_field = 'my_value'")
    );
  }

  #[test]
  fn comparison_with_no_spaces_inside() {
    assert_eq!(
      Ok(Query::Comparison(
        String::from("my_field"),
        String::from("my_value")
      )),
      Query::parse("my_field='my_value'")
    );
  }

  #[test]
  fn comparison_with_spaces_outside() {
    assert_eq!(
      Ok(Query::Comparison(
        String::from("my_field"),
        String::from("my_value")
      )),
      Query::parse("  my_field='my_value'  ")
    );
  }

  #[test]
  fn with_brackets_in_field() {
    assert_eq!(
      Ok(Query::Comparison(
        String::from("my field"),
        String::from("my_value")
      )),
      Query::parse("[my field] = 'my_value'")
    );
  }

  #[test]
  fn simple_or_combination() {
    assert_eq!(
      Ok(Query::Combination(
        Box::new(Query::Comparison(
          String::from("my field"),
          String::from("my_value")
        )),
        Box::new(Query::Comparison(
          String::from("my field"),
          String::from("other value")
        )),
      )),
      Query::parse("[my field] = 'my_value' or [my field] = 'other value'")
    );
  }
}
