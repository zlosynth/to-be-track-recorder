//! Manipulate persistent buffers with lengths that are exceeding memory capacity.
//!
//! # Requirements
//!
//! * Can handle loops from 32 sample length up to tens of minutes.
//! * Allows simultaneous playback and recording.
//! * Recorded audio is being saved even while recording is in progress.
//! * Can immediatelly jump to the beginning of the sample and start playing.
//! * Saving and loading is done in another routine.
//!
//! # Architecture
//!
//! * Caller is responsible for:
//!   * Providing new empty pages on request.
//!   * Providing previously returned pages on request.
//!   * Persisting returned pages.
//!   * Doing the two listed above with RT guarantees.
//! * Each of the page contains:
//!   * Fixed-size array of data.
//!   * "Dirty" flag.
//!   * Length of recorded data.
//!   * Start address, relative to the parent sample.
//!
//! # Flow starting from fresh
//!
//! 1. Caller initializes empty page on and passes it to the buffer.
//! 2. Buffer stores the page in its struct.
//! 3. Caller passes input audio, info about armed channels.
//! 4. Buffer writes the audio into its active page and returns output audio.
//! 5. Caller asks the buffer whether it is full, if it is, it takes its page and passes
//!    a fresh one to it again.
//! 6. Since this was the first page, caller stores it in its cache.
//! 7. Caller passess the dirty page to SD save queue.
//! 8. Buffer continues recording, until its full again, swaps the page.
//! 9. Caller passes the dirty page to save queue.
//! 10. This continues for some time, until position reset is triggered.
//! 11. With reset armed, caller will force buffer to return its current buffer,
//!     and it will pass a clone of the start page to it.
//! 12. Caller recognizes that the next page is available on SD, it will send
//!     a request for SD loader to pull it. It should be eventually available
//!     in a loaded queue.
//! 13. The loaded page is then passed to buffer instead of empty pages used before.
//! 14. At some point, midway through the sample, recording stops.
//! 15. Any new samples will be returned like before, except now they will not
//!     be dirty and thus just thrown away.
//!
//! # Flow starting from a loaded sample
//!
//! 1. The caller recognizes there is a sample available and it reads its length.
//! 2. The caller loads the first page, queues fetching of the second one, if there is one.
//! 3. The caller passes the first page to the buffer.
//! 4. Business as usual.
//!
//! # Working with samples shorter than a single page
//!
//! 1. Buffer gets an inpulse to reset midway through the first page.
//! 2. The caller takes page from the buffer, clones it for save queue, clones it for
//!    its own cache and passes it back to the buffer.

mod buffer;
mod cassette;
mod config;
mod manager;
mod page;
mod pool;

#[cfg(test)]
mod tests {

    use heapless::spsc::{Consumer, Producer};

    use super::*;

    #[test]
    fn full_flow_starting_from_nothing_with_long_recording() {
        use heapless::spsc::Queue;

        use cassette::{Cassette, CassetteId};
        use config::Config;
        use manager::Manager;
        use page::{Page, PageId, PageRequest};
        use pool::{Handle, Pool};

        let mut save_request_queue: Queue<Handle, 4> = Queue::new();
        let (mut save_request_producer, mut save_request_consumer) = save_request_queue.split();

        let mut save_request_first_page_queue: Queue<Page, 4> = Queue::new();
        let (mut save_request_first_page_producer, mut save_request_first_page_consumer) =
            save_request_first_page_queue.split();

        let mut load_request_queue: Queue<PageRequest, 4> = Queue::new();
        let (mut load_request_producer, mut load_request_consumer) = load_request_queue.split();

        let mut load_response_queue: Queue<Handle, 4> = Queue::new();
        let (mut load_response_producer, mut load_response_consumer) = load_response_queue.split();

        let mut config_queue: Queue<Config, 4> = Queue::new();
        let (mut config_producer, mut config_consumer) = config_queue.split();

        // Owned by page manager.
        let mut sd: [Option<Page>; 4] = [None, None, None, None];
        static mut POOL: Pool = Pool::new();
        let pool = unsafe { &mut POOL };

        // Owned by the caller. Running as DSP loop.
        let mut manager = Manager::new();

        // Loading metadata about the selected cassette from SD.
        // This will be solely based on the length of the file found on the file
        // system. There should be no metadata saved on side.
        manager.set_cassette(Cassette::new(1));
        manager.start_loading_next_page(&mut load_request_producer);

        // Page manager initializing the page and passing it to the caller.
        assert_and_handle_load_page_request(
            Some(PageRequest::Blank(PageId::new(CassetteId::new(1), 0))),
            pool,
            &mut load_request_consumer,
            &mut load_response_producer,
        );

        // Control loop issues request for recording.
        config_producer
            .enqueue(Config { recording: true })
            .ok()
            .unwrap();

        // Caller records into the first page until its full. This would span multiple
        // DSP ticks.
        loop {
            manager.process_configuration_updates(&mut config_consumer);

            if manager.is_waiting_for_page() {
                let acquired = manager.try_fetching_next_page(&mut load_response_consumer);
                if acquired {
                    manager.start_loading_next_page(&mut load_request_producer);
                }
            }

            manager.process(&mut [0.1; 32]);

            if manager.has_full_page() {
                manager.start_saving(
                    &mut save_request_producer,
                    &mut save_request_first_page_producer,
                );
                break;
            }
        }

        // Page manager answers request for loading of the next page and stores
        // the fully populated first page. The first page is passed by value since the
        // manager keeps the original for caching purposes.
        {
            assert_and_handle_load_page_request(
                Some(PageRequest::Blank(PageId::new(CassetteId::new(1), 1))),
                pool,
                &mut load_request_consumer,
                &mut load_response_producer,
            );
            assert_and_handle_page_save_request(
                Some(PageId::new(CassetteId::new(1), 0)),
                &mut sd,
                &mut save_request_first_page_consumer,
            );
            assert_and_handle_handle_save_request(None, &mut sd, pool, &mut save_request_consumer);
            assert_recorded(0, 0.1, &mut sd);
        }

        // Caller records into the second page until its full. This would span multiple
        // DSP ticks.
        loop {
            manager.process_configuration_updates(&mut config_consumer);

            if manager.is_waiting_for_page() {
                let acquired = manager.try_fetching_next_page(&mut load_response_consumer);
                if acquired {
                    manager.start_loading_next_page(&mut load_request_producer);
                }
            }

            manager.process(&mut [0.2; 32]);

            if manager.has_full_page() {
                manager.start_saving(
                    &mut save_request_producer,
                    &mut save_request_first_page_producer,
                );
                break;
            }
        }

        // Page manager responds to the request for the next blank page. It also saves
        // the fully populated page. This time the page is passed by reference to the
        // shared pool.
        {
            assert_and_handle_load_page_request(
                Some(PageRequest::Blank(PageId::new(CassetteId::new(1), 2))),
                pool,
                &mut load_request_consumer,
                &mut load_response_producer,
            );
            assert_and_handle_page_save_request(
                None,
                &mut sd,
                &mut save_request_first_page_consumer,
            );
            assert_and_handle_handle_save_request(
                Some(PageId::new(CassetteId::new(1), 1)),
                &mut sd,
                pool,
                &mut save_request_consumer,
            );
            assert_recorded(0, 0.1, &mut sd);
            assert_recorded(1, 0.2, &mut sd);
        }

        // Caller records into the third page, but is interrupted with a position reset.
        {
            for _ in 0..3 {
                manager.process_configuration_updates(&mut config_consumer);

                if manager.is_waiting_for_page() {
                    let acquired = manager.try_fetching_next_page(&mut load_response_consumer);
                    if acquired {
                        manager.start_loading_next_page(&mut load_request_producer);
                    }
                }

                manager.process(&mut [0.3; 32]);
            }

            manager.start_saving(
                &mut save_request_producer,
                &mut save_request_first_page_producer,
            );
            manager.reset_position();
        }

        // Page manager first handles a blank page request. This was sent from the manager
        // before it was reset. Then it saves the partially populated last page.
        {
            assert_and_handle_load_page_request(
                Some(PageRequest::Blank(PageId::new(CassetteId::new(1), 3))),
                pool,
                &mut load_request_consumer,
                &mut load_response_producer,
            );
            assert_and_handle_page_save_request(
                None,
                &mut sd,
                &mut save_request_first_page_consumer,
            );
            assert_and_handle_load_page_request(
                None,
                pool,
                &mut load_request_consumer,
                &mut load_response_producer,
            );
            assert_and_handle_handle_save_request(
                Some(PageId::new(CassetteId::new(1), 2)),
                &mut sd,
                pool,
                &mut save_request_consumer,
            );
            assert_recorded(0, 0.1, &mut sd);
            assert_recorded(1, 0.2, &mut sd);
            assert_recorded(2, 0.3, &mut sd);
        }

        // Caller records into the first page again.
        loop {
            manager.process_configuration_updates(&mut config_consumer);

            if manager.is_waiting_for_page() {
                let acquired = manager.try_fetching_next_page(&mut load_response_consumer);
                if acquired {
                    manager.start_loading_next_page(&mut load_request_producer);
                }
            }

            manager.process(&mut [0.4; 32]);

            if manager.has_full_page() {
                manager.start_saving(
                    &mut save_request_producer,
                    &mut save_request_first_page_producer,
                );
                break;
            }
        }

        // Page manager handles the load request for the previously stored recond page.
        // It also saves the new version of the first page.
        {
            assert_and_handle_load_page_request(
                Some(PageRequest::Load(PageId::new(CassetteId::new(1), 1))),
                pool,
                &mut load_request_consumer,
                &mut load_response_producer,
            );
            assert_and_handle_page_save_request(
                Some(PageId::new(CassetteId::new(1), 0)),
                &mut sd,
                &mut save_request_first_page_consumer,
            );
            assert_and_handle_handle_save_request(None, &mut sd, pool, &mut save_request_consumer);
            assert_recorded(0, 0.4, &mut sd);
            assert_recorded(1, 0.2, &mut sd);
            assert_recorded(2, 0.3, &mut sd);
        }

        // Control loop issues request for recording.
        {
            config_producer
                .enqueue(Config { recording: false })
                .ok()
                .unwrap();
        }

        // Caller records into the first page until its fully processed. This would span multiple
        // DSP ticks.
        loop {
            manager.process_configuration_updates(&mut config_consumer);

            if manager.is_waiting_for_page() {
                let acquired = manager.try_fetching_next_page(&mut load_response_consumer);
                if acquired {
                    manager.start_loading_next_page(&mut load_request_producer);
                }
            }

            manager.process(&mut [0.5; 32]);

            if manager.has_full_page() {
                manager.start_saving(
                    &mut save_request_producer,
                    &mut save_request_first_page_producer,
                );
                break;
            }
        }

        // Page manager handles request for loading of the next previously saved page.
        // No save request is expected since recording was disabled.
        {
            assert_and_handle_load_page_request(
                Some(PageRequest::Load(PageId::new(CassetteId::new(1), 2))),
                pool,
                &mut load_request_consumer,
                &mut load_response_producer,
            );
            assert_and_handle_page_save_request(
                None,
                &mut sd,
                &mut save_request_first_page_consumer,
            );
            assert_and_handle_handle_save_request(None, &mut sd, pool, &mut save_request_consumer);
            assert_recorded(0, 0.4, &mut sd);
            assert_recorded(1, 0.2, &mut sd);
            assert_recorded(2, 0.3, &mut sd);
        }
    }

    fn assert_recorded(page_index: usize, value: f32, sd: &mut [Option<page::Page>; 4]) {
        let first_sample = sd[page_index].as_ref().unwrap().data[0];
        assert_eq!(
            first_sample, value,
            "First sample of the given page has an unexpected value"
        );
    }

    fn assert_and_handle_handle_save_request(
        expected_handle_save_request: Option<page::PageId>,
        sd: &mut [Option<page::Page>; 4],
        pool: &mut pool::Pool,
        save_request_consumer: &mut Consumer<pool::Handle, 4>,
    ) {
        let received_handle_save_request = save_request_consumer.dequeue();
        if let Some(expected_handle_save_request) = expected_handle_save_request {
            let handle = received_handle_save_request.expect("No save request was received");
            assert_eq!(
                handle.page_ref().id(),
                expected_handle_save_request,
                "Unexpectde handle save request"
            );
            let index = handle.page_ref().index();
            sd[index] = Some(pool.take_page(handle));
        } else {
            assert!(
                received_handle_save_request.is_none(),
                "Unexpected handle save request"
            );
        }
    }

    fn assert_and_handle_page_save_request(
        expected_first_page_save_request: Option<page::PageId>,
        sd: &mut [Option<page::Page>; 4],
        save_request_first_page_consumer: &mut Consumer<page::Page, 4>,
    ) {
        let received_first_page_save_request = save_request_first_page_consumer.dequeue();
        if let Some(expected_first_page_save_request) = expected_first_page_save_request {
            let page = received_first_page_save_request.expect("No page save request was received");
            assert_eq!(
                page.id(),
                expected_first_page_save_request,
                "Unexpected page save request"
            );
            let index = page.index();
            sd[index] = Some(page);
        } else {
            assert!(
                received_first_page_save_request.is_none(),
                "Unexpected page save request"
            );
        }
    }

    fn assert_and_handle_load_page_request(
        expected_load_request: Option<page::PageRequest>,
        pool: &mut pool::Pool,
        load_request_consumer: &mut Consumer<page::PageRequest, 4>,
        load_response_producer: &mut Producer<pool::Handle, 4>,
    ) {
        let received_load_request = load_request_consumer.dequeue();

        if let Some(expected_load_request) = expected_load_request {
            let request = received_load_request.expect("No load request was received");
            assert_eq!(request, expected_load_request, "Unexpected load request");

            let handle = pool.new_page(expected_load_request.page_id());
            load_response_producer.enqueue(handle).ok().unwrap();
        } else {
            assert!(received_load_request.is_none(), "Unexpected load request");
        }
    }
}
