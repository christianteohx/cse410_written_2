use crate::data_file::{DataFile, Record};

const TEST_FILE:&str = "test_data.dat";
const TEST_FILE_SIZE: usize = 1000;


const MID_RECORD: Record = Record {
  key: 24979,
  value: ['l', 'z', 'm', 'd', 'b', 'd', 'm', 'w', 's', 'l', 'b', 'c', 'p', 'o', 'w', 'j', 'q', 'a', 'p', 'o']
};

const FOUND_RECORD: Record = Record {
  key: 49252,
  value: ['c', 'i', 'i', 'v', 'b', 'q', 'y', 'f', 'n', 'e', 'c', 'y', 'n', 'o', 'l', 'h', 'k', 'b', 'c', 'z']
};

const FIRST_RECORD: Record = Record {
  key: 49252,
  value: ['c', 'i', 'i', 'v', 'b', 'q', 'y', 'f', 'n', 'e', 'c', 'y', 'n', 'o', 'l', 'h', 'k', 'b', 'c', 'z']
};

#[test]
fn open_file()
{
  let file = DataFile::open(&TEST_FILE.to_string()).unwrap();
  assert!(file.len() == TEST_FILE_SIZE);
}

#[test]
fn get_one()
{
  let mut file = DataFile::open(&TEST_FILE.to_string()).unwrap();
  let result = file.get(file.len() / 2).unwrap();

  println!("get({}): {:?}", file.len() / 2, result);
  assert!(result == MID_RECORD);
}

#[test]
fn find_present()
{
  let mut file = DataFile::open(&TEST_FILE.to_string()).unwrap();
  let result = file.find(FOUND_RECORD.key).unwrap().unwrap();

  println!("find({}): {:?}", FOUND_RECORD.key, result);
  assert!(result == FOUND_RECORD);
}

#[test]
fn find_missing()
{
  let mut file = DataFile::open(&TEST_FILE.to_string()).unwrap();
  let result = file.find(FOUND_RECORD.key-1).unwrap().unwrap();

  println!("find({}): {:?}", FOUND_RECORD.key-1, result);
  assert!(result == FOUND_RECORD);
}