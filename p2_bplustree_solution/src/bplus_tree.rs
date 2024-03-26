use std::borrow::Borrow;
use std::fs::OpenOptions;
use std::io::SeekFrom;
use std::ops::Range;
use std::{error::Error, fs::File, io::Seek};


use super::page::{ NULL_IDX, DEFAULT_ROOT_IDX, DEFAULT_PAGE0_IDX, METADATA_IDX };

use super::page::{ PagePointer, PAGE_SIZE, Page };
use super::page::{ LeafPage, DirectoryPage, MetadataPage, FreePage };

pub type BPlusResult<T> = Result<T, Box<dyn Error>>;


#[derive(Debug)]
pub struct BPlusTree
{
  file: File,
  meta: MetadataPage
}

#[derive(Debug)]
pub struct BPlusTreeIterator<'a>
{
  tree: &'a mut BPlusTree,
  page: LeafPage,
  idx: usize
}

fn seek_addr(idx: PagePointer) -> SeekFrom
{
  SeekFrom::Start(idx * (PAGE_SIZE as u64))
}

#[allow(dead_code)]
impl BPlusTree
{

  /// Initialize a brand new BPlusTree at the provided path
  pub fn init(path: &String) -> BPlusResult<BPlusTree>
  {
    let mut file = 
      OpenOptions::new()
                 .create(true)   // Create file if not present
                 .truncate(true) // Empty the file if it is
                 .read(true)     // Allow reads
                 .write(true)    // Allow writes
                 .open(path)?;

    // Write initial metadata page
    let meta = MetadataPage::init(
      /* next_free_page */  NULL_IDX,
      /* root_page */       DEFAULT_ROOT_IDX,
      /* data_head */       DEFAULT_PAGE0_IDX,
      /* data_tail */       DEFAULT_PAGE0_IDX,
      /* pages_allocated */ 3,
      /* depth */           1,
    );
    file.seek(seek_addr(METADATA_IDX))?;
    meta.write(&mut file)?;

    // Write initial root directory page
    let mut root = DirectoryPage::init();
    root.pointers[0] = DEFAULT_PAGE0_IDX;
    file.seek(seek_addr(DEFAULT_ROOT_IDX))?;
    root.write(&mut file)?;

    // Write initial data page
    let data = LeafPage::init();
    file.seek(seek_addr(DEFAULT_PAGE0_IDX))?;
    data.write(&mut file)?;

    Ok(BPlusTree { file, meta })
  }

  /// Open an existing BPlusTree at the provided path
  pub fn open(path: &String) -> BPlusResult<BPlusTree>
  {
    let mut file = 
      OpenOptions::new()
                 .read(true)     // Allow reads
                 .write(true)    // Allow writes
                 .open(path)?;

    file.seek(seek_addr(METADATA_IDX))?;
    let meta = MetadataPage::read(&mut file)?;

    Ok(BPlusTree { file, meta })
  }

  ////////////////////////////////////////////////////////////////
  //////////////////// Part 1: Page Management ///////////////////
  ////////////////////////////////////////////////////////////////

  /// Write the content of the provided page to a free page
  ///  - Available pages freed with free_page should be used first
  ///  - If no existing free page exists, allocate a new page by
  ///    writing to the end of the file
  /// 
  /// This function should ensure that the file metadata page is 
  /// up-to-date after the page is written.
  /// 
  /// This function should:
  /// - Use O(1) memory
  /// - Perform O(1) IOs
  /// - Have an O(1) runtime 
  pub fn alloc_page<T: Page>(&mut self, page: &T) -> BPlusResult<PagePointer>
  {
    // BEGIN SNIP
    // SNIP ALT:todo!()
    if self.meta.next_free_page == NULL_IDX
    {
      let ptr = self.meta.pages_allocated;
      self.meta.pages_allocated += 1;
      self.put_meta()?;
      self.put_page(ptr, page)?;
      return Ok(ptr);
    }
    else 
    {
      let ptr = self.meta.next_free_page;
      let free = self.get_page::<FreePage>(ptr)?;
      self.meta.next_free_page = free.next_free_page;
      self.put_meta()?;
      self.put_page(ptr, page)?;
      return Ok(ptr);
    }
    // END SNIP
  }

  /// Release the page for use in a new context.  The freed pointer
  /// may be freely overwritten.
  /// 
  /// This function should ensure that the file metadata page is 
  /// up-to-date after the page is written.
  /// 
  /// This function should:
  /// - Use O(1) memory
  /// - Perform O(1) IOs
  /// - Have an O(1) runtime 
  pub fn free_page(&mut self, ptr: PagePointer) -> BPlusResult<()>
  {
    // BEGIN SNIP
    // SNIP ALT:todo!()
    let free = FreePage::init(self.meta.next_free_page);
    self.put_page(ptr, &free)?;
    self.meta.next_free_page = ptr;
    self.put_meta()?;
    Ok(())
    // END SNIP
  }

  /// Retrieve the content of a disk page and decode it.
  ///
  /// For example, the following code retrieves the DirectoryPage
  /// located on page 3:
  /// ```
  /// let page = tree.get_page::<DirectoryPage>(3)?
  /// ```
  ///
  /// This function should:
  /// - Use O(1) memory
  /// - Perform O(1) IOs
  /// - Have an O(1) runtime 
  pub fn get_page<T: Page>(&mut self, ptr: PagePointer) -> BPlusResult<T>
  {
    self.file.seek(seek_addr(ptr))?;
    let ret = T::read(&mut self.file)?;
    assert!(ret.page_type() == T::EXPECTED_PAGE_TYPE);
    Ok(ret)
  }

  /// Write the content of an in-memory page to disk
  ///
  /// This function should:
  /// - Use O(1) memory
  /// - Perform O(1) IOs
  /// - Have an O(1) runtime 
  pub fn put_page<T: Page>(&mut self, ptr: PagePointer, page: &T) -> BPlusResult<()>
  {
    // SNIP ALT:todo!()
    self.file.seek(seek_addr(ptr))?;
    page.write(&mut self.file)
  }

  /// Write the metadata page to disk
  ///
  /// Shorthand for self.put_page(METADATA_IDX, self.meta)
  pub fn put_meta(&mut self) -> BPlusResult<()>
  {
    self.put_page(METADATA_IDX, &self.meta.clone())
  }

  ////////////////////////////////////////////////////////////////
  ////////////////////// Read Methods ////////////////////////////
  ////////////////////////////////////////////////////////////////

  /// Retrieve a specific key, if present
  pub fn get(&mut self, key: u32) -> BPlusResult<Option<u32>>
  {
    let v = self.find_page(key)?;
    let ptr = v[v.len()-1];
    let page = self.get_page::<LeafPage>(ptr)?;
    Ok(page.find_value(key))
  }

  /// Iterate over all of the data values
  pub fn iter<'a>(&'a mut self) -> BPlusResult<BPlusTreeIterator<'a>>
  {
    let data_idx = self.meta.data_head.to_owned();
    let data_page = self.get_page::<LeafPage>(data_idx)?;

    Ok(BPlusTreeIterator { 
      tree: self, 
      page: data_page, 
      idx: 0
    })
  }

  ////////////////////////////////////////////////////////////////
  /////////////////// Part 2: Insertion //////////////////////////
  ////////////////////////////////////////////////////////////////


  /// Insert a new key/value pair into the dataset.
  ///
  /// With N records and K keys per directory page, this 
  /// function's asymptotic bounds should be:
  /// - Memory: O(log_K(N))
  /// - IO: O(log_K(N)) reads, O(1) amortized writes.
  ///
  /// Amortized bounds may assume no intervening calls
  /// to delete()
  ///
  /// You are encouraged to write this function in several steps:
  /// 1. First solve the case where the leaf that would hold 
  ///    key has sufficient space; Leave a todo!() for the 
  ///    other case
  /// 2. Then solve the case where the leaf that would hold
  ///    key needs to be split; Leave a todo!() for the case
  ///    where the parent directory page needs to be split.
  /// 3. Then solve the case where the parent directory page
  ///    is the root and needs to be split.  Leave a todo!() for
  ///    the case where a non-root directory page needs to be
  ///    split.
  /// 4. Finally solve the case where the a non-root directory 
  ///    page needs to be split.
  /// 
  /// You are also encouraged to use several helper functions:
  /// - LeafPage::split()
  /// - DirectoryPage::split_ptr()
  /// - DirectoryPage::split()
  /// - BPlusTree::find_page()
  ///
  pub fn put(&mut self, key: u32, value: u32) -> BPlusResult<()>
  {
    // BEGIN SNIP
    // SNIP ALT:todo!()
    let ptr_stack = self.find_page(key)?;
    // println!("{:?}", ptr_stack);

    let leaf_ptr = ptr_stack[ptr_stack.len()-1];

    let mut leaf = self.get_page::<LeafPage>(leaf_ptr)?;
    // println!("BEFORE: {:?}", leaf);
    if leaf.is_full()
    {
      // Split required
      // println!("BEFORE: {:?}", leaf);
      let (split_key, new_leaf_ptr, mut new_leaf) = 
        self.split_leaf(&mut leaf, ptr_stack.borrow())?;
      assert!(!leaf.is_full());
      assert!(!new_leaf.is_full());
      if key < split_key
      {
        leaf.put(key, value)?;
        self.put_page(leaf_ptr, &leaf)?;
      }
      else 
      {
        new_leaf.put(key, value)?;
        self.put_page(new_leaf_ptr, &new_leaf)?;
      }
      // println!("AFTER: {:?}", leaf);
      // println!("AFTER: {:?}", new_leaf);
      // println!("AFTER: {:?}", self.meta);
    }
    else
    {
      // Split not required
      leaf.put(key, value)?;
      self.put_page(leaf_ptr, &leaf)?;
    }
    // println!("AFTER: {:?}", leaf);

    Ok(())
    // END SNIP
  }

  // BEGIN SNIP
  pub fn split_leaf(&mut self, leaf: &mut LeafPage, ptr_stack: &[PagePointer]) 
    -> BPlusResult<(u32, PagePointer, LeafPage)>
  {
    assert!(leaf.is_full());
    let leaf_ptr = ptr_stack[ptr_stack.len()-1];
    let mut new_leaf = leaf.split();
    let split_key = new_leaf.get(0).0;
    new_leaf.prev = leaf_ptr;
    new_leaf.next = leaf.next;
    let new_leaf_ptr = self.alloc_page(&new_leaf)?;
    if new_leaf.next == NULL_IDX 
    {
      self.meta.data_tail = new_leaf_ptr;
      self.put_meta()?;
    } else 
    {
      let mut old_next = self.get_page::<LeafPage>(new_leaf.next)?;
      old_next.prev = new_leaf_ptr;
      self.put_page(new_leaf.next, &old_next)?
    }
    leaf.next = new_leaf_ptr;
    self.put_page(leaf_ptr, &leaf.clone())?;

    self.split_dir_entry(ptr_stack, split_key, new_leaf_ptr)?;

    return Ok( (split_key, new_leaf_ptr, new_leaf) )
  }

  /// Add a new pointer to a directory page by splitting an
  /// existing pointer into two.
  ///
  /// - `ptr_stack`: The leaf page pointer and its ancestors 
  ///    (see find_page)
  ///
  /// With N records, K keys per directory page, and a directory 
  /// page at depth D < O(log_K(N)), this function should:
  /// - Use O(D) memory
  /// - Perform O(D) unqualified, O(1) amortized IOs
  /// - Have a O(D) unqualified, O(1) amortized runtime
  /// 
  ///
  pub fn split_dir_entry(&mut self, ptr_stack: &[PagePointer], split_key: u32, new_child_ptr: PagePointer)
   -> BPlusResult<()>
  {
    let child_ptr = ptr_stack[ptr_stack.len()-1];
    let dir_ptr = ptr_stack[ptr_stack.len()-2];
    let mut dir_page = self.get_page::<DirectoryPage>(dir_ptr)?;
    if dir_page.is_full()
    {
      let (parent_split_key, new_dir_ptr, mut new_dir_page) = 
        self.split_dir(&mut dir_page, &ptr_stack[0..ptr_stack.len()-1])?;
      assert!(!dir_page.is_full());
      assert!(!new_dir_page.is_full());
      if split_key < parent_split_key
      {
        dir_page.split_at_ptr(child_ptr, split_key, new_child_ptr)?;
        self.put_page(dir_ptr, &dir_page)?;
      }
      else 
      {
        new_dir_page.split_at_ptr(child_ptr, split_key, new_child_ptr)?;
        self.put_page(new_dir_ptr, &new_dir_page)?;
      }
    } else
    {
      dir_page.split_at_ptr(child_ptr, split_key, new_child_ptr)?;
      self.put_page(dir_ptr, &dir_page)?;
    }
    Ok(())
  }

  pub fn split_dir(&mut self, dir: &mut DirectoryPage, ptr_stack: &[PagePointer]) 
    -> BPlusResult<(u32, PagePointer, DirectoryPage)>
  {
    let dir_ptr = ptr_stack[ptr_stack.len()-1];
    if ptr_stack.len() <= 1 // root split
    {
      let (split_key, new_dir_page) = dir.split_page();
      // self.write_tree()?;
      // println!("Split {} at {}", dir_ptr, split_key);
      let new_dir_ptr = self.alloc_page(&new_dir_page)?;
      self.put_page(dir_ptr, &dir.clone())?;
      let mut new_root = DirectoryPage::init();
      new_root.keys[0] = split_key;
      new_root.pointers[0] = dir_ptr;
      new_root.pointers[1] = new_dir_ptr;
      new_root.count = 1;
      let new_root_ptr = self.alloc_page(&new_root)?;
      self.meta.root_page = new_root_ptr;
      self.meta.depth += 1;
      self.put_meta()?;
      // self.write_tree()?;
      Ok( (split_key, new_dir_ptr, new_dir_page) )
    } else
    {
      let (split_key, new_dir_page) = dir.split_page();
      let new_dir_ptr = self.alloc_page(&new_dir_page)?;
      self.put_page(dir_ptr, &dir.clone())?;
      self.split_dir_entry(ptr_stack, split_key, new_dir_ptr)?;
      Ok( (split_key, new_dir_ptr, new_dir_page) )
    }
  }
  // END SNIP


  ////////////////////////////////////////////////////////////////
  //////////////////// Part 2: Deletion //////////////////////////
  ////////////////////////////////////////////////////////////////
  
  /// Delete a key from the tree
  ///
  /// With N records and K keys per directory page, this 
  /// function's asymptotic bounds should be:
  /// - Memory: O(log_K(N))
  /// - IO: O(log_K(N)) reads, O(1) amortized writes.
  ///
  /// Amortized bounds may assume no intervening calls
  /// to put()
  ///
  /// You are encouraged to write this function in several steps:
  /// 1. First solve the case where the leaf that holds the key
  ///    is at least 50% full after the deletion, and does not 
  ///    require a merge.  Leave a todo!() for the other cases.
  /// 2. Next, solve the case where one of the leaves adjacent
  ///    to the underfull leaf has keys that can be stolen.  
  ///    Leave a todo!() for the other cases.
  /// 3. Then, solve the case where the parent of the leaf
  ///    is at least 50% full after losing a pointer, and so
  ///    does not require a recursive merge.  Leave a todo!() 
  ///    for the other cases.
  /// 4. After that, solve the case where an adjacent sibling of
  ///    the parent of the leaf has keys that can be stolen.
  ///    Leave a todo!() for the other cases.
  /// 5. Fifth, solve the case where the parent of the leaf is
  ///    a root page that contains >= 2 pointers after losing
  ///    a pointer.  Leave a todo!() for the other cases. (Hint:
  ///    this case is really really easy :) )
  /// 6. Next, solve the case where the parent of the leaf is
  ///    not a root page.  Leave a todo!() for the final case.
  /// 7. Finally, solve the case where the deletion drops the
  ///    root page down to a single pointer (with no keys).
  /// 
  /// You are also encouraged to use several helper functions:
  /// - BPlusTree::find_page()
  /// - LeafPage::is_underfull()
  /// - LeafPage::can_allow_stolen_key()
  /// - LeafPage::steal_low()
  /// - LeafPage::steal_high()
  /// - DirectoryPage::is_underfull()
  /// - DirectoryPage::can_allow_stolen_key()
  /// - DirectoryPage::steal_low()
  /// - DirectoryPage::steal_high()
  ///
  pub fn delete(&mut self, key: u32) -> BPlusResult<()>
  {
    // BEGIN SNIP
    // SNIP ALT:todo!()
    let ptr_stack = self.find_page(key)?;
    let leaf_ptr = ptr_stack[ptr_stack.len()-1];
    let mut leaf_page = self.get_page::<LeafPage>(leaf_ptr)?;

    if !leaf_page.delete(key) { return Ok(()) }
    if !leaf_page.is_underfull()
    { 
      self.put_page(leaf_ptr, &leaf_page)?;
      return Ok(()) 
    }

    let dir_ptr = ptr_stack[ptr_stack.len()-2];
    let mut dir_page = self.get_page::<DirectoryPage>(dir_ptr)?;
    let dir_idx = dir_page.find_pointer_idx(key);

    let merge_is_low;
    let mut merge_page: LeafPage;
    let merge_ptr: PagePointer;

    // Attempt thefts
    if dir_idx > 0
    {
      let prev_leaf_ptr = leaf_page.prev;
      let mut prev_leaf_page = self.get_page::<LeafPage>(prev_leaf_ptr)?;
      if prev_leaf_page.can_allow_stolen_key()
      {
        // println!("STEALING HIGH TO LEAF PAGE[{}] = {:?}\nFROM PAGE[{}] = {:?}", leaf_ptr, leaf_page, prev_leaf_ptr, prev_leaf_page);
        let (key, value) = prev_leaf_page.steal_high();
        leaf_page.put(key, value)?;
        assert!(dir_idx > 0);
        dir_page.keys[dir_idx-1] = key;
        self.put_page(leaf_ptr, &leaf_page)?;
        self.put_page(prev_leaf_ptr, &prev_leaf_page)?;
        self.put_page(dir_ptr, &dir_page)?;
        return Ok(())
      } else {
        // self.write_tree()?;
        // println!("Merge #{} -> #{}", prev_leaf_ptr, leaf_ptr);
        merge_page = prev_leaf_page;
        merge_ptr  = prev_leaf_ptr;
        merge_is_low = true;
      }
    } else if dir_idx < dir_page.count
    {
      let next_leaf_ptr = dir_page.pointers[dir_idx+1];
      let mut next_leaf_page = self.get_page::<LeafPage>(next_leaf_ptr)?;
      if next_leaf_page.can_allow_stolen_key()
      {
        // println!("STEALING LOW TO LEAF PAGE[{}] = {:?}\nFROM PAGE[{}] = {:?}", leaf_ptr, leaf_page, next_leaf_ptr, next_leaf_page);
        let (key, value) = next_leaf_page.steal_low();
        leaf_page.put(key, value)?;
        dir_page.keys[dir_idx] = next_leaf_page.get(0).0;
        self.put_page(leaf_ptr, &leaf_page)?;
        self.put_page(next_leaf_ptr, &next_leaf_page)?;
        self.put_page(dir_ptr, &dir_page)?;
        return Ok(())
      } else {
        // self.write_tree()?;
        // println!("Merge #{} <- #{}", leaf_ptr, next_leaf_ptr);
        merge_page = next_leaf_page;
        merge_ptr  = next_leaf_ptr;
        merge_is_low = false;
      }
    }
    else 
    {
      // ooops... only one leaf page.  We can't delete it, so
      // leave it be.  Don't forget to flush though.
      self.put_page(leaf_ptr, &leaf_page)?;
      return Ok(())
    }

    // If theft fails, merge pages
    if merge_is_low
    {
      merge_page.merge_with(&leaf_page);
      merge_page.next = leaf_page.next;
      if merge_page.next == NULL_IDX { 
        self.meta.data_tail = merge_ptr;
        self.put_meta()?;
      } else {
        let mut temp_page: LeafPage = self.get_page(merge_page.next)?;
        temp_page.prev = merge_ptr;
        self.put_page(merge_page.next, &temp_page)?
      }
      dir_page.delete_idx(dir_idx);
      self.free_page(leaf_ptr)?;
      self.put_page(merge_ptr, &merge_page)?;
      self.put_page(dir_ptr, &dir_page)?;
    } else
    {
      leaf_page.merge_with(&merge_page);
      leaf_page.next = merge_page.next;
      if leaf_page.next == NULL_IDX { 
        self.meta.data_tail = leaf_ptr;
        self.put_meta()?;
      } else {
        let mut temp_page: LeafPage = self.get_page(leaf_page.next)?;
        temp_page.prev = leaf_ptr;
        self.put_page(leaf_page.next, &temp_page)?
      }
      dir_page.delete_idx(dir_idx+1);
      self.free_page(merge_ptr)?;
      self.put_page(leaf_ptr, &leaf_page)?;
      self.put_page(dir_ptr, &dir_page)?;
    }
    if dir_page.is_underfull()
    { 
      self.merge_dir_page(&ptr_stack[0..ptr_stack.len()-1], key)?
    }

    Ok(())
    // END SNIP
  }

  fn merge_dir_page(&mut self, ptr_stack: &[PagePointer], key: u32) -> BPlusResult<()>
  {
    // println!("Merge Dir @ {:?}", ptr_stack);
    let dir_ptr = ptr_stack[ptr_stack.len()-1];
    let mut dir_page = self.get_page::<DirectoryPage>(dir_ptr)?;

    if ptr_stack.len() <= 1
    {
      // Root merge
      // Case 1: The root has > 1 key.  This is ok.  Leave
      //         it as is.
      if dir_page.count >= 1 { return Ok(()); }

      // Case 2: The tree is a single level deep
      if self.meta.depth == 1 { return Ok(()); }

      // Case 3: The root has no keys.  Replace the root with
      //         the page being pointed to
      self.meta.root_page = dir_page.pointers[0];
      self.meta.depth -= 1;
      self.put_meta()?;
      self.free_page(ptr_stack[0])?;
      return Ok(())
    } else 
    {
      let parent_ptr = ptr_stack[ptr_stack.len()-2];
      let mut parent_page = self.get_page::<DirectoryPage>(parent_ptr)?;
      let dir_idx = parent_page.find_pointer_idx(key);

      let sibling_is_low;
      let sibling_ptr;
      let mut sibling_page: DirectoryPage;

      // Attempt thefts
      if dir_idx > 0 
      {
        sibling_ptr = parent_page.pointers[dir_idx-1];
        sibling_page = self.get_page::<DirectoryPage>(sibling_ptr)?;
        if sibling_page.can_allow_stolen_key()
        {
          // println!("STEALING HIGH TO PAGE[{}] = {:?}\nFROM: PAGE[{}] = {:?}", dir_ptr, dir_page, sibling_ptr, sibling_page);
          let new_parent_key = 
            dir_page.steal_high_from(&mut sibling_page, parent_page.keys[dir_idx-1]);
          parent_page.keys[dir_idx-1] = new_parent_key;
          self.put_page(dir_ptr, &dir_page)?;
          self.put_page(sibling_ptr, &sibling_page)?;
          self.put_page(parent_ptr, &parent_page)?;
          return Ok(())
        } else {
          // self.write_tree()?;
          // println!("Merge #{} -> #{}", prev_leaf_ptr, leaf_ptr);
          sibling_is_low = true;
        }
      } else
      {
        sibling_ptr = parent_page.pointers[dir_idx+1];
        sibling_page = self.get_page::<DirectoryPage>(sibling_ptr)?;
        if sibling_page.can_allow_stolen_key()
        {
          // println!("STEALING LOW TO PAGE[{}] = {:?}\nFROM: PAGE[{}] = {:?}", dir_ptr, dir_page, sibling_ptr, sibling_page);
          let new_parent_key = 
            dir_page.steal_low_from(&mut sibling_page, parent_page.keys[dir_idx]);
          parent_page.keys[dir_idx] = new_parent_key;
          self.put_page(dir_ptr, &dir_page)?;
          self.put_page(sibling_ptr, &sibling_page)?;
          self.put_page(parent_ptr, &parent_page)?;
          return Ok(())
        } else {
          // self.write_tree()?;
          // println!("Merge #{} <- #{}", leaf_ptr, next_leaf_ptr);
          sibling_is_low = false;
        }
      }

      // If theft fails, merge pages
      
      assert!(sibling_ptr != NULL_IDX); // at least one of the above
                                        // must have failed
      if sibling_is_low
      {
        // println!("Before Merge 1 <-: {:?}", sibling_page);
        // println!("Before Merge 2: {:?}", dir_page);
        sibling_page.merge_with(&dir_page, parent_page.keys[dir_idx-1]);
        // println!("After Merge @ {}: {:?}", dir_idx, sibling_page);
        parent_page.delete_idx(dir_idx);
        self.free_page(dir_ptr)?;
        self.put_page(sibling_ptr, &sibling_page)?;
        self.put_page(parent_ptr, &parent_page)?;
      } else
      {
        // println!("Before Merge 1 ->: {:?}", sibling_page);
        // println!("Before Merge 2: {:?}", dir_page);
        dir_page.merge_with(&sibling_page, parent_page.keys[dir_idx]);
        // println!("After Merge @ {}: {:?}", dir_idx, dir_page);
        parent_page.delete_idx(dir_idx+1);
        self.free_page(sibling_ptr)?;
        self.put_page(dir_ptr, &dir_page)?;
        self.put_page(parent_ptr, &parent_page)?;
      }
      if parent_page.is_underfull()
      { 
        self.merge_dir_page(&ptr_stack[0..ptr_stack.len()-1], key)?
      }
    }
    Ok(())
  }

  ////////////////////////////////////////////////////////////////
  /////////////////// Utility Functions //////////////////////////
  ////////////////////////////////////////////////////////////////

  /// Recover the page path from the root to the leaf containing the 
  /// specified key
  /// 
  /// - The first page pointer returned is the root
  /// - The final page pointer in the is the leaf containing (or 
  ///   that would contain the key)
  pub fn find_page(&mut self, key: u32) -> BPlusResult<Box<[PagePointer]>>
  {
    let mut ret: Vec<PagePointer> = Vec::new();
    let mut curr_ptr = self.meta.root_page;
    ret.push(curr_ptr);

    for _i in (Range { start: 0, end: self.meta.depth })
    {
      let dir = self.get_page::<DirectoryPage>(curr_ptr)?;
      curr_ptr = dir.find_pointer(key);
      ret.push(curr_ptr);
    }

    return Ok(ret.into_boxed_slice())
  }

  /// Return the depth of the tree
  pub fn depth(&self) -> u16
  {
    self.meta.depth
  }

  /// Sanity check the tree
  ///
  /// Returns a string containing the first problem it encounters
  /// or None if no errors are encountered.
  ///
  /// As usual, an error is reported if there's a problem.
  pub fn check_tree(&mut self) -> BPlusResult<Option<String>>
  {
    let mut dir_stack: Vec<(PagePointer, usize, u32, u32)> = Vec::new();

    let mut curr_ptr: PagePointer = self.meta.root_page;
    let mut curr_idx = 0;
    let mut low: u32 = 0;
    let mut high: u32 = u32::MAX;

    let mut last_data: PagePointer = 0;
    let mut next_data: PagePointer = self.meta.data_head;

    loop {
      // Descend to the next data page
      for _i in dir_stack.len() as u16 .. self.meta.depth
      {
        dir_stack.push( (
          curr_ptr,
          curr_idx,
          low,
          high
        ) );
        if curr_ptr >= self.meta.pages_allocated 
        { 
          if dir_stack.is_empty() { return Ok(Some(format!("Invalid root pointer for tree: {}", curr_ptr))); }
          else                    { return Ok(Some(format!("Invalid pointer: {} stored in directory page {}", curr_ptr, dir_stack.last().unwrap().0))); }
        }
        // println!("Descend into directory page {} at index {} (low = {}, high = {})", curr_ptr, curr_idx, low, high);
        let curr_dir_page: DirectoryPage = self.get_page(curr_ptr)?;
        if dir_stack.len() > 1 {
          if curr_dir_page.is_underfull() 
            { return Ok(Some(format!("Underfull page {}: {:?}", curr_ptr, curr_dir_page))); }
        } else {
          if curr_dir_page.count == 0 && self.meta.depth > 1
            { return Ok(Some(format!("Empty root page {}: {:?}", curr_ptr, curr_dir_page))); }
        }
        for k in curr_dir_page.keys.iter().take(curr_dir_page.count)
        {
          if *k < low   { return Ok(Some(format!("Split Key {} < Parent constraint {} on page {}: {:?}", k, low, curr_ptr, curr_dir_page))); }
          if *k >= high { return Ok(Some(format!("Split Key {} >= Parent constraint {} on page {}: {:?}", k, high, curr_ptr, curr_dir_page))); }
        }
        curr_ptr = curr_dir_page.pointers[curr_idx];
        if curr_idx > 0                        { low = curr_dir_page.keys[curr_idx-1]; }
        if curr_dir_page.count > 0
           && curr_idx < curr_dir_page.count-1 { high = curr_dir_page.keys[curr_idx]; }
        curr_idx = 0;
      }

      // println!("Visit leaf page {} (prev = {}, curr = {}; low = {}, high = {})", last_data, next_data, curr_ptr, low, high);
      // Sanity check the current leaf page
      if curr_ptr >= self.meta.pages_allocated 
      { 
        if dir_stack.is_empty() { return Ok(Some(format!("Invalid root pointer for tree: {}", curr_ptr))); }
        else                    { return Ok(Some(format!("Invalid pointer: {} stored in directory page {}", curr_ptr, dir_stack.last().unwrap().0))); }
      }
      let curr_leaf_page: LeafPage = self.get_page(curr_ptr)?;
      if curr_leaf_page.is_underfull() && self.meta.depth > 1 
        { return Ok(Some(format!("Underfull page {}: {:?}", curr_ptr, curr_leaf_page))); }
      for (k, _) in curr_leaf_page.iter()
      {
        if *k < low   { return Ok(Some(format!("Split Key {} < Parent constraint {} on page {}: {:?}", k, low, curr_ptr, curr_leaf_page))); }
        if *k >= high { return Ok(Some(format!("Split Key {} >= Parent constraint {} on page {}: {:?}", k, high, curr_ptr, curr_leaf_page))); }
      }
      if next_data != curr_ptr            { return Ok(Some(format!("Next pointer != {} on page {}", next_data, curr_ptr))); }
      if last_data != curr_leaf_page.prev { return Ok(Some(format!("Prev pointer != {} on page {}: {:?}", last_data, curr_ptr, curr_leaf_page))); }
      next_data = curr_leaf_page.next;
      last_data = curr_ptr;

      // Ascend until we have a 'next'
      (curr_ptr, curr_idx, low, high) = dir_stack.pop().unwrap();
      if curr_ptr >= self.meta.pages_allocated 
      { 
        if dir_stack.is_empty() { return Ok(Some(format!("Invalid root pointer for tree: {}", curr_ptr))); }
        else                    { return Ok(Some(format!("Invalid pointer: {} stored in directory page {}", curr_ptr, dir_stack.last().unwrap().0))); }
      }
      let mut curr_dir_page: DirectoryPage = self.get_page(curr_ptr)?;
      // println!("Ascend to directory page {} from index {} / {}", curr_ptr, curr_idx, curr_dir_page.count);
      while curr_idx >= curr_dir_page.count
      {
        (curr_ptr, curr_idx, low, high) = 
          match dir_stack.pop() {
            Some(s) => s,
            None => {
              if next_data != 0                   { return Ok(Some(format!("Last data page {} points to {} and not NULL", last_data, next_data)))}
              if last_data != self.meta.data_tail { return Ok(Some(format!("Metadata tail pointer points to {} and not {}", self.meta.data_tail, last_data)))}
              return Ok(None)
            }
          };
        if curr_ptr >= self.meta.pages_allocated 
        { 
          if dir_stack.is_empty() { return Ok(Some(format!("Invalid root pointer for tree: {}", curr_ptr))); }
          else                    { return Ok(Some(format!("Invalid pointer: {} stored in directory page {}", curr_ptr, dir_stack.last().unwrap().0))); }
        }
        curr_dir_page = self.get_page(curr_ptr)?;
        // println!("Ascend to directory page {} from index {} / {}", curr_ptr, curr_idx, curr_dir_page.count);
      }
      curr_idx += 1;
    }
  }


  /// Helper function: print the entire tree
  pub fn print_tree(&mut self) -> BPlusResult<()>
  {
    fn rcr(tree: &mut BPlusTree, page: PagePointer, depth: u16)
      -> BPlusResult<()>
    {
      if depth < tree.meta.depth
      {
        let data = tree.get_page::<DirectoryPage>(page)?;
        println!("{}PAGE[{}] = {:?}\n", std::iter::repeat(" ").take((depth*2) as usize).collect::<String>(), page, data);
        for page in &data.pointers[0 .. data.count+1]
        {
          rcr(tree, page.clone(), depth+1)?;
        }
      } else
      {
        let data = tree.get_page::<LeafPage>(page)?;
        println!("{}PAGE[{}] = {:?}\n", std::iter::repeat(" ").take((depth*2) as usize).collect::<String>(), page, data);
      }
      Ok(())
    }
    rcr(self, self.meta.root_page, 0)
  }
}

impl<'a> Iterator for BPlusTreeIterator<'a>
{
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
      while self.idx >= self.page.count
      {
        if self.page.next == NULL_IDX
        {
          return None
        }
        else {
          let next_page = self.page.next;
          self.page = 
            self.tree.get_page(next_page)
                     .expect(format!("Couldn't read next page {}", next_page).as_str());
          self.idx = 0
        }
      }
      let ret = self.page.get(self.idx);
      self.idx += 1;
      return Some(ret);
    }
}