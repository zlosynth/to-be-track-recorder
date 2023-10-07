//! Non-blocking public interface.

use heapless::spsc::{Consumer, Producer};

use super::buffer::Buffer;
use super::cassette::Cassette;
use super::config::Config;
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

    pub(crate) fn process_configuration_updates(
        &mut self,
        config_consumer: &mut Consumer<Config, 4>,
    ) {
        let buffer = self.buffer.as_mut().unwrap();
        while let Some(config) = config_consumer.dequeue() {
            buffer.recording = config.recording;
        }
    }
}
