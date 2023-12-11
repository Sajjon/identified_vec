#[cfg(feature = "serde")]
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum IdentifiedVecOfSerdeFailure {
    #[error("Duplicate element at offset {0}")]
    DuplicateElementsAtIndex(usize),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum InsertionFailure {
    #[error("Duplicate element with same id found")]
    ElementWithSameIDFound,
    #[error("Duplicate element with same value found")]
    ElementWithSameValueFound,
}
