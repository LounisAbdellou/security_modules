use std::collections::VecDeque;

use crate::scrapper::Scrapper;

mod scrapper;

fn main() {
  let mut scrapper = Scrapper::new();
  let arguments = std::env::args().skip(1).collect::<VecDeque<String>>();

  if arguments.len() < 1 {
    println!("Error: Wrong number of arguments");
    std::process::exit(1)
  }

  scrapper.run(arguments);
}
