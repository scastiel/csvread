use crate::errors::AppError;
use crate::query_parser::Query;
use crate::Args;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use tabular::{Row, Table};

pub fn run(args: &Args, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
  let mut reader = csv::ReaderBuilder::new()
    .has_headers(true)
    .from_path(&args.filename)?;

  let headers = reader.headers()?;
  let header_positions = header_positions(&headers);

  let query = args.parse_query()?;
  let mut table = Table::new(&row_spec(&headers));
  table.add_row(headers_row(&headers));
  for record in reader.records() {
    let record = record?;
    if !should_display_record(&record, &query, &header_positions)? {
      continue;
    }
    table.add_row(row_for_record(&record));
  }

  write!(writer, "{}", table)?;
  Ok(())
}

fn header_positions(headers: &csv::StringRecord) -> HashMap<String, usize> {
  let mut header_positions: HashMap<String, usize> = HashMap::new();
  for (i, header) in headers.iter().enumerate() {
    header_positions.insert(String::from(header), i);
  }
  header_positions
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

#[cfg(test)]
mod tests {
  use super::*;

  fn get_output(where_: Option<String>) -> Result<String, Box<dyn Error>> {
    let args = Args {
      filename: String::from("example_data/weather.csv"),
      where_,
    };
    let mut out = Vec::new();
    run(&args, &mut out)?;

    Ok(String::from(String::from_utf8(out)?.trim()))
  }

  #[test]
  fn without_filter() -> Result<(), Box<dyn Error>> {
    let out = get_output(None)?;
    assert_eq!(16744, out.lines().count());
    Ok(())
  }

  #[test]
  fn with_one_filter() -> Result<(), Box<dyn Error>> {
    let out = get_output(Some(String::from("[Data.Temperature.Avg Temp] = '-21'")))?;
    assert_eq!("
Data.Precipitation Date.Full  Date.Month Date.Week of Date.Year Station.City Station.Code Station.Location Station.State Data.Temperature.Avg Temp Data.Temperature.Max Temp Data.Temperature.Min Temp Data.Wind.Direction Data.Wind.Speed
0.0                2016-12-11 12         11           2016      Fairbanks    FAI          Fairbanks, AK    Alaska        -21                       -16                       -27                       3                   2.57
0.0                2016-12-18 12         18           2016      Northway     ORT          Northway, AK     Alaska        -21                       -15                       -28                       16                  0.2
    ".trim(), out);
    Ok(())
  }

  #[test]
  fn with_two_filters_with_or() -> Result<(), Box<dyn Error>> {
    let out = get_output(Some(String::from(
      "[Data.Temperature.Avg Temp] = '-18' or [Data.Temperature.Max Temp] = '-11'",
    )))?;
    assert_eq!("
Data.Precipitation Date.Full  Date.Month Date.Week of Date.Year Station.City Station.Code Station.Location Station.State Data.Temperature.Avg Temp Data.Temperature.Max Temp Data.Temperature.Min Temp Data.Wind.Direction Data.Wind.Speed
0.0                2016-12-04 12         4            2016      Mc Grath     MCG          Mc Grath, AK     Alaska        -19                       -11                       -26                       29                  2.87
0.0                2016-12-04 12         4            2016      Tanana       TAL          Tanana, AK       Alaska        -18                       -10                       -26                       20                  1.9
0.0                2016-12-11 12         11           2016      Bettles      BTT          Bettles, AK      Alaska        -19                       -11                       -27                       30                  3.28
0.47               2016-12-11 12         11           2016      Gulkana      GKN          Gulkana, AK      Alaska        -20                       -11                       -28                       33                  0.85
0.0                2016-12-11 12         11           2016      Mc Grath     MCG          Mc Grath, AK     Alaska        -18                       -11                       -24                       16                  3.43
0.0                2016-12-11 12         11           2016      Northway     ORT          Northway, AK     Alaska        -18                       -10                       -26                       15                  0.62
    ".trim(), out);
    Ok(())
  }

  #[test]
  fn with_two_filters_with_and() -> Result<(), Box<dyn Error>> {
    let out = get_output(Some(String::from(
      "[Data.Temperature.Avg Temp] = '-18' and [Data.Temperature.Max Temp] = '-11'",
    )))?;
    assert_eq!("
Data.Precipitation Date.Full  Date.Month Date.Week of Date.Year Station.City Station.Code Station.Location Station.State Data.Temperature.Avg Temp Data.Temperature.Max Temp Data.Temperature.Min Temp Data.Wind.Direction Data.Wind.Speed
0.0                2016-12-11 12         11           2016      Mc Grath     MCG          Mc Grath, AK     Alaska        -18                       -11                       -24                       16                  3.43
    ".trim(), out);
    Ok(())
  }

  #[test]
  fn with_mixed_filters() -> Result<(), Box<dyn Error>> {
    let out = get_output(Some(String::from(
      "[Data.Temperature.Avg Temp] = '-18' or [Data.Temperature.Max Temp] = '-11' and ([Data.Temperature.Min Temp] = '-28' or [Data.Temperature.Min Temp] = '-26')",
    )))?;
    assert_eq!("
Data.Precipitation Date.Full  Date.Month Date.Week of Date.Year Station.City Station.Code Station.Location Station.State Data.Temperature.Avg Temp Data.Temperature.Max Temp Data.Temperature.Min Temp Data.Wind.Direction Data.Wind.Speed
0.0                2016-12-04 12         4            2016      Mc Grath     MCG          Mc Grath, AK     Alaska        -19                       -11                       -26                       29                  2.87
0.0                2016-12-04 12         4            2016      Tanana       TAL          Tanana, AK       Alaska        -18                       -10                       -26                       20                  1.9
0.47               2016-12-11 12         11           2016      Gulkana      GKN          Gulkana, AK      Alaska        -20                       -11                       -28                       33                  0.85
0.0                2016-12-11 12         11           2016      Mc Grath     MCG          Mc Grath, AK     Alaska        -18                       -11                       -24                       16                  3.43
0.0                2016-12-11 12         11           2016      Northway     ORT          Northway, AK     Alaska        -18                       -10                       -26                       15                  0.62
    ".trim(), out);
    Ok(())
  }
}
