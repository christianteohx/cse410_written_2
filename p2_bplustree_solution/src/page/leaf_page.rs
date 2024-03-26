use crate::page::PageIsFullError;

use super::{ Page, PagePointer, LEAF_PAGE_T, NULL_IDX, PAGE_SIZE };
use static_assertions::const_assert;
use std::{mem::size_of, ops::Index};

// You may wish to temporarily change the LEAF_RECORD_COUNT
// parameter below to something smaller while debugging.
// to make your life easier.

pub const LEAF_RECORD_COUNT: usize = 502;  // Max key/value pairs that will fit on one page

#[repr(C)]
#[derive(Debug, Clone)]
pub struct LeafPage
{
  page_type: u8,
  pub count:     usize,
  pub key_value: [(u32, u32); LEAF_RECORD_COUNT],
  pub next:      PagePointer,
  pub prev:      PagePointer
}
const_assert!(PAGE_SIZE >= size_of::<LeafPage>());

#[allow(dead_code)]
impl LeafPage
{
  /// Initialize a fresh, empty leaf page
  pub fn init() -> LeafPage
  {
    LeafPage { 
      page_type: LEAF_PAGE_T,
      count: 0, 
      key_value: [(0,0); LEAF_RECORD_COUNT], 
      next: NULL_IDX,
      prev: NULL_IDX,
    }
  }

  /// Return true if no further key/value pairs may be added
  /// to this directory page.
  pub fn is_full(&self) -> bool
  {
    self.count >= LEAF_RECORD_COUNT
  }

  /// Return true if this page has too few key/value pairs and 
  /// needs to steal/be merged
  pub fn is_underfull(&self) -> bool
  {
    self.count < LEAF_RECORD_COUNT / 2
  }

  /// Return true if this page can afford to lose a key/value 
  /// pair without risking the need for stealing/merging
  pub fn can_allow_stolen_key(&self) -> bool
  {
    self.count > LEAF_RECORD_COUNT / 2
  }

  /// Return the key-value pair 
  /// without risking the need for stealing/merging
  pub fn get(&self, idx: usize) -> (u32, u32)
  {
    self.key_value[idx]
  }

  /// Find the index of the provided key, or where the
  /// key would be inserted if it doesn't exist
  /// 
  /// - Ok(idx) means that the key exists at index idx
  /// - Err(idx) means that the key does not exist, but would
  ///   be inserted at index idx
  pub fn find_index(&self, key: u32) -> Result<usize, usize>
  {
    self.key_value[0..self.count]
        .binary_search_by(|probe:&(u32,u32)|{
          probe.0.cmp(&key)
        })
  }

  /// Split this leaf page into two parts
  ///
  /// Removes half of the key/value pairs on this page
  /// and places them into a newly allocated leaf page
  /// 
  /// **Note:** Split does not attempt to manage the
  /// next/prev pointers.  This must be done by the
  /// caller.
  pub fn split(&mut self) -> LeafPage
  {
    let mut new_page = LeafPage::init();
    let my_size = LEAF_RECORD_COUNT / 2;
    let new_size = LEAF_RECORD_COUNT - my_size;

    new_page.key_value[0 .. new_size].copy_from_slice(
      &self.key_value[my_size .. LEAF_RECORD_COUNT]
    );
    self.count = my_size;
    new_page.count = new_size;
    // For easier debugging, zero out the deleted values
    for i in my_size .. LEAF_RECORD_COUNT
    {
      self.key_value[i] = (0,0)
    }

    return new_page
  }

  /// Find the value for the specified key in the index
  /// if it exists, or None otherwise.
  pub fn find_value(&self, key: u32) -> Option<u32>
  {
    match self.find_index(key)
    {
      Ok(idx) => Some(self.key_value[idx].1),
      Err(_) => None
    }
  }

  /// Insert or update the provided key/value pair.
  ///
  /// - If the key already exists on this page, the corresponding
  ///   value is updated.
  /// - If the key does not already exist on this page, it is
  ///   inserted.  A PageIsFullError is returned if insufficient
  ///   space exists in this case.
  pub fn put(&mut self, key: u32, value: u32) -> Result<(), PageIsFullError>
  {
    match self.find_index(key)
    {
      Ok(idx) => 
      {
        self.key_value[idx].1 = value
      }
      Err(idx) =>
      {
        if self.is_full() { return Err(PageIsFullError {}) }
        self.key_value.copy_within(idx..self.count, idx+1);
        self.key_value[idx] = (key, value);
        self.count += 1;
      }
    }
    Ok(())
  }

  /// Delete the provided key from this page if it exists
  /// Return whether a key was deleted.
  pub fn delete(&mut self, key: u32) -> bool
  {
    match self.find_index(key)
    {
      Ok(idx) => 
      {
        self.key_value.copy_within(idx+1..self.count, idx);
        self.count -= 1;
        self.key_value[self.count] = (0,0);
        true
      }
      Err(_) => false
    }
  }

  /// 'Steal' the greatest key from this page and return
  /// the corresponding key/value pair.  The pair is
  /// removed from this page.
  pub fn steal_high(&mut self) -> (u32, u32)
  {
    assert!(self.can_allow_stolen_key());
    self.count -= 1;
    let kv = self.key_value[self.count];
    // to aid in debugging set the stolen value to 0
    self.key_value[self.count] = (0, 0);
    return kv
  }

  /// 'Steal' the least key from this page and return
  /// the corresponding key/value pair.  The pair is
  /// removed from this page.
  pub fn steal_low(&mut self) -> (u32, u32)
  {
    assert!(self.can_allow_stolen_key());
    let kv = self.key_value[0];
    self.key_value.copy_within(1..self.count, 0);
    self.count -= 1;
    self.key_value[self.count] = (0, 0);
    return kv
  }

  /// Update this page by appending the contents of another 
  /// page.  
  ///
  /// This page must contain the **lesser** of the two sets of
  /// keys.  
  pub fn merge_with(&mut self, other: &LeafPage)
  {
    assert!(self.count + other.count <= LEAF_RECORD_COUNT);

    self.key_value[self.count .. self.count + other.count]
        .copy_from_slice(&other.key_value[0 .. other.count]);
    self.count += other.count;
  }

  /// Obtain an iterator over the elements of this page.
  pub fn iter(&self) -> Box<dyn '_ + Iterator<Item = &(u32, u32)>>
  {
    Box::new(self.key_value.iter().take(self.count))
  }
}

impl Page for LeafPage
{
  const EXPECTED_PAGE_TYPE: u8 = LEAF_PAGE_T;

  fn page_type(&self) -> u8 { self.page_type }
}

impl Index<usize> for LeafPage
{
  type Output = (u32, u32);

  fn index(&self, index: usize) -> &(u32, u32) 
    { &self.key_value[index] }
}