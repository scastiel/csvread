use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::multispace0;
use nom::combinator::all_consuming;
use nom::sequence::tuple;
use nom::IResult;

pub type ParsingError<'a> = nom::Err<nom::error::Error<&'a str>>;

#[derive(Debug, PartialEq, Clone)]
pub struct SelectFields(pub Vec<String>);

impl SelectFields {
  pub fn parse(select: &str) -> Result<Self, ParsingError> {
    match all_consuming(Self::parse_select)(&select) {
      Ok((_, query)) => Ok(query),
      Err(err) => Err(err),
    }
  }

  fn parse_select(input: &str) -> IResult<&str, Self> {
    return alt((Self::parse_several_fields, Self::parse_one_field))(input);
  }

  fn parse_several_fields(input: &str) -> IResult<&str, Self> {
    let (input, (field, _, _, mut fields)) =
      tuple((Self::parse_field, multispace0, tag(","), Self::parse_select))(input)?;
    let mut new_fields = vec![String::from(field)];
    new_fields.append(&mut fields.0);
    return Ok((input, SelectFields(new_fields)));
  }

  fn parse_one_field(input: &str) -> IResult<&str, Self> {
    let (input, field) = Self::parse_field(input)?;
    return Ok((input, SelectFields(vec![String::from(field)])));
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
      take_while(|c| c != ' ' && c != ','),
      multispace0,
    ))(input)?;
    return Ok((input, field));
  }

  fn parse_field_with_brackets(input: &str) -> IResult<&str, &str> {
    let (input, (_, _, field, _, _)) = tuple((
      multispace0,
      tag("["),
      take_while(|c| c != ']'),
      tag("]"),
      multispace0,
    ))(input)?;
    return Ok((input, field));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_parses_a_unique_field_without_brackets() {
    assert_eq!(
      Ok(SelectFields(vec![String::from("field1")])),
      SelectFields::parse("field1")
    );
  }

  #[test]
  fn it_parses_a_unique_field_with_brackets() {
    assert_eq!(
      Ok(SelectFields(vec![String::from("field 1")])),
      SelectFields::parse("[field 1]")
    );
  }

  #[test]
  fn it_parses_several_fields_without_brackets() {
    assert_eq!(
      Ok(SelectFields(vec![
        String::from("field1"),
        String::from("field2"),
        String::from("field3"),
        String::from("field4")
      ])),
      SelectFields::parse("  field1, field2,field3  ,  field4  ")
    );
  }

  #[test]
  fn it_parses_several_fields_with_brackets() {
    assert_eq!(
      Ok(SelectFields(vec![
        String::from("field 1"),
        String::from("field2"),
        String::from("field 3"),
        String::from("field4")
      ])),
      SelectFields::parse("  [field 1], field2,[field 3]  ,  field4  ")
    );
  }
}
