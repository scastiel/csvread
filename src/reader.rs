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

  let headers_to_display: Vec<String> = match args.parse_select()? {
    Some(headers) => headers.0,
    None => reader.headers()?.into_iter().map(String::from).collect(),
  };

  let query = args.parse_query()?;

  if args.count {
    let mut total = 0;
    let mut filtered = 0;
    for record in reader.records() {
      let record = record?;
      if should_display_record(&record, &query, &header_positions)? {
        filtered += 1;
      }
      total += 1;
    }
    if args.where_.is_some() {
      writeln!(writer, "{} rows ({} total)", filtered, total)?;
    } else {
      writeln!(writer, "{} rows", total)?;
    }
  } else {
    let mut table = Table::new(&row_spec(&headers_to_display));
    table.add_row(headers_row(&headers_to_display));
    for record in reader.records() {
      let record = record?;
      if !should_display_record(&record, &query, &header_positions)? {
        continue;
      }
      table.add_row(row_for_record(
        &record,
        &header_positions,
        &headers_to_display,
      )?);
    }
    write!(writer, "{}", table)?;
  }

  Ok(())
}

fn header_positions(headers: &csv::StringRecord) -> HashMap<String, usize> {
  let mut header_positions: HashMap<String, usize> = HashMap::new();
  for (i, header) in headers.iter().enumerate() {
    header_positions.insert(String::from(header), i);
  }
  header_positions
}

fn row_spec(headers: &Vec<String>) -> String {
  headers
    .into_iter()
    .map(|_| String::from("{:<}"))
    .collect::<Vec<String>>()
    .join(" ")
}

fn headers_row(headers: &Vec<String>) -> Row {
  let mut headers_row = Row::new();
  for header in headers {
    headers_row.add_cell(&header);
  }
  return headers_row;
}

fn row_for_record(
  record: &csv::StringRecord,
  header_positions: &HashMap<String, usize>,
  headers_to_display: &Vec<String>,
) -> Result<Row, AppError> {
  let mut row = Row::new();
  for header in headers_to_display {
    match header_positions.get(header) {
      Some(&header_pos) => row.add_cell(record.get(header_pos).unwrap()),
      None => return Err(AppError::InvalidFieldInSelectClause(header.clone())),
    };
  }
  return Ok(row);
}

fn should_display_record(
  record: &csv::StringRecord,
  query: &Option<Query>,
  header_positions: &HashMap<String, usize>,
) -> Result<bool, AppError> {
  match query {
    Some(Query::Equality(field, value)) => match header_positions.get(field) {
      Some(&col_pos) => return Ok(record.get(col_pos).unwrap() == value),
      None => return Err(AppError::InvalidFieldInWhereClause(field.clone())),
    },
    Some(Query::Difference(field, value)) => match header_positions.get(field) {
      Some(&col_pos) => return Ok(record.get(col_pos).unwrap() != value),
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

  fn get_output(
    select: Option<String>,
    where_: Option<String>,
    count: bool,
  ) -> Result<String, Box<dyn Error>> {
    let args = Args {
      filename: String::from("example_data/weather.csv"),
      select,
      where_,
      count,
    };
    let mut out = Vec::new();
    run(&args, &mut out)?;

    Ok(String::from(String::from_utf8(out)?.trim()))
  }

  #[test]
  fn without_filter() -> Result<(), Box<dyn Error>> {
    let out = get_output(None, None, false)?;
    assert_eq!(16744, out.lines().count());
    Ok(())
  }

  #[test]
  fn with_one_filter() -> Result<(), Box<dyn Error>> {
    let out = get_output(
      None,
      Some(String::from("[Data.Temperature.Avg Temp] = '-21'")),
      false,
    )?;
    assert_eq!("
Data.Precipitation Date.Full  Date.Month Date.Week of Date.Year Station.City Station.Code Station.Location Station.State Data.Temperature.Avg Temp Data.Temperature.Max Temp Data.Temperature.Min Temp Data.Wind.Direction Data.Wind.Speed
0.0                2016-12-11 12         11           2016      Fairbanks    FAI          Fairbanks, AK    Alaska        -21                       -16                       -27                       3                   2.57
0.0                2016-12-18 12         18           2016      Northway     ORT          Northway, AK     Alaska        -21                       -15                       -28                       16                  0.2
    ".trim(), out);
    Ok(())
  }

  #[test]
  fn with_one_filter_and_select_two_columns() -> Result<(), Box<dyn Error>> {
    let out = get_output(
      Some(String::from(
        "Date.Full, Station.City, [Data.Temperature.Avg Temp]",
      )),
      Some(String::from("[Data.Temperature.Avg Temp] = '-21'")),
      false,
    )?;
    assert_eq!(
      "
Date.Full  Station.City Data.Temperature.Avg Temp
2016-12-11 Fairbanks    -21
2016-12-18 Northway     -21
      "
      .trim(),
      out
    );
    Ok(())
  }

  #[test]
  fn with_two_filters_with_or() -> Result<(), Box<dyn Error>> {
    let out = get_output(
      None,
      Some(String::from(
        "[Data.Temperature.Avg Temp] = '-18' or [Data.Temperature.Max Temp] = '-11'",
      )),
      false,
    )?;
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
    let out = get_output(
      None,
      Some(String::from(
        "[Data.Temperature.Avg Temp] = '-18' and [Data.Temperature.Max Temp] = '-11'",
      )),
      false,
    )?;
    assert_eq!("
Data.Precipitation Date.Full  Date.Month Date.Week of Date.Year Station.City Station.Code Station.Location Station.State Data.Temperature.Avg Temp Data.Temperature.Max Temp Data.Temperature.Min Temp Data.Wind.Direction Data.Wind.Speed
0.0                2016-12-11 12         11           2016      Mc Grath     MCG          Mc Grath, AK     Alaska        -18                       -11                       -24                       16                  3.43
    ".trim(), out);
    Ok(())
  }

  #[test]
  fn with_mixed_filters() -> Result<(), Box<dyn Error>> {
    let out = get_output(None, Some(String::from(
      "[Data.Temperature.Avg Temp] = '-18' or [Data.Temperature.Max Temp] = '-11' and ([Data.Temperature.Min Temp] = '-28' or [Data.Temperature.Min Temp] = '-26')",
    )), false)?;
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

  #[test]
  fn with_other_mixed_filters() -> Result<(), Box<dyn Error>> {
    let out = get_output(None, Some(String::from(
      "[Data.Temperature.Avg Temp] = '-18' or [Data.Temperature.Max Temp] = '-11' and ([Data.Temperature.Min Temp] <> '-28' and [Data.Temperature.Min Temp] <> '-26')",
    )), false)?;
    assert_eq!("
Data.Precipitation Date.Full  Date.Month Date.Week of Date.Year Station.City Station.Code Station.Location Station.State Data.Temperature.Avg Temp Data.Temperature.Max Temp Data.Temperature.Min Temp Data.Wind.Direction Data.Wind.Speed
0.0                2016-12-04 12         4            2016      Tanana       TAL          Tanana, AK       Alaska        -18                       -10                       -26                       20                  1.9
0.0                2016-12-11 12         11           2016      Bettles      BTT          Bettles, AK      Alaska        -19                       -11                       -27                       30                  3.28
0.0                2016-12-11 12         11           2016      Mc Grath     MCG          Mc Grath, AK     Alaska        -18                       -11                       -24                       16                  3.43
0.0                2016-12-11 12         11           2016      Northway     ORT          Northway, AK     Alaska        -18                       -10                       -26                       15                  0.62
    ".trim(), out);
    Ok(())
  }

  #[test]
  fn with_count() -> Result<(), Box<dyn Error>> {
    let out = get_output(None, None, true)?;
    assert_eq!("16743 rows", out);
    Ok(())
  }

  #[test]
  fn with_count_and_where() -> Result<(), Box<dyn Error>> {
    let out = get_output(None, Some(String::from(
      "[Data.Temperature.Avg Temp] = '-18' or [Data.Temperature.Max Temp] = '-11' and ([Data.Temperature.Min Temp] <> '-28' and [Data.Temperature.Min Temp] <> '-26')",
    )), true)?;
    assert_eq!("4 rows (16743 total)", out);
    Ok(())
  }
}
