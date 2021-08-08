use crate::args::Args;
use crate::reader::run;
use clap::Clap;

mod args;
mod errors;
mod query_parser;
mod reader;

fn main() {
  let args = Args::parse();
  match run(&args, &mut std::io::stdout()) {
    Ok(()) => (),
    Err(err) => eprintln!("{}", err),
  }
}
