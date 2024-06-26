use std::array;
use std::env::args;
use std::mem::{transmute, size_of};
use std::{ops::Range, fs::File};
use std::error::Error;
use std::io::Write;

use rand::random;

const DEFAULT_ARRAY_SIZE: usize = 1000;
const DATA_SIZE: usize = 20;

const array_size_list: [usize; 7] = [10, 100, 1000, 2000, 4000, 8000, 16000];

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

  for array_size in array_size_list {

    let mut data: Vec<Record> = Vec::new();
    let mut accum: u32 = 0;

    for _i in (Range { start: 0, end: array_size })
    {
      accum += (random::<u32>() % 100) + 1;
      data.push(Record {
        key: accum,
        value: make_str()
      })

    }

    let filename = format!("test_files/data_{}.dat", array_size);
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

      file.write_all(&buffer)?;
    }

  }

  Ok(())
}
