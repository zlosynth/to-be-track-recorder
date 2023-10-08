//! Non-blocking public interface.

use heapless::spsc::{Consumer, Producer};

use super::buffer::Buffer;
use super::cassette::Cassette;
use super::config::Config;
use super::page::{Page, PageId, PageRequest};
use super::pool::Handle;

/// Manager is a non-blocking public interface to paging buffer.
pub(crate) struct Manager {
    buffer: Option<Buffer>,
    page_1_cache: Option<Handle>,
}

impl Manager {
    pub(crate) fn new() -> Self {
        Self {
            buffer: None,
            page_1_cache: None,
        }
    }

    pub(crate) fn set_cassette(&mut self, cassette: Cassette) {
        self.buffer = Some(Buffer::from_cassette(cassette));
    }

    pub(crate) fn start_loading_next_page(
        &mut self,
        load_request_producer: &mut Producer<PageRequest, 4>,
    ) {
        let buffer = self.buffer.as_mut().unwrap();
        let next_page_request = buffer.next_page();
        load_request_producer
            .enqueue(next_page_request)
            .ok()
            .unwrap();
    }

    pub(crate) fn process_configuration_updates(
        &mut self,
        config_consumer: &mut Consumer<Config, 4>,
    ) {
        let buffer = self.buffer.as_mut().unwrap();
        while let Some(config) = config_consumer.dequeue() {
            buffer.recording = config.recording;
        }
    }

    pub(crate) fn is_waiting_for_page(&self) -> bool {
        let buffer = self.buffer.as_ref().unwrap();
        !buffer.has_page()
    }

    pub(crate) fn try_fetching_next_page(
        &mut self,
        load_response_consumer: &mut Consumer<Handle, 4>,
    ) -> bool {
        let buffer = self.buffer.as_mut().unwrap();

        if buffer.next_page().page_id().page_index() == 0 {
            if let Some(handle) = self.page_1_cache.take() {
                buffer.set_page(handle);
                return true;
            }
        }

        while let Some(handle) = load_response_consumer.dequeue() {
            if handle.page_ref().id() == buffer.next_page().page_id() {
                buffer.set_page(handle);
                return true;
            }
        }

        false
    }

    pub(crate) fn process(&mut self, block: &[f32]) {
        let buffer = self.buffer.as_mut().unwrap();
        buffer.process(block);
    }

    pub(crate) fn has_full_page(&self) -> bool {
        let buffer = self.buffer.as_ref().unwrap();
        buffer.has_full_page()
    }

    pub(crate) fn start_saving(
        &mut self,
        save_request_producer: &mut Producer<Handle, 4>,
        save_request_first_page_producer: &mut Producer<Page, 4>,
    ) {
        let buffer = self.buffer.as_mut().unwrap();

        let page = buffer.take_page();

        // TODO: Check cassette ID too
        if page.page_ref().is_dirty() {
            if page.page_ref().index() == 0 {
                save_request_first_page_producer
                    .enqueue(page.page_clone())
                    .ok()
                    .unwrap();
                self.page_1_cache = Some(page);
            } else {
                save_request_producer.enqueue(page).ok().unwrap();
            }
        }
    }

    pub(crate) fn reset_position(&mut self) {
        let buffer = self.buffer.as_mut().unwrap();
        buffer.reset_position();
    }
}
