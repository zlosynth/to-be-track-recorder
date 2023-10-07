//! Non-blocking public interface.

use heapless::spsc::Producer;

use super::buffer::Buffer;
use super::cassette::Cassette;
use super::page::PageRequest;

/// Manager is a non-blocking public interface to paging buffer.
pub(crate) struct Manager {
    buffer: Option<Buffer>,
}

impl Manager {
    pub(crate) fn new() -> Self {
        Self { buffer: None }
    }

    pub(crate) fn set_cassette(&mut self, cassette: Cassette) {
        self.buffer = Some(Buffer::from_cassette(cassette));
    }

    pub(crate) fn start_loading_next_page(
        &mut self,
        load_request_producer: &mut Producer<PageRequest, 4>,
    ) {
        let buffer = self.buffer.as_mut().unwrap();
        let next_page_request = buffer.wants_next();
        load_request_producer
            .enqueue(next_page_request)
            .ok()
            .unwrap();
    }
}
