pub(crate) struct Page {}

impl Page {
    fn new() -> Self {
        Self {}
    }
}

pub(crate) enum PageRequest {
    Load(PageId),
    Blank(PageId),
}

pub(crate) struct PageId {}
