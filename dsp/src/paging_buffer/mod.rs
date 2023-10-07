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
        {
            let request = load_request_consumer
                .dequeue()
                .expect("Must receive a load request");
            assert_eq!(
                request,
                PageRequest::Blank(PageId::new(CassetteId::new(1), 0)),
                "The first request must be for a blank page"
            );
            let handle = pool.new_page(PageId::new(CassetteId::new(1), 0));
            load_response_producer.enqueue(handle).ok().unwrap();
        }

        // Control loop issues request for recording.
        {
            config_producer
                .enqueue(Config { recording: true })
                .ok()
                .unwrap();
        }

        // Caller records into the first page until its full. This would span multiple
        // DSP ticks.
        loop {
            manager.process_configuration_updates(&mut config_consumer);

            // if caller.is_waiting_for_page() {
            //     let acquired = caller.try_fetching_next_page(&mut load_response_consumer);
            //     if acquired {
            //         caller.start_loading_next_page(&mut load_request_producer);
            //     }
            // }

            // caller.process(&mut [0.0; 32]);

            // if caller.has_full_page() {
            //     caller.start_saving(&mut save_request_producer);
            //     break;
            // }
        }

        // // SD manager
        // {
        //     let _request = load_request_consumer.dequeue().unwrap();
        //     load_response_producer
        //         .enqueue(Page::new(HARDCODED_PARENT, 1))
        //         .ok()
        //         .unwrap();

        //     let page_1 = save_request_consumer.dequeue().unwrap();
        //     sd[0] = Some(page_1);
        // }

        // // Caller records into the second page until its full. This would span multiple
        // // DSP ticks.
        // loop {
        //     caller.process_configuration_updates(&mut dsp_config_consumer);

        //     if caller.is_waiting_for_page() {
        //         let acquired = caller.try_fetching_next_page(&mut load_response_consumer);
        //         if acquired {
        //             caller.start_loading_next_page(&mut load_request_producer);
        //         }
        //     }

        //     caller.process(&mut [0.0; 32]);

        //     if caller.has_full_page() {
        //         caller.start_saving(&mut save_request_producer);
        //         break;
        //     }
        // }

        // // SD manager
        // {
        //     let _request = load_request_consumer.dequeue().unwrap();
        //     load_response_producer
        //         .enqueue(Page::new(HARDCODED_PARENT, 2))
        //         .ok()
        //         .unwrap();

        //     let page_2 = save_request_consumer.dequeue().unwrap();
        //     sd[1] = Some(page_2);
        // }

        // // Caller records into the third page, but is interrupted with a position reset.
        // {
        //     for _ in 0..3 {
        //         caller.process_configuration_updates(&mut dsp_config_consumer);

        //         if caller.is_waiting_for_page() {
        //             let acquired = caller.try_fetching_next_page(&mut load_response_consumer);
        //             if acquired {
        //                 caller.start_loading_next_page(&mut load_request_producer);
        //             }
        //         }

        //         caller.process(&mut [0.0; 32]);
        //     }

        //     caller.start_saving(&mut save_request_producer);
        //     caller.reset_position();
        // }

        // // SD manager
        // {
        //     let _request = load_request_consumer.dequeue().unwrap();
        //     load_response_producer
        //         .enqueue(Page::new(HARDCODED_PARENT, 1))
        //         .ok()
        //         .unwrap();

        //     let page_3 = save_request_consumer.dequeue().unwrap();
        //     sd[2] = Some(page_3);
        // }

        // // Caller records into the first page again
        // loop {
        //     caller.process_configuration_updates(&mut dsp_config_consumer);

        //     if caller.is_waiting_for_page() {
        //         let acquired = caller.try_fetching_next_page(&mut load_response_consumer);
        //         if acquired {
        //             caller.start_loading_next_page(&mut load_request_producer);
        //         }
        //     }

        //     caller.process(&mut [0.0; 32]);

        //     if caller.has_full_page() {
        //         caller.start_saving(&mut save_request_producer);
        //         break;
        //     }
        // }

        // // SD manager stores the first page and returns next.
        // {
        //     let page_1 = save_request_consumer.dequeue().unwrap();
        //     sd[0] = Some(page_1);

        //     let _request = load_request_consumer.dequeue().unwrap();
        //     load_response_producer
        //         .enqueue(sd[1].clone().unwrap())
        //         .ok()
        //         .unwrap();
        // }

        // // Control loop issues request for recording.
        // {
        //     dsp_config_producer.enqueue(DSPConfig::new()).ok().unwrap();
        // }

        // // Caller records into the first page until its full. This would span multiple
        // // DSP ticks.
        // loop {
        //     caller.process_configuration_updates(&mut dsp_config_consumer);

        //     if caller.is_waiting_for_page() {
        //         let acquired = caller.try_fetching_next_page(&mut load_response_consumer);
        //         if acquired {
        //             caller.start_loading_next_page(&mut load_request_producer);
        //         }
        //     }

        //     caller.process(&mut [0.0; 32]);

        //     if caller.has_full_page() {
        //         caller.start_saving(&mut save_request_producer);
        //         break;
        //     }
        // }
    }
}
