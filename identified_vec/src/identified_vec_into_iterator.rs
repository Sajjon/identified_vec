use crate::is_identifiable_vec::IsIdentifiableVec;
use crate::vec::IdentifiedVec;
use std::fmt::Debug;
use std::hash::Hash;

/// An owning iterator over the items of an `IdentifiedVec`.
pub struct IdentifiedVecIntoIterator<I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    identified_vec: IdentifiedVec<I, E>,
}

impl<I, E> IdentifiedVecIntoIterator<I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    pub fn new(identified_vec: IdentifiedVec<I, E>) -> Self {
        Self { identified_vec }
    }
}

impl<I, E> Iterator for IdentifiedVecIntoIterator<I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.identified_vec.len() == 0 {
            return None;
        }
        let result = self.identified_vec.remove_at(0);
        Some(result)
    }
}
