struct Page {}

impl Page {
    fn new() -> Self {
        Self {}
    }
}

enum PageRequest {
    Load(PageId),
    Blank(PageId),
}

struct PageId {}
