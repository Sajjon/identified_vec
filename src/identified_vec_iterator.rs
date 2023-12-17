use crate::is_identifiable_vec::IsIdentifiableVec;
use crate::vec::IdentifiedVec;
use std::fmt::Debug;
use std::hash::Hash;

/// An iterator over the items of an `IdentifiedVec`.
pub struct IdentifiedVecIterator<'a, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    identified_vec: &'a IdentifiedVec<I, E>,
    index: usize,
}

impl<'a, I, E> IdentifiedVecIterator<'a, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    pub fn new(identified_vec: &'a IdentifiedVec<I, E>) -> Self {
        Self {
            identified_vec,
            index: 0,
        }
    }
}

impl<'a, I, E> Iterator for IdentifiedVecIterator<'a, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.identified_vec.len() {
            let id = Some(&self.identified_vec.order[self.index]).unwrap();
            self.index += 1;
            return self.identified_vec.get(id);
        } else {
            None
        }
    }
}
