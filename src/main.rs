use clap::Clap;

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

fn main() {
  let args = Args::parse();
  println!("{:?}", args);
}
