#[cfg(feature = "serde")]
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum IdentifiedVecOfSerdeFailure {
    #[error("Duplicate element at offset {0}")]
    DuplicateElementsAtIndex(usize),
}
