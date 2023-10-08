//! Backend of the paging buffer.

use super::cassette::Cassette;
use super::page::{PageId, PageRequest};
use super::pool::Handle;

/// Internal component responsible for recording and playback.
///
/// It is responsibility of the higher levels to make sure that this structure
/// has access to needed `Page`s and to interact with other components.
pub(crate) struct Buffer {
    active_page: Option<Handle>,
    pointer: usize,
    cassette: Cassette,
    pub recording: bool,
}

impl Buffer {
    pub(crate) fn from_cassette(cassette: Cassette) -> Buffer {
        Self {
            active_page: None,
            pointer: 0,
            cassette,
            recording: false,
        }
    }

    pub(crate) fn next_page(&self) -> PageRequest {
        // TODO: Take this from Page module
        const PAGE_LENGTH: usize = 512;
        let next_index = if let Some(active_page) = self.active_page.as_ref() {
            active_page.page_ref().index() + 1
        } else {
            self.pointer / PAGE_LENGTH
        };
        let load_next = self.pointer < self.cassette.length;
        if load_next {
            PageRequest::Load(PageId::new(self.cassette.id, next_index))
        } else {
            PageRequest::Blank(PageId::new(self.cassette.id, next_index))
        }
    }

    pub(crate) fn has_page(&self) -> bool {
        self.active_page.is_some()
    }

    pub(crate) fn set_page(&mut self, handle: Handle) {
        self.active_page = Some(handle);
    }

    pub(crate) fn process(&mut self, block: &[f32]) {
        if self.has_page() {
            self.pointer += block.len();
            if self.pointer > self.cassette.length {
                self.cassette.length = self.pointer;
            }
            if self.recording {
                self.active_page.as_ref().unwrap().page_mut().mark_dirty();
            }
        }
    }

    pub(crate) fn has_full_page(&self) -> bool {
        // TODO: Take this from Page module
        const PAGE_LENGTH: usize = 512;
        self.active_page.is_some() && self.pointer % PAGE_LENGTH == 0
    }

    pub(crate) fn take_page(&mut self) -> Handle {
        self.active_page.take().unwrap()
    }

    pub(crate) fn reset_position(&mut self) {
        self.pointer = 0;
    }
}
