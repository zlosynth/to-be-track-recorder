//! Memory pool storing `Page`s.

use super::page::{Page, PageId};

/// Memory pool that could be used as a global singleton to avoid copying
/// of `Page` blobs.
struct Pool {
    store: [Option<Page>; 4],
}

impl Pool {
    const fn new() -> Pool {
        Pool {
            store: [None, None, None, None],
        }
    }

    fn new_page(&mut self, id: PageId) -> Handle {
        let free = self
            .store
            .iter()
            .enumerate()
            .find(|(_, x)| x.is_none())
            .expect("The pool is full")
            .0;
        self.store[free] = Some(Page::new(id));
        Handle {
            pool_index: free,
            address: &mut self.store[free] as *mut Option<Page>,
        }
    }

    fn drop_page(&mut self, handle: Handle) {
        self.store[handle.pool_index] = None;
    }

    fn stored(&self) -> usize {
        self.store.iter().filter(|x| x.is_some()).count()
    }
}

/// Handle expresses ownership and allows access to a `Page` stored in the `Pool`.
pub(crate) struct Handle {
    pool_index: usize,
    address: *mut Option<Page>,
}

impl Handle {
    pub(crate) fn page_ref(&self) -> &Page {
        unsafe { &*self.address }.as_ref().unwrap()
    }

    fn page_mut(&self) -> &mut Page {
        unsafe { &mut *self.address }.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::paging_buffer::page::PageId;

    use super::*;

    #[test]
    fn initialize_the_pool() {
        static mut POOL: Pool = Pool::new();
        let pool = unsafe { &mut POOL };

        assert_eq!(pool.stored(), 0);
    }

    #[test]
    fn initialize_pages_on_pool() {
        static mut POOL: Pool = Pool::new();
        let pool = unsafe { &mut POOL };

        let _handle_1 = pool.new_page(PageId::new(1, 2));
        assert_eq!(pool.stored(), 1);

        let _handle_2 = pool.new_page(PageId::new(1, 2));
        assert_eq!(pool.stored(), 2);
    }

    #[test]
    fn get_reference_to_a_page_in_pool() {
        static mut POOL: Pool = Pool::new();
        let pool = unsafe { &mut POOL };

        let handle_1 = pool.new_page(PageId::new(1, 2));
        assert_eq!(handle_1.page_ref().id(), PageId::new(1, 2));

        let handle_2 = pool.new_page(PageId::new(1, 3));
        assert_eq!(handle_2.page_ref().id(), PageId::new(1, 3));
    }

    #[test]
    fn get_mutable_reference_to_a_page_in_pool() {
        static mut POOL: Pool = Pool::new();
        let pool = unsafe { &mut POOL };

        let handle = pool.new_page(PageId::new(1, 2));
        assert_eq!(handle.page_mut().id(), PageId::new(1, 2));
    }

    #[test]
    fn drop_page_from_pool() {
        static mut POOL: Pool = Pool::new();
        let pool = unsafe { &mut POOL };

        let handle_1 = pool.new_page(PageId::new(1, 2));
        let handle_2 = pool.new_page(PageId::new(1, 3));

        pool.drop_page(handle_2);
        assert_eq!(pool.stored(), 1);
        pool.drop_page(handle_1);
        assert_eq!(pool.stored(), 0);
    }

    #[test]
    #[should_panic]
    fn fail_when_the_pool_is_full() {
        static mut POOL: Pool = Pool::new();
        let pool = unsafe { &mut POOL };

        loop {
            let _handle = pool.new_page(PageId::new(1, 2));
        }
    }
}
