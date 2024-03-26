use std::env::args;
use std::mem::{transmute, size_of};
use std::{ops::Range, fs::File};
use std::error::Error;
use std::io::Write;

use rand::random;

const DEFAULT_ARRAY_SIZE: usize = 1000;
const DATA_SIZE: usize = 20;

#[repr(C)]
#[derive(Debug,Clone,Copy)]
struct Record 
{
  key: u32,
  value: [char; DATA_SIZE]
}

fn make_str() -> [char; DATA_SIZE]
{
  let mut ret = [' '; DATA_SIZE];
  for i in (Range { start: 0, end: DATA_SIZE })
  {
    ret[i] = ((random::<u8>() % 26) + ('a' as u8)) as char;
  }
  return ret;
}

fn main() -> Result<(), Box<dyn Error>> {
  let mut data: Vec<Record> = Vec::new();
  let mut accum: u32 = 0;

  let args: Vec<String> = args().collect();

  let array_size =
    if args.len() > 1
    {
      args[1].parse::<usize>().unwrap()
    } else {
      DEFAULT_ARRAY_SIZE
    };

  for _i in (Range { start: 0, end: array_size })
  {
    accum += (random::<u32>() % 100) + 1;
    data.push(Record {
      key: accum,
      value: make_str()
    })

  }

  let filename = format!("data_{}.dat", array_size);
  let mut file = File::create(&filename)?;

  println!("Generating {} records of size {} each = 0x{:x} -> {}", 
      array_size,
      size_of::<Record>(),
      size_of::<Record>(),
      &filename
    );

  for i in data
  {
    let buffer: [u8; size_of::<Record>()] = 
      unsafe { transmute::<Record, [u8; size_of::<Record>()]>(i) };

    // println!("{:?}", i);
    file.write_all(&buffer)?;
  }

  Ok(())
}
