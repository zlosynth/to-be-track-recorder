//! Blobs of data.

use super::cassette::CassetteId;

/// Blob of data containing a part of audio sample.
#[derive(Clone)]
pub(crate) struct Page {
    id: PageId,
    dirty: bool,
    // TODO: Use constants
    pub data: [f32; 512],
}

impl Page {
    pub(crate) fn new(id: PageId) -> Self {
        Self {
            id,
            dirty: false,
            data: [0.0; 512],
        }
    }

    pub(crate) fn id(&self) -> PageId {
        self.id
    }

    pub(crate) fn index(&self) -> usize {
        self.id.page_index
    }

    pub(crate) fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// Used to request blank or loaded page from another coroutine.
#[derive(Debug, PartialEq)]
pub(crate) enum PageRequest {
    Load(PageId),
    Blank(PageId),
}

impl PageRequest {
    pub(crate) fn page_id(&self) -> PageId {
        match self {
            PageRequest::Load(page_id) => *page_id,
            PageRequest::Blank(page_id) => *page_id,
        }
    }
}

/// Unique identificator of a page.
#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) struct PageId {
    page_index: usize,
}

impl PageId {
    pub(crate) fn page_index(&self) -> usize {
        self.page_index
    }
}

impl PageId {
    pub(crate) fn new(_cassette_id: CassetteId, page_index: usize) -> Self {
        Self { page_index }
    }
}
