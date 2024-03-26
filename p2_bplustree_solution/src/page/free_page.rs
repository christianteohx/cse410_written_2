use super::{ Page, PagePointer, FREE_PAGE_T, PAGE_SIZE };
use static_assertions::const_assert;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FreePage
{
  page_type: u8,
  pub next_free_page: PagePointer,
}
const_assert!(PAGE_SIZE >= size_of::<FreePage>());

impl FreePage
{
  pub fn init(next: PagePointer) -> FreePage
  {
    FreePage { page_type: FREE_PAGE_T, next_free_page: next }
  }
}

impl Page for FreePage
{
  const EXPECTED_PAGE_TYPE: u8 = FREE_PAGE_T;

  fn page_type(&self) -> u8 { self.page_type }
}