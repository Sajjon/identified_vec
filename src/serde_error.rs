#[cfg(feature = "serde")]
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum IdentifiedVecOfSerdeFailure {
    #[error("Duplicate element at offset {0}")]
    DuplicateElementsAtIndex(usize),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Element with that id not found in collection")]
    ExpectedElementNotPresent,
    #[error("Duplicate element with same value found")]
    ElementWithSameValueFound,
    #[error("Duplicate element with same ID found")]
    ElementWithSameIDFound,
}
