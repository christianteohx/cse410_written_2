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
fn written_2_test() {

  let test_size = 100000;
  let data_info = vec![
    ("data_10.dat", 10),
    ("data_100.dat", 100),
    ("data_1000.dat", 1000),
    ("data_2000.dat", 2000),
    ("data_4000.dat", 4000),
    ("data_8000.dat", 8000),
    ("data_16000.dat", 16000)
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

      let key = rand::random::<u32>();
      let start = Instant::now();
      file.find(key).unwrap().unwrap();
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