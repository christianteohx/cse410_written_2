use crate::page::NULL_IDX;

use super::{ Page, PageIsFullError, PagePointer, DIR_PAGE_T, PAGE_SIZE };
use static_assertions::const_assert;
use std::mem::size_of;

// You may wish to temporarily change the DIR_KEY_COUNT
// parameter below to something smaller while debugging.
// to make your life easier.

pub const DIR_KEY_COUNT: usize     = 335;  // Max key/ptr pairs that will fit on one page
pub const DIR_PTR_COUNT: usize     = DIR_KEY_COUNT+1;


/// A page containing directory data
///
/// ```
/// DirPage([p0 k0 p1 k1 p2, ...]
/// ```
/// - p0 is a pointer to a subtree who's keys are
///   strictly lesser than k0
/// - p1 is a pointer to a subtree who's keys are
///   greater than or equal to k0 and strictly lesser
///   than k1
/// - etc...
///
/// Keys and pointers are stored in separate arrays:
/// - keys = [k0, k1, ...]
/// - pointers = [p0, p1, ...]
/// Note that there is always exactly one more pointer than 
/// there is key (count measures the number of **keys**).
#[repr(C)]
#[derive(Debug, Clone)]
pub struct DirectoryPage
{
  page_type:    u8,
  
  /// The number of keys in this page.  The number of 
  /// pointers is always 1 higher
  pub count:    usize,

  /// The array of keys
  pub keys:     [u32; DIR_KEY_COUNT],

  /// The array of pointers
  pub pointers: [PagePointer; DIR_PTR_COUNT],
}
const_assert!(PAGE_SIZE >= size_of::<DirectoryPage>());

#[allow(dead_code)]
impl DirectoryPage
{
  /// Generate a fresh DirectoryPage
  pub fn init() -> DirectoryPage
  {
    DirectoryPage {
      page_type: DIR_PAGE_T, 
      count: 0, 
      keys: [0 as u32; DIR_KEY_COUNT], 
      pointers: [NULL_IDX; DIR_PTR_COUNT]
    }
  }

  /// Find the index into `.pointers` that one would follow
  /// to retrieve the provided key.
  /// 
  /// Discounting edge cases, if find_pointer_index(k)
  /// returns idx, then the subtree rooted at .pointers[idx]
  /// is guaranteed to...
  /// - Contain only keys strictly lesser than .keys[idx]
  /// - Contain only keys greater than or equal to .keys[idx-1]
  ///
  /// The return value is guaranteed to be in the range 
  /// [0, count] (note the *inclusive* upper bound).
  pub fn find_pointer_idx(&self, key: u32) -> usize
  {
    if self.count == 0 || key < self.keys[0] { return 0; }

    let mut start = 0;
    let mut end = self.count;

    while start < end - 1
    {
      let mid = (end - start) / 2 + start;
      // println!("  BINARY: {}-{}-{}", start, mid, end);
      if key < self.keys[mid]                        { end = mid; }
      else if start == mid && key < self.keys[mid+1] { return mid; }
      else                                           { start = mid; }
      // println!("    -> {}-{}", start, end);
    }
    return start+1;
  }
  
  /// Find the pointer that one would follow to retrieve the
  /// provided key.  
  ///
  /// This function is just a shorthand for 
  /// `self.pointers[self.find_pointer_idx(key)]`
  pub fn find_pointer(&self, key: u32) -> PagePointer
  {
    self.pointers[self.find_pointer_idx(key)]
  }

  /// Return true if no further keys/pointers may be added
  /// to this directory page.
  pub fn is_full(&self) -> bool
  {
    self.count >= DIR_KEY_COUNT
  }

  /// Return true if this page has too few keys/pointers and 
  /// needs to steal/be merged
  pub fn is_underfull(&self) -> bool
  {
    // count is given in terms of keys, not pointers
    // allow the count to drop to half the number of *pointers*
    self.count < (DIR_KEY_COUNT / 2)-1
  }

  /// Return true if this page can afford to lose a key/pointer
  /// without risking the need for stealing/merging
  pub fn can_allow_stolen_key(&self) -> bool
  {
    self.count > (DIR_KEY_COUNT / 2)
  }

  /// Modify the page by inserting a new key/pointer pair after
  /// split_ptr.
  /// 
  /// - split_ptr must be an existing pointer in the page.
  /// - split_key must be a value in the range [idx-1, idx)
  ///   where split_ptr is the idx'th key on this page.
  /// - new_ptr is the new pointer
  ///
  /// Starting With `DirPage([p0 k0 p1 k1 p2 k2])`
  /// calling: `split_ptr(p1, k4, p4)` 
  /// would result in: `DirPage([p0 k0 p1 k4 p4 k1 p2 k2])`
  /// 
  /// Note that k0 < k4 < k1
  pub fn split_at_ptr(&mut self, split_ptr: PagePointer, split_key: u32, new_ptr: PagePointer) 
    -> Result<(), PageIsFullError>
  {
    // println!("{:?} <- Split {} @ {} to add {}", self, split_ptr, split_key, new_ptr);
    if self.is_full() { return Err(PageIsFullError {  })}

    let idx = self.find_pointer_idx(split_key);

    // println!("   @Idx: {}", idx);
    assert!(self.pointers[idx] == split_ptr);
    if idx < self.count
    {
      self.keys.copy_within(idx .. self.count, idx+1);
      self.pointers.copy_within(idx+1 .. self.count+1, idx+2);
    }
    self.keys[idx] = split_key;
    self.pointers[idx+1] = new_ptr;
    self.count += 1;
    // println!("   AFTER: {:?}", self);

    Ok(())
  }

  /// Split the page in half
  ///
  /// This function returns the newly created DirectoryPage
  /// object, and the key that separates them.
  ///
  /// ```
  ///   [k0, k1, ..., kN-1, kN]
  /// [p0, p1, p2, ...,  pN, pN+1]
  /// 
  ///   |<--- my_size+1 -->|
  ///   |<- my_size ->|        |<- new_size ->|
  ///   [k0, ..., kM-1]   kM   [kM+1, ...,  kN]
  /// [p0, p1, ..., pM]      [pM+1, ..., pN, pN+1]
  /// |<- my_size+1 ->|      |<-- new_size+1 --->|
  /// ```
  /// 
  pub fn split_page(&mut self) -> (u32, DirectoryPage)
  {
    let mut new_page = DirectoryPage::init();
    let my_size = DIR_KEY_COUNT / 2;            // M = N/2
    let new_size = DIR_KEY_COUNT - my_size -1;  // N-M-1

    // println!("me: {}; new: {}", my_size, new_size);

    new_page.keys[0 .. new_size].copy_from_slice(
      &self.keys[my_size+1 .. DIR_KEY_COUNT]
    );
    new_page.pointers[0 .. new_size+1].copy_from_slice(
      &self.pointers[my_size+1 .. DIR_PTR_COUNT]
    );
    // clear out the old k/p pairs to aid in debugging
    for i in &mut self.keys[my_size+1 .. DIR_KEY_COUNT]     { *i = 0 }
    for i in &mut self.pointers[my_size+1 .. DIR_PTR_COUNT] { *i = NULL_IDX }

    self.count = my_size;
    new_page.count = new_size;

    return (self.keys[my_size], new_page)
  }

  /// Delete the pointer at the specified index and the preceding
  /// key.
  ///
  /// Note that the index provided is that of the *pointer*.
  ///
  /// Before: DirPage([p0 k0 p1 k1 p2 k2 p3])
  /// Delete of index 2 (i.e., p2)
  /// After: DirPage([p0 k0 p1 k2 p3])
  pub fn delete_idx(&mut self, idx: usize)
  {
    assert!(idx > 0);
    self.keys.copy_within(idx..self.count, idx-1);
    self.pointers.copy_within((idx+1)..(self.count+1), idx);
    self.count -= 1;
    // Technically not needed, but just for safety, let's clear
    // out the old values 
    self.keys[self.count] = 0;
    self.pointers[self.count+1] = NULL_IDX;
  }

  /// 'Steal' a key/pointer from the other page, assuming that
  /// the other page is the immediately preceding sibling.
  ///
  ///           DirPage( [p0 k0 p1 k1 p2 k2 p3])
  ///       ...❜       /               \        `...
  ///  p1:DirPage( [p4 k4 p5 ] )  p2:DirPage( [p6 k6 p7] )
  /// 
  ///
  ///  Note that pages p1 and p2 are separated by k1
  ///
  ///  After calling p2.steal_high_from(p1, k1)...
  ///  - p1:DirPage( [p4] )
  ///  - p2:DirPage( [p5 k1 p6 k6 p7])
  ///  - k4 is returned for re-insertion into the parent
  ///  directory page.
  pub fn steal_high_from(&mut self, other: &mut DirectoryPage, parent_key: u32)
    -> u32
  {
    assert!(self.count < DIR_KEY_COUNT);
    assert!(other.count > 0);
    // free up space
    self.keys.copy_within(0..self.count, 1);
    println!("{:?} -> {}", self.pointers, self.count);
    self.pointers.copy_within(0..self.count+1, 1);
    println!("{:?}", self.pointers);
    // move p5 (@other.count - 1 + 1)
    self.pointers[0] = other.pointers[other.count];
    // update k1
    self.keys[0] = parent_key;
    // retrieve the new parent pointer
    let ret = other.keys[other.count-1];

    // note that we don't need to overwrite `other`, since
    // decrementing its count automatically removes the
    // pointer from consideration... still, for the sake
    // of safety:
    other.keys[other.count-1] = 0;
    other.pointers[other.count] = NULL_IDX;

    other.count -= 1;
    self.count += 1;

    return ret;
  }


  /// 'Steal' a key/pointer from the other page, assuming that
  /// the other page is the immediately following sibling.
  ///
  ///           DirPage( [p0 k0 p1 k1 p2 k2 p3])
  ///       ...❜       /               \        `...
  ///  p1:DirPage( [p4 k4 p5 ] )  p2:DirPage( [p6 k6 p7] )
  /// 
  ///
  ///  Note that pages p1 and p2 are separated by k1
  ///
  ///  After calling p1.steal_low_from(p2, k1)...
  ///  - p1:DirPage( [p4 k4 p5 k1 p6] )
  ///  - p2:DirPage( [p7])
  ///  - k6 is returned for re-insertion into the parent
  ///  directory page.
  pub fn steal_low_from(&mut self, other: &mut DirectoryPage, parent_key: u32)
   -> u32
  {
    assert!(self.count < DIR_KEY_COUNT);
    assert!(other.count > 0);
    // preserve k6 to be returned
    let ret = other.keys[0];
    // insert k1
    self.keys[self.count] = parent_key;
    // insert p6
    self.pointers[self.count+1] = other.pointers[0];

    self.count += 1;

    // Shift all of the keys/pointers in other to move them
    // back into place.
    other.keys.copy_within(1..other.count, 0);
    other.pointers.copy_within(1..other.count+1, 0);
    other.count -= 1;

    // Technically unnecessary, but just to aid in debugging
    // zero out the old keys.
    other.keys[other.count] = 0;
    other.pointers[other.count+1] = NULL_IDX;

    return ret;
  }

  /// 'Merge' this directory page with it's immediately 
  /// following sibling
  ///
  ///           DirPage( [p0 k0 p1 k1 p2 k2 p3])
  ///       ...❜       /               \        `...
  ///  p1:DirPage( [p4 k4 p5 ] )  p2:DirPage( [p6 k6 p7] )
  ///
  ///  Note that pages p1 and p2 are separated by k1
  /// 
  ///  After calling p1.merge_with(p2, k1)...
  ///  - p1: DirPage( [p4 k4 p5 k1 p6 k6 p7] )
  ///  - p2: unchanged
  pub fn merge_with(&mut self, other: & DirectoryPage, parent_key: u32)
  {
    assert!(self.count + other.count <= DIR_KEY_COUNT);
    self.keys[self.count] = parent_key;
    self.pointers[(self.count+1)..(self.count+1+other.count+1)]
        .copy_from_slice(&other.pointers[0..(other.count+1)]);
    self.keys[(self.count+1)..(self.count+1+other.count)]
        .copy_from_slice(&other.keys[0..(other.count)]);
    self.count += other.count + 1;
  }
}

impl Page for DirectoryPage
{
  const EXPECTED_PAGE_TYPE: u8 = DIR_PAGE_T;
  fn page_type(&self) -> u8 { self.page_type }
}