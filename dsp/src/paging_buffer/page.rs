//! Blobs of data.

/// Blob of data containing a part of audio sample.
pub(crate) struct Page {
    id: PageId,
}

impl Page {
    pub(crate) fn new(id: PageId) -> Self {
        Self { id }
    }

    pub(crate) fn id(&self) -> PageId {
        self.id
    }
}

/// Used to request blank or loaded page from another coroutine.
pub(crate) enum PageRequest {
    Load(PageId),
    Blank(PageId),
}

/// Unique identificator of a page.
#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) struct PageId {}

impl PageId {
    pub(crate) fn new(_cassette_index: usize, _page_index: usize) -> Self {
        Self {}
    }
}
