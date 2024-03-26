use std::fs::File;
use std::error::Error;
use std::io::{Seek, Read};
use std::mem::{size_of, transmute};

/// The number of unicode characters in a value blob
const VALUE_SIZE: usize = 20;

/// A representation of one record
#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Record 
{
  pub key: u32,
  pub value: [char; VALUE_SIZE]
}

/// Encodes the runtime metadata for a data file
pub struct DataFile
{
  file: File,
  number_of_records: usize,
  pub min_key: u32,
  pub max_key: u32,
}

/// Transmute a raw byte buffer into a record
fn buffer_to_record(buffer: [u8; size_of::<Record>()]) -> Record
{
   unsafe { transmute::<[u8; size_of::<Record>()], Record>(buffer) }
}

impl DataFile
{
  /// Opens a data file and initializes its metadata
  ///
  /// # Arguments
  ///
  /// * `filename`: The path to the file to be opened
  /// 
  /// The file size must be a multiple of the number of records
  /// 
  /// # Complexity
  /// - Runtime: O(1)
  /// - Memory: O(1)
  /// - IO: O(1)
  ///
  pub fn open(path: &String) 
    -> Result<DataFile,Box<dyn Error>>
  {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len() as usize;
    assert!(len % size_of::<Record>() == 0);
    let number_of_records = len / size_of::<Record>();

    // let mut buf: Vec<u8> = Vec::new();
    // file.read_to_end(&mut buf)?;

    // let mut i: u64 = 0;
    // for c in buf
    // {
    //   i += c as u64
    // }
    // println!("Total: {}", i);

    let mut low_buffer:[u8; size_of::<Record>()] = [0; size_of::<Record>()];
    file.read_exact(&mut low_buffer)?;
    let low = buffer_to_record(low_buffer);

    file.seek(std::io::SeekFrom::End(-(size_of::<Record>() as i64)))?;
    let mut high_buffer:[u8; size_of::<Record>()] = [0; size_of::<Record>()];
    file.read_exact(&mut high_buffer)?;
    let high = buffer_to_record(high_buffer);

    file.seek(std::io::SeekFrom::Start(0))?;

    Ok(DataFile { file, number_of_records, min_key: low.key, max_key: high.key })
  }

  /// Returns the `idx`th record from the file.
  ///
  /// # Arguments
  ///
  /// * `idx`: The index of the record.
  ///
  /// The record to be loaded will begin at byte `idx * size_of::<Record>()`
  /// 
  /// # Complexity
  /// - Runtime: O(1)
  /// - Memory: O(1)
  /// - IO: O(1)
  ///
  pub fn get(&mut self, idx: usize) -> Result<Record,Box<dyn Error>>
  {
    assert!(idx < self.number_of_records);
    self.file.seek(std::io::SeekFrom::Start(
        (idx as u64) * (size_of::<Record>() as u64)
      ))?;

    let mut buffer:[u8; size_of::<Record>()] = [0; size_of::<Record>()];
    self.file.read_exact(&mut buffer)?;

    Ok(buffer_to_record(buffer))
  }

  /// Returns the number of records in the file
  pub fn len(&self) -> usize
  {
    return self.number_of_records;
  }

  /// Retrieves the record with key `key`, or the immediately following record, if one exists.
  ///
  /// # Arguments
  ///
  /// * `key`: The key of the record to retrieve
  ///
  /// If `key` is present in the data file, the corresponding record should be returned. If
  /// not, then the next highest key in the file should be returned.  If key > file.max_key
  /// then this function should return None.
  ///
  /// # Complexity
  /// - Runtime: O(log_2(N))
  /// - Memory: O(1)
  /// - IO: O(log_2(N))
  ///
  pub fn find(&mut self, key: u32) -> Result<Option<Record>,Box<dyn Error>> 
  {
    if key <= self.min_key { Ok(Some(self.get(0)?)) }
    else if key > self.max_key { Ok(None) }
    else { 
      let mut low_idx = 0;
      let mut high_idx = self.number_of_records - 1;

      while low_idx < high_idx
      {
        let split_idx = (high_idx - low_idx) / 2 + low_idx;
        let split_record = self.get(split_idx)?;
        if split_record.key == key { return Ok(Some(split_record)) }
        else if split_record.key < key {
          assert!(split_idx+1 > low_idx);
          low_idx = split_idx+1;
        } else {
          assert!(split_idx < high_idx);
          high_idx = split_idx;
        }
      }
      return Ok(Some(self.get(low_idx)?))
    }
  }
}

