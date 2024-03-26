mod dir_page;
mod leaf_page;
mod metadata_page;
mod free_page;

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use core::slice;
use std::mem::{ transmute_copy, size_of };

/// The number of bytes in a page
pub const PAGE_SIZE: usize         = 4048; 

/// The expected index of the metadata page
pub const METADATA_IDX: PagePointer = 0;
/// The index of the root directory page in a newly initialized file
pub const DEFAULT_ROOT_IDX: PagePointer = 1;
/// The index of the first data page in a newly initialized file
pub const DEFAULT_PAGE0_IDX: PagePointer = 2;

/// A 'null' index (canonically the metadata page index)
pub const NULL_IDX: PagePointer = 0;

/// The number of records in a directory page (see dir_page.rs)
#[allow(dead_code)]
pub const DIR_KEY_COUNT: usize = dir_page::DIR_KEY_COUNT;
/// The number of records in a leaf page (see leaf_page.rs)
#[allow(dead_code)]
pub const LEAF_RECORD_COUNT: usize = leaf_page::LEAF_RECORD_COUNT;

/// The index of a page
pub type PagePointer = u64;
/// A page holding metadata for the B+Tree
pub type MetadataPage = metadata_page::MetadataPage;
/// A page holding separator values and page pointers
pub type DirectoryPage = dir_page::DirectoryPage;
/// A page holding actual data
pub type LeafPage = leaf_page::LeafPage;
/// An empty 'free' page
pub type FreePage = free_page::FreePage;

/// Type constant for metadata pages
pub const META_PAGE_T:u8 = 0;
/// Type constant for directory pages
pub const DIR_PAGE_T:u8 = 1;
/// Type constant for leaf pages
pub const LEAF_PAGE_T:u8 = 2;
/// Type constant for free pages
pub const FREE_PAGE_T:u8 = 3;

/// A 'page'; a PAGE_SIZE kb-sized chunk of memory that can be
/// written to disk.  This trait implements most of the general
/// logic for encoding/decoding any struct that implements this
/// trait.  
pub trait Page<T = Self>
{
  /// Instances of this page must have the following type code
  const EXPECTED_PAGE_TYPE: u8;

  /// The type code for this page type
  fn page_type(&self) -> u8;

  /// Decode the contents of a buffer into an instance of this
  /// page type
  fn decode(buffer: &[u8; PAGE_SIZE]) -> T
  {
    unsafe {
      transmute_copy::<[u8; PAGE_SIZE], T>(&buffer)
    }
  }

  /// Encode this instance into a provided buffer.
  fn encode(&self, buffer: &mut [u8; PAGE_SIZE])
  {
    let data: &[u8] = 
      unsafe {
        slice::from_raw_parts(
          (self as *const Self) as *const u8, 
          size_of::<T>()
        )
      };
    assert!(data.len() <= PAGE_SIZE);
    buffer[..size_of::<T>()].copy_from_slice(&data);
  }

  /// Read this page from a file
  ///
  /// **Note:** You must seek to the correct position in the
  /// file before calling this function.
  fn read(file: &mut File) -> Result<T, Box<dyn Error>>
  {
    let mut buffer = [0 as u8; PAGE_SIZE];
    file.read_exact(&mut buffer)?;
    Ok(Self::decode(&buffer))
  }

  /// Write this page to a file
  ///
  /// **Note:** You must seek to the correct position in the
  /// file before calling this function.
  fn write(&self, file: &mut File) -> Result<(), Box<dyn Error>>
  {
    let mut buffer = [0 as u8; PAGE_SIZE];
    self.encode(&mut buffer);
    file.write_all(&buffer)?;
    Ok(())
  }
}

#[derive(Debug)]
pub struct PageIsFullError
{
}

impl fmt::Display for PageIsFullError
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Page is full!")
  }
}

impl Error for PageIsFullError
{
  fn source(&self) -> Option<&(dyn Error + 'static)> { None }
}