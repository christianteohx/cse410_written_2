use crate::data_file::{DataFile, Record};
use std::{ops::Range, time::Instant};


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
  let file = DataFile::open(&"test_files/data_10.dat".to_string()).unwrap();
  // assert!(file.len() == TEST_FILE_SIZE);
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

#[test]
fn written_2_test() {

  let test_size = 100000;
  let data_info = vec![
    ("test_files/data_10.dat", 10),
    ("test_files/data_100.dat", 100),
    ("test_files/data_1000.dat", 1000),
    ("test_files/data_2000.dat", 2000),
    ("test_files/data_4000.dat", 4000),
    ("test_files/data_8000.dat", 8000),
    ("test_files/data_16000.dat", 16000)
    ];

  for (test_file, array_size) in data_info {

    let mut time_list: Vec<f32> = Vec::new();

    // heat system up
    for _i in (Range { start: 0, end: test_size }) {
      let mut file = DataFile::open(&TEST_FILE.to_string()).unwrap();
      file.find(24979).unwrap().unwrap();
    }

    println!("Heating up done for {}", test_file);

    let mut used_time: f32 = 0.0;
    let mut file = DataFile::open(&test_file.to_string()).unwrap();

    while used_time < 10.0 {

      let key = rand::random::<u32>() % array_size as u32;
      let start = Instant::now();
      let record = file.find(key).unwrap().unwrap();
      let end = Instant::now();
      let time = (end-start).as_secs_f32();
      time_list.push(time);
      used_time += time;

    }

    let total_time: f32 = time_list.iter().sum();
    let mean_time = total_time / time_list.len() as f32;
    let variance = time_list.iter().map(|value| {
      let diff = mean_time - (*value as f32);
      diff * diff
    }).sum::<f32>() / test_size as f32;

    println!("Experiment with {} elements", array_size);
    println!("Total Time: {}", total_time);
    println!("Average Time: {}", mean_time);
    println!("Variance: {}\n\n", variance);
    
  }

}