#[cfg(feature = "serde")]
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum IdentifiedVecOfSerdeFailure {
    #[error("Duplicate element at offset {0}")]
    DuplicateElementsAtIndex(usize),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Element with that id: `{0}` not found in collection")]
    ExpectedElementNotPresent(String),
    #[error("Duplicate element with same value: `{0}` found")]
    ElementWithSameValueFound(String),
    #[error("Duplicate element with same ID: `{0}` found")]
    ElementWithSameIDFound(String),
}
