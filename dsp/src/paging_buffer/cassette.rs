//! Virtual cassette representation.

/// Represents a cassete with its recorded tracks and samples.
pub(crate) struct Cassette {
    pub id: CassetteId,
    pub length: usize,
}

impl Cassette {
    pub(crate) fn new(index: usize) -> Self {
        Self {
            id: CassetteId::new(index),
            length: 0,
        }
    }
}

/// Unique identificator of the given `Cassette`.
#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) struct CassetteId {
    index: usize,
}

impl CassetteId {
    pub(crate) fn new(index: usize) -> Self {
        Self { index }
    }
}
