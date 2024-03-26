mod data_file;
#[cfg(test)] mod tests;

use std::env::args; 
use std::error::Error;
use data_file::DataFile;

fn main() -> Result<(), Box<dyn Error>>
{
  let args: Vec<String> = args().collect();

  if args.len() != 3
  {
    println!("usage: ./binary_search data_file key");
    return Ok(())
  }

  let data_file: String = args[1].to_owned();
  let key = args[2].parse::<u32>()?;

  let mut file = DataFile::open(&data_file)?;

  println!("Searching {} records in [{}, {}]...", file.len(), file.min_key, file.max_key);

  match file.find(key)? 
  {
    Some(record) => { println!("{:?}", record) }
    None         => { println!("NOT FOUND") }
  }

  Ok(())
}
