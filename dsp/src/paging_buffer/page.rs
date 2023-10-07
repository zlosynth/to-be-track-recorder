//! Blobs of data.

use super::cassette::CassetteId;

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

    pub(crate) fn index(&self) -> usize {
        self.id.page_index
    }
}

/// Used to request blank or loaded page from another coroutine.
#[derive(Debug, PartialEq)]
pub(crate) enum PageRequest {
    Load(PageId),
    Blank(PageId),
}

/// Unique identificator of a page.
#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) struct PageId {
    page_index: usize,
}

impl PageId {
    pub(crate) fn new(_cassette_id: CassetteId, page_index: usize) -> Self {
        Self { page_index }
    }
}
