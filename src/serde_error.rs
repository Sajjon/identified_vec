#[cfg(feature = "serde")]
#[derive(thiserror::Error, Debug, Clone)]
pub enum IdentifiedVecOfSerdeFailure {
    #[error("Duplicate element at offset {0}")]
    DuplicateElementsAtIndex(usize),
}
