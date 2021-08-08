use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::multispace0;
use nom::combinator::all_consuming;
use nom::sequence::tuple;
use nom::IResult;

pub type ParsingError<'a> = nom::Err<nom::error::Error<&'a str>>;

#[derive(Debug, PartialEq, Clone)]
pub enum Query {
  Equality(String, String),
  Difference(String, String),
  OrCombination(Box<Query>, Box<Query>),
  AndCombination(Box<Query>, Box<Query>),
}

impl Query {
  pub fn parse(query: &str) -> Result<Query, ParsingError> {
    match all_consuming(Self::parse_query)(&query) {
      Ok((_, query)) => Ok(query),
      Err(err) => Err(err),
    }
  }

  fn parse_query(input: &str) -> IResult<&str, Query> {
    return alt((
      Self::parse_or_combination,
      Self::parse_and_combination,
      Self::parse_comparison,
    ))(input);
  }

  fn parse_parentheses_query(input: &str) -> IResult<&str, Query> {
    let (input, (_, query, _)) = tuple((tag("("), Self::parse_query, tag(")")))(input)?;
    Ok((input, query))
  }

  fn parse_or_combination(input: &str) -> IResult<&str, Query> {
    let (input, (left, _, _, _, right)) = tuple((
      alt((
        Self::parse_and_combination,
        Self::parse_comparison,
        Self::parse_parentheses_query,
      )),
      multispace0,
      tag("or"),
      multispace0,
      alt((Self::parse_query, Self::parse_parentheses_query)),
    ))(input)?;
    Ok((input, Query::OrCombination(Box::new(left), Box::new(right))))
  }

  fn parse_and_combination(input: &str) -> IResult<&str, Query> {
    let (input, (left, _, _, _, right)) = tuple((
      alt((Self::parse_comparison, Self::parse_parentheses_query)),
      multispace0,
      tag("and"),
      multispace0,
      alt((
        Self::parse_and_combination,
        Self::parse_comparison,
        Self::parse_parentheses_query,
      )),
    ))(input)?;
    Ok((
      input,
      Query::AndCombination(Box::new(left), Box::new(right)),
    ))
  }

  fn parse_comparison(input: &str) -> IResult<&str, Query> {
    let (input, (field, op, _, _, value, _, _)) = tuple((
      Self::parse_field,
      alt((tag("="), tag("<>"))),
      multispace0,
      tag("'"),
      take_while(|c| c != '\''),
      tag("'"),
      multispace0,
    ))(input)?;
    match op {
      "=" => Ok((
        input,
        Query::Equality(String::from(field), String::from(value)),
      )),
      _ => Ok((
        input,
        Query::Difference(String::from(field), String::from(value)),
      )),
    }
  }

  fn parse_field(input: &str) -> IResult<&str, &str> {
    return alt((
      Self::parse_field_with_brackets,
      Self::parse_field_without_brackets,
    ))(input);
  }

  fn parse_field_without_brackets(input: &str) -> IResult<&str, &str> {
    let (input, (_, field, _)) = tuple((
      multispace0,
      take_while1(|c| c != ' ' && c != '='),
      multispace0,
    ))(input)?;
    Ok((input, field))
  }

  fn parse_field_with_brackets(input: &str) -> IResult<&str, &str> {
    let (input, (_, _, field, _, _)) = tuple((
      multispace0,
      tag("["),
      take_while1(|c| c != ']'),
      tag("]"),
      multispace0,
    ))(input)?;
    Ok((input, field))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn equality_with_spaces_inside() {
    assert_eq!(
      Ok(Query::Equality(
        String::from("my_field"),
        String::from("my_value")
      )),
      Query::parse("my_field = 'my_value'")
    );
  }

  #[test]
  fn difference_with_spaces_inside() {
    assert_eq!(
      Ok(Query::Difference(
        String::from("my_field"),
        String::from("my_value")
      )),
      Query::parse("my_field <> 'my_value'")
    );
  }

  #[test]
  fn equality_with_no_spaces_inside() {
    assert_eq!(
      Ok(Query::Equality(
        String::from("my_field"),
        String::from("my_value")
      )),
      Query::parse("my_field='my_value'")
    );
  }

  #[test]
  fn equality_with_spaces_outside() {
    assert_eq!(
      Ok(Query::Equality(
        String::from("my_field"),
        String::from("my_value")
      )),
      Query::parse("  my_field='my_value'  ")
    );
  }

  #[test]
  fn with_brackets_in_field() {
    assert_eq!(
      Ok(Query::Equality(
        String::from("my field"),
        String::from("my_value")
      )),
      Query::parse("[my field] = 'my_value'")
    );
  }

  #[test]
  fn simple_or_combination() {
    assert_eq!(
      Ok(Query::OrCombination(
        Box::new(Query::Equality(
          String::from("my field"),
          String::from("my_value")
        )),
        Box::new(Query::Equality(
          String::from("my field"),
          String::from("other value")
        )),
      )),
      Query::parse("[my field] = 'my_value' or [my field] = 'other value'")
    );
  }

  #[test]
  fn simple_and_combination() {
    assert_eq!(
      Ok(Query::AndCombination(
        Box::new(Query::Equality(
          String::from("my field"),
          String::from("my_value")
        )),
        Box::new(Query::Equality(
          String::from("my field"),
          String::from("other value")
        )),
      )),
      Query::parse("[my field] = 'my_value' and [my field] = 'other value'")
    );
  }

  #[test]
  fn two_or_combination() {
    assert_eq!(
      Ok(Query::OrCombination(
        Box::new(Query::Equality(
          String::from("my field"),
          String::from("my_value")
        )),
        Box::new(
          Query::OrCombination(
            Box::new(Query::Equality(
              String::from("my field"),
              String::from("other value")
            )),
            Box::new(Query::Equality(
              String::from("my other field"),
              String::from("another value")
            ))
          )
        ),
      )),
      Query::parse("[my field] = 'my_value' or [my field] = 'other value' or [my other field] = 'another value'")
    );
  }

  #[test]
  fn mixed_combination() {
    assert_eq!(
      Ok(Query::OrCombination(
        Box::new(Query::Equality(
          String::from("my field"),
          String::from("my_value")
        )),
        Box::new(
          Query::AndCombination(
            Box::new(Query::Equality(
              String::from("my field"),
              String::from("other value")
            )),
            Box::new(Query::Equality(
              String::from("my other field"),
              String::from("another value")
            ))
          )
        ),
      )),
      Query::parse("[my field] = 'my_value' or [my field] = 'other value' and [my other field] = 'another value'")
    );
  }

  #[test]
  fn mixed_combination_with_priority() {
    assert_eq!(
      Ok(Query::OrCombination(
        Box::new(
          Query::AndCombination(
            Box::new(Query::Equality(
              String::from("my field"),
              String::from("my_value")
            )),
            Box::new(Query::Equality(
              String::from("my field"),
              String::from("other value")
            )),
          )
        ),
        Box::new(Query::Equality(
          String::from("my other field"),
          String::from("another value")
        )),
      )),
      Query::parse("[my field] = 'my_value' and [my field] = 'other value' or [my other field] = 'another value'")
    );
  }

  #[test]
  fn two_mixed_combinations_with_priority() {
    assert_eq!(
      Ok(Query::OrCombination(
        Box::new(
          Query::AndCombination(
            Box::new(Query::Equality(
              String::from("my field"),
              String::from("my_value")
            )),
            Box::new(Query::Equality(
              String::from("my field"),
              String::from("other value")
            )),
          )
        ),
        Box::new(
          Query::AndCombination(
            Box::new(Query::Equality(
              String::from("my other field"),
              String::from("another value")
            )),
            Box::new(Query::Equality(
              String::from("last field"),
              String::from("v")
            )),
          )
        ),
      )),
      Query::parse("[my field] = 'my_value' and [my field] = 'other value' or [my other field] = 'another value' and [last field] = 'v'")
    );
  }

  #[test]
  fn mixed_combinations_with_parentheses() {
    assert_eq!(
      Ok(Query::OrCombination(
        Box::new(Query::AndCombination(
          Box::new(Query::Equality(String::from("my field"), String::from("my_value"))),
          Box::new(Query::AndCombination(
            Box::new(Query::OrCombination(
              Box::new(Query::Equality(String::from("my field"), String::from("other value"))),
              Box::new(Query::Equality(String::from("my other field"), String::from("another value"))),
            )),
            Box::new(Query::Equality(String::from("last field"), String::from("v"))),
          ))
        )),
        Box::new(Query::Equality(
          String::from("last field"),
          String::from("last value"),
        ))
      )),
      Query::parse("[my field] = 'my_value' and ([my field] = 'other value' or [my other field] = 'another value') and [last field] = 'v' or [last field] = 'last value'")
    );
  }
}
