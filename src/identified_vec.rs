use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Identifiable: Debug {
    type ID: Eq + Hash + Clone;
    fn id(&self) -> Self::ID;
}

/// A fast collection of unique elements preserving **insertion order**,
/// do NOT use this if you need something memory efficient, this collection
/// is not optimized for that. It is only optimized for speed.
pub struct IdentifiedVec<ID, Item>
where
    Item: Debug,
    ID: Eq + Hash + Clone,
{
    /// The holder of the insertion order
    order: Vec<ID>,

    /// Lookup table for the index inside `order` vec, with `ID` as key.
    /// this allows for constant time lookup of the index of an ID inside
    /// `order`
    id_to_index_in_order: HashMap<ID, usize>,

    /// The storage of items.
    items: HashMap<ID, Item>,

    _id_of_item: fn(&Item) -> ID,
}

impl<ID, Item> IdentifiedVec<ID, Item>
where
    ID: Eq + Hash + Clone,
    Item: Debug,
{
    /// Constructs a new, empty `IdentifiedVec<ID, Item>` with the specified
    /// `id of item` closure
    pub fn new_identifying_item(id_of_item: fn(&Item) -> ID) -> Self {
        Self {
            order: Vec::new(),
            id_to_index_in_order: HashMap::new(),
            items: HashMap::new(),
            _id_of_item: id_of_item,
        }
    }
}

impl<Item> IdentifiedVec<Item::ID, Item>
where
    Item: Identifiable,
{
    /// Constructs a new, empty `IdentifiedVec<ID, Item>`, using `id()` on `Item`
    /// as id function.
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            id_to_index_in_order: HashMap::new(),
            items: HashMap::new(),
            _id_of_item: |i| i.id(),
        }
    }
}

pub type IdentifiedVecOf<Item> = IdentifiedVec<<Item as Identifiable>::ID, Item>;

impl<ID, Item> IdentifiedVec<ID, Item>
where
    ID: Eq + Hash + Clone,
    Item: Debug,
{
    /// Returns the number of elements in the `IdentifiedVec`, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        if cfg!(debug_assertions) {
            assert_eq!(self.order.len(), self.items.len());
            assert_eq!(self.id_to_index_in_order.len(), self.items.len());
        }
        self.order.len()
    }

    fn id(&self, of: &Item) -> ID {
        (self._id_of_item)(of)
    }

    fn index_of_id(&self, id: &ID) -> Option<&usize> {
        self.id_to_index_in_order.get(id)
    }

    /// Looks up the `index` (position) of `item` if any.
    fn index_of_existing(&self, item: &Item) -> Option<&usize> {
        self.index_of_id(&self.id(item))
    }

    fn contains(&self, item: &Item) -> bool {
        self.index_of_existing(item).is_some()
    }

    fn update_value(
        &mut self,
        item: Item,
        for_key: ID,
        inserting_at: usize,
    ) -> IdentifiedVecInsertionResult {
        if let Some(_) = self.order.get(inserting_at) {
            return IdentifiedVecInsertionResult::already_present(inserting_at);
        }
        println!("🔮 Not already present: {:?} at index {inserting_at}", item);
        self.order.insert(inserting_at, for_key.clone());
        self.id_to_index_in_order
            .insert(for_key.clone(), inserting_at);
        self.items.insert(for_key, item);
        return IdentifiedVecInsertionResult::inserted_new(inserting_at);
    }

    /// Insert a new member to this array at the specified index, if the array doesn't already contain
    /// it.
    ///
    /// - Parameter item: The element to insert.
    /// - Returns: A pair `(inserted, index)`, where `inserted` is a Boolean value indicating whether
    ///   the operation added a new element, and `index` is the index of `item` in the resulting
    ///   array. If `inserted` is true, then the returned `index` may be different from the index
    ///   requested.
    ///
    /// - Complexity: The operation is expected to perform amortized O(`self.count`) copy, hash, and
    ///   compare operations on the `ID` type, if it implements high-quality hashing. (Insertions need
    ///   to make room in the storage array to add the inserted element.)
    #[inline]
    pub fn insert(&mut self, item: Item, at: usize) -> IdentifiedVecInsertionResult {
        let id = self.id(&item);
        if let Some(existing) = self.index_of_id(&id) {
            println!("🙅 Already present: {:?} at index {existing}", item);
            return IdentifiedVecInsertionResult::already_present(existing.clone());
        }
        self.update_value(item, id, at);
        IdentifiedVecInsertionResult::inserted_new(at)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IdentifiedVecInsertionResult {
    pub inserted: bool,
    pub index: usize,
}

impl IdentifiedVecInsertionResult {
    pub(crate) fn inserted_new(at: usize) -> Self {
        Self {
            inserted: true,
            index: at,
        }
    }
    pub(crate) fn already_present(at: usize) -> Self {
        Self {
            inserted: false,
            index: at,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::identified_vec::IdentifiedVecInsertionResult;

    use super::{Identifiable, IdentifiedVec, IdentifiedVecOf};
    use rand::Rng;

    #[derive(Debug, Eq, PartialEq, Clone, Copy)]
    struct User {
        id: u16,
        year_of_birth: u16,
    }
    impl User {
        fn new(year_of_birth: u16) -> Self {
            let mut rng = rand::thread_rng();
            Self {
                id: rng.gen(),
                year_of_birth,
            }
        }
        fn alex() -> Self {
            Self::new(1987)
        }
        fn klara() -> Self {
            Self::new(1990)
        }
        fn stella() -> Self {
            Self::new(2020)
        }
    }
    impl Identifiable for User {
        type ID = u16;
        fn id(&self) -> Self::ID {
            self.id
        }
    }
    type SUT = IdentifiedVecOf<User>;

    #[test]
    fn new_is_empty() {
        assert_eq!(SUT::new().len(), 0);
    }

    #[test]
    fn insertion() {
        let mut sut = SUT::new();
        let user = User::alex();
        sut.insert(user, 0);
        assert_eq!(sut.len(), 1);
        assert_eq!(
            sut.insert(user, 0),
            IdentifiedVecInsertionResult::already_present(0)
        );
        assert_eq!(sut.len(), 1);
    }
}
