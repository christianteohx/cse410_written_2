use super::{ Page, PagePointer, META_PAGE_T, PAGE_SIZE };
use static_assertions::const_assert;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MetadataPage
{
  page_type: u8,
  pub next_free_page: PagePointer,
  pub root_page: PagePointer,
  pub data_head: PagePointer,
  pub data_tail: PagePointer,
  pub pages_allocated: PagePointer,
  pub depth: u16,
}
const_assert!(PAGE_SIZE >= size_of::<MetadataPage>());

impl MetadataPage
{
  pub fn init(
    next_free_page: PagePointer,
    root_page: PagePointer,
    data_head: PagePointer,
    data_tail: PagePointer,
    pages_allocated: PagePointer,
    depth: u16
  ) -> MetadataPage
  {
    MetadataPage {
      page_type: META_PAGE_T,
      next_free_page,
      root_page,
      data_head,
      data_tail,
      pages_allocated,
      depth
    }
  }
}

impl Page for MetadataPage
{
  const EXPECTED_PAGE_TYPE: u8 = META_PAGE_T;

  fn page_type(&self) -> u8 { self.page_type }
}