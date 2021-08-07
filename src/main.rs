use clap::Clap;
use std::error::Error;

#[derive(Clap, Debug)]
#[clap(
  version = "0.1.0",
  author = "Sébastien Castiel <sebastien@castiel.me>",
  about = "Efficient CSV reader for the console."
)]
struct Args {
  #[clap(about = "The CSV file to read.")]
  filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();
  println!("{:?}", args);

  let mut reader = csv::ReaderBuilder::new()
    .has_headers(true)
    .from_path(args.filename)?;
  println!("Headers: {:?}", reader.headers()?);
  for record in reader.records() {
    println!("{:?}", record?)
  }

  Ok(())
}
