use std::{collections::HashSet, error::Error, ops::Range};

use crate::{bplus_tree::{BPlusResult, BPlusTree}, page::{FreePage, PagePointer}};

use rand::{ rngs::StdRng, RngCore, SeedableRng };

/// Development Step 1:
/// 
/// Implement alloc_page and free_page
///
/// You may modify get_page and put_page
#[test]
fn test_allocation() -> BPlusResult<()>
{
  let mut tests: Vec<(PagePointer, PagePointer)> = Vec::new();

  let path = "target/test_allocation.btree".to_string();
  {
    println!("Init tree");
    let mut tree = BPlusTree::init(&path)?;

    // Use FreePage as a placeholder that we can store stuff in
    let page = FreePage::init(0xfeed);

    println!("Allocate one page");
    let ptr = tree.alloc_page(&page)?;

    println!("Get page {}", ptr);
    {
      let test = tree.get_page::<FreePage>(ptr)?;
      assert!(test.next_free_page == 0xfeed)
    }

    println!("Allocate 16 more pages");
    let ptrs:Vec<PagePointer> = 
      Range { start: 0, end: 16 }
        .map(|i| { 
          tree.alloc_page(&FreePage::init(0xbeef0000+i))
              .expect("Page allocation failed")
        })
        .collect();

    println!("Check page writes");
    for i in (Range { start: 0, end: ptrs.len() })
    {
      let test = tree.get_page::<FreePage>(ptrs[i])?;
      let value = (0xbeef0000+i) as u64;
      println!("Expecting to see {:x} on page {}", value, ptrs[i]);
      assert!(test.next_free_page == value)
    }

    println!("Free the original page and two others");
    tree.free_page(ptr)?;
    tree.free_page(ptrs[8])?;
    tree.free_page(ptrs[9])?;

    println!("Allocate five more pages");
    let more_ptrs:Vec<PagePointer> = 
      Range { start: 0, end: 5 }
        .map(|i| { 
          tree.alloc_page(&FreePage::init(0xabcd0000+i))
              .expect("Page allocation failed")
        })
        .collect();

    println!("Check page writes");
    for i in (Range { start: 0, end: more_ptrs.len() })
    {
      let test = tree.get_page::<FreePage>(more_ptrs[i])?;
      let value = (0xabcd0000+i) as u64;
      println!("Expecting to see {:x} on page {}", value, more_ptrs[i]);
      assert!(test.next_free_page == value)
    }

    println!("Checking that pages are getting re-used on re-allocation");
    let re_use_check:HashSet<PagePointer> = 
      more_ptrs.clone().into_iter().collect();

    assert!(re_use_check.contains(&ptr));
    assert!(re_use_check.contains(&ptrs[8]));
    assert!(re_use_check.contains(&ptrs[9]));

    for i in (Range { start: 0, end: ptrs.len() })
    {
      if (i != 8) && (i != 9)
      {
        tests.push( (ptrs[i], (0xbeef0000+i) as u64) );
      }
    }
    for i in (Range { start: 0, end: more_ptrs.len() })
    {
      tests.push( (more_ptrs[i], (0xabcd0000+i) as u64) );
    }

  }
  // close the block, 'tree' should be freed and closed.
  // open up a new block where we can test the new tree
  {
    let mut tree = BPlusTree::open(&path)?;

    for (ptr, value) in tests
    {
      let test = tree.get_page::<FreePage>(ptr)?;
      println!("Expecting to see {:x} on page {}", value, ptr);
      assert!(test.next_free_page == value)
    }

  }

  Ok(())
}

/// Utility function: Invokes tree.check_tree and asserts if
/// an error is found after printing out the current tree.
fn check_tree(tree: &mut BPlusTree) -> Result<(), Box<dyn Error>>
{
  match tree.check_tree()
  {
    Err(err) => 
    {
      println!("Error reading tree: {:?}", err);
      tree.print_tree()?;
      assert!(false);
    }
    Ok(Some(err)) => 
    {
      println!("Error in tree: {}", err);
      tree.print_tree()?;
      assert!(false);
    }
    Ok(None) => ()
  }
  Ok(())
}

/// Development Step 2:
/// 
/// Implement put
#[test]
fn test_read_write() -> Result<(), Box<dyn Error>>
{
  let path = "target/test_read_write.btree".to_string();
  let mut tree = BPlusTree::init(&path)?;

    check_tree(&mut tree)?;
  tree.put(10, 111)?;
    check_tree(&mut tree)?;
  tree.put(12, 222)?;
    check_tree(&mut tree)?;
  tree.put(8, 333)?;
    check_tree(&mut tree)?;
  tree.put(7, 444)?;
    check_tree(&mut tree)?;
  tree.put(9, 555)?;
    check_tree(&mut tree)?;
  tree.put(14, 666)?;
    check_tree(&mut tree)?;

  let elems: Vec<(u32, u32)> = tree.iter()?.collect();

  println!("Elems after insert: {:?}", elems);
  assert!(elems.len() == 6);
  assert!(elems[0] == (7, 444));
  assert!(elems[1] == (8, 333));
  assert!(elems[2] == (9, 555));
  assert!(elems[3] == (10, 111));
  assert!(elems[4] == (12, 222));
  assert!(elems[5] == (14, 666));

  println!("Passed single-page tests");

  let mut tests: Vec<u32> = Vec::new();

  for _i in (Range { start: 0, end: 1000 })
  {
    let k = rand::random::<u32>();

    tree.put(k, k % 10000)?;
    check_tree(&mut tree)?;
    tests.push(k);
  }

  for k in tests
  {
    assert!(tree.get(k)?.expect("Key not defined") == k % 10000);
  }

  Ok(())
}

/// Development Step 3:
/// 
/// Implement delete
#[test]
fn test_delete() -> Result<(), Box<dyn Error>>
{
  let path = "target/test_delete.btree".to_string();
  let mut tree = BPlusTree::init(&path)?;

  tree.put(50000, 12345)?;

  println!("Passed single-page tests");

  let mut tests: Vec<u32> = Vec::new();

  // Change the seed value to get a different test
  // or use a randomly generated seed to get a fully random test.
  let seed = rand::random::<u64>() % 10000;
  println!("Today's Seed is: {}", seed);
  // let mut rng = StdRng::seed_from_u64(seed);
  let mut rng = StdRng::seed_from_u64(7069);

  for _i in 0 .. 1000
  {
    let k = rng.next_u32() % 10000;
    // println!("Insert {}", k);
    if k != 50000
    {
      tree.put(k, k % 10000)?;
      tests.push(k);
    }
    check_tree(&mut tree)?;
  }

  check_tree(&mut tree)?;

  for k in tests.iter()
  {
    assert!(tree.get(k.to_owned())?.expect("Key not defined") == k % 10000);
  }
  assert!(tree.get(50000)?.expect("Key not defined") == 12345);

  for k in tests
  {
    // println!("Delete {}", k);
    tree.delete(k)?;
    assert!(tree.get(k)?.is_none());
  }
  assert!(tree.depth() == 1);

  Ok(())
}