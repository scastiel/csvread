# CSVRead – An efficient CSV reader for the console

CSVRead is a command line program that you can use to query big CSV files, instead of opening them in Excel for instance.

I developed this program for personal use (inspired by [querycsv.py](https://pythonhosted.org/querycsv/)), and to learn Rust.

Please consider it very WIP 😉

## TL;DR

```
$ csvread path/to/file.csv \
    --select "name, age, [date of birth]"
    --where "name = 'Joe' and (age = '34' or age = '35')"
```

## Installation

As the project is still work-in-progress, the installation is a pretty manual process, and you will need to [install Rust](https://www.rust-lang.org/tools/install) on your machine first.

```
$ git clone https://github.com/scastiel/csvread
$ cd csvread
$ cargo build
$ cargo install --path .
```

## Usage

```
USAGE:
    csvread [FLAGS] [OPTIONS] <filename>

ARGS:
    <filename>    The CSV file to read.

FLAGS:
    -c, --count      Counts the number of rows instead of displaying them.
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --select <select>    List of columns to display, e.g "Col1, [Column 2]".
    -w, --where <where>      Query to filter the data, e.g "[My column] = 'the value'".
```

## Features

### Display a CSV in the console

```
$ csvread examples/weather.csv
Data.Precipitation Date.Full  Date.Month Date.Week of  ...
0.0                2016-01-03 1          3             ...
0.0                2016-01-03 1          3             ...
0.16               2016-01-03 1          3             ...
0.0                2016-01-03 1          3             ...
```

_Tip: to display big file, you can pipe the command with the `less` command: `csvread file.csv | less -S`_

### Select the columns to display with `--select`

```
$ csvread example_data/weather.csv --select "Date.Full, [Station.City]"
Date.Full  Station.City
2016-02-07 Bettles
2016-02-14 Bettles
2016-02-21 Bettles
2016-02-28 Bettles
```

Separate the fields by commas, and use brackets `[]` for fields containing a space character or a comma, e.g. `field, [my field], [my field, again]`.

### Filter the rows with `--where`

```
$ csvread example_data/weather.csv --where "[Station.Location] = 'Bettles, AK' and [Date.Month] <> '1'"
Data.Precipitation Date.Full  Date.Month Date.Week of Date.Year Station.City
0.0                2016-02-07 2          7            2016      Bettles
0.24               2016-02-14 2          14           2016      Bettles
0.0                2016-02-21 2          21           2016      Bettles
0.0                2016-02-28 2          28           2016      Bettles
```

You can combine conditions using `and` and `or` logical operators, and parentheses `()` to group them.

The right operand needs to be surrounded by single quotes `'`, even for numbers.

The supported comparison operators are `=` (equality) and `<>` (difference).

### Count the rows with `--count` (instead of displaying them)

```
$ csvread example_data/weather.csv --count
16,743 rows
```

```
$ csvread example_data/weather.csv --where "[Station.Location] = 'Bettles, AK' and [Date.Month] <> '1'" --count
47 rows (16,743 total)
```

## LICENSE

MIT. See [LICENSE](./LICENSE).
