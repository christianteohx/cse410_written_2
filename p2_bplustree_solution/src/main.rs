mod bplus_tree;
mod page;
#[cfg(test)] mod test;

use std::error::Error;
use std::result::Result;
use bplus_tree::BPlusTree;

fn main() -> Result<(), Box<dyn Error>>
{
  BPlusTree::init(&"test.btree".to_string())?;

  Ok(())
}
