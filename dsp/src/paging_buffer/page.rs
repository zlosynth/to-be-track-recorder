//! Blobs of data.

/// Blob of data containing a part of audio sample.
pub(crate) struct Page {}

impl Page {
    fn new() -> Self {
        Self {}
    }
}

/// Used to request blank or loaded page from another coroutine.
pub(crate) enum PageRequest {
    Load(PageId),
    Blank(PageId),
}

/// Unique identificator of a page.
pub(crate) struct PageId {}
