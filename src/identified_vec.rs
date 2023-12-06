use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Add;

pub trait Identifiable: Debug {
    type ID: Eq + Hash + Clone + Debug;
    fn id(&self) -> Self::ID;
}

/// A fast collection of unique elements preserving **insertion order**,
/// do NOT use this if you need something memory efficient, this collection
/// is not optimized for that. It is only optimized for speed.
pub struct IdentifiedVec<ID, Item>
where
    Item: Debug,
    ID: Eq + Hash + Clone + Debug,
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
    ID: Eq + Hash + Clone + Debug,
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
    ID: Eq + Hash + Clone + Debug,
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

    #[cfg(debug_assertions)]
    pub fn debug_str(&self) -> String {
        format!(
            "order: {:?}\nid_to_index_in_order: {:?}\nitems: {:?}",
            self.order, self.id_to_index_in_order, self.items
        )
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self) {
        println!("{}", self.debug_str());
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

    fn _offset_indices_of_if_needed(
        id_to_index_in_order: &mut HashMap<ID, usize>,
        order: &Vec<ID>,
        index: usize,
    ) {
        match order.get(index).cloned() {
            Some(existing) => {
                let key = &existing;
                let current_index = id_to_index_in_order.get(key).expect("duh");

                let new_index = current_index.add(1);

                // RECURSIVE CALL
                Self::_offset_indices_of_if_needed(id_to_index_in_order, order, new_index);
                println!("⚡️ Moving ID={:?} to index={}", existing, new_index);

                *id_to_index_in_order.get_mut(key).expect("checked already") = new_index;
            }
            None => {
                // println!("✨ Nothing found at index={index}.");
            }
        }
    }
    fn offset_indices_if_needed(&mut self, index: usize) {
        let mut modified = self.id_to_index_in_order.clone();
        Self::_offset_indices_of_if_needed(&mut modified, &self.order, index);
        self.id_to_index_in_order = modified;
    }

    fn update_value(&mut self, item: Item, for_key: ID, inserting_at: usize) -> (bool, usize) {
        self.offset_indices_if_needed(inserting_at);
        println!("➕ Adding: {:?} at index {inserting_at}", item);
        self.order.insert(inserting_at, for_key.clone());
        self.id_to_index_in_order
            .insert(for_key.clone(), inserting_at);

        self.items.insert(for_key, item);
        (true, inserting_at)
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
    pub fn insert(&mut self, item: Item, at: usize) -> (bool, usize) {
        println!(
            "\n\n{}\n☑️ START OF INSERT\n📦Arguments:\nitem: {:?}, inserting at:{at}\n🔮State:\n{}\n{}",
            "=".repeat(60),
            item,
            self.debug_str(),
            "*".repeat(60),
        );
        let id = self.id(&item);
        if let Some(existing) = self.index_of_id(&id) {
            println!(
                "❌ Skipped adding: {:?} at index {at}, already present at {existing}",
                item
            );
            let output = (false, existing.clone());
            println!(
                "✅ END OF INSERT\noutput: {:?}\n{}\n✅✅✅",
                output,
                self.debug_str()
            );
            return output;
        }
        self.update_value(item, id, at);
        let output = (true, at);
        println!(
            "✅ END OF INSERT\noutput: {:?}\n{}\n✅✅✅",
            output,
            self.debug_str()
        );
        return output;
    }

    fn end_index(&self) -> usize {
        self.len()
    }

    #[inline]
    pub fn append(&mut self, item: Item) -> (bool, usize) {
        self.insert(item, self.end_index())
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<&Item> {
        let mut items_ordered = Vec::<&Item>::new();
        for id in &self.order {
            items_ordered.push(self.items.get(id).expect("item"))
        }
        items_ordered
    }
}

#[cfg(test)]
mod tests {

    use super::{Identifiable, IdentifiedVec, IdentifiedVecOf};
    use maplit::hashmap;
    use rand::Rng;

    #[derive(Debug, Eq, PartialEq, Clone)]
    struct User {
        name: String,
    }
    impl User {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }

        fn alex() -> Self {
            Self::new("Alex")
        }

        fn klara() -> Self {
            Self::new("Klara")
        }

        fn zelda() -> Self {
            Self::new("Zelda")
        }

        fn stella() -> Self {
            Self::new("Stella")
        }
    }

    impl Identifiable for User {
        type ID = String;
        fn id(&self) -> Self::ID {
            self.name.clone().to_lowercase()
        }
    }
    type SUT = IdentifiedVecOf<User>;

    #[test]
    fn new_is_empty() {
        assert_eq!(SUT::new().len(), 0);
    }

    #[test]
    fn insertion_duplicates_same_index_not_allowed() {
        let mut sut = SUT::new();
        let user = User::alex();
        sut.insert(user.clone(), 0);
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.insert(user, 0), (false, 0));
        assert_eq!(sut.len(), 1);
    }

    #[test]
    fn insertion_duplicates_different_indices_does_not_lead_to_duplicates() {
        let mut sut = SUT::new();
        let user = User::alex();
        sut.insert(user.clone(), 0);
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.insert(user, 1), (false, 0));
        assert_eq!(sut.len(), 1);
    }

    #[test]
    fn add_two_insert_third_in_middle_order_is_maintained() {
        let mut sut = SUT::new();
        let alex = User::alex();
        let klara: User = User::klara();
        let stella = User::stella();
        sut.append(alex.clone());
        sut.append(klara.clone());
        sut.insert(stella.clone(), 1);
        assert_eq!(sut.to_vec(), vec![&alex, &stella, &klara]);
    }

    #[test]
    fn insert_at_0_three_times_order_is_maintained() {
        let mut sut = SUT::new();
        let alex = User::alex();
        let klara: User = User::klara();
        let stella = User::stella();
        let zelda = User::zelda();
        sut.insert(alex.clone(), 0);
        sut.insert(klara.clone(), 0);
        sut.insert(zelda.clone(), 0);
        sut.insert(stella.clone(), 0);
        assert_eq!(sut.to_vec(), vec![&stella, &zelda, &klara, &alex]);
        assert_eq!(
            sut.id_to_index_in_order,
            hashmap! {
                "stella".to_string() => 0,
               "zelda".to_string() => 1,
               "klara".to_string() => 2,
               "alex".to_string() => 3,
            }
        );
        sut.debug();
    }
}
