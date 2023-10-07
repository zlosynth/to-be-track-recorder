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

    pub(crate) fn wants_next(&self) -> PageRequest {
        let next_index = self
            .active_page
            .as_ref()
            .map_or(0, |h| h.page_ref().index());
        let load_next = self.pointer < self.cassette.length;
        if load_next {
            PageRequest::Load(PageId::new(self.cassette.id, next_index))
        } else {
            PageRequest::Blank(PageId::new(self.cassette.id, next_index))
        }
    }
}
