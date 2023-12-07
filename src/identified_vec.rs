use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, SubAssign};

pub trait Identifiable: Debug + Clone {
    type ID: Eq + Hash + Clone + Debug;
    fn id(&self) -> Self::ID;
}

/// A fast collection of unique elements preserving **insertion order**,
/// do NOT use this if you need something memory efficient, this collection
/// is not optimized for that. It is only optimized for speed.
pub struct IdentifiedVec<ID, Item>
where
    Item: Debug + Clone,
    ID: Eq + Hash + Clone + Debug,
{
    /// The holder of the insertion order
    order: Vec<ID>,

    /// The storage of items.
    items: HashMap<ID, Item>,

    _id_of_item: fn(&Item) -> ID,
}

impl<ID, Item> IdentifiedVec<ID, Item>
where
    Item: Debug + Clone,
    ID: Eq + Hash + Clone + Debug,
{
    /// Constructs a new, empty `IdentifiedVec<ID, Item>` with the specified
    /// `id of item` closure
    pub fn new_identifying_item(id_of_item: fn(&Item) -> ID) -> Self {
        Self {
            order: Vec::new(),
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
            items: HashMap::new(),
            _id_of_item: |i| i.id(),
        }
    }

    /// Creates a new identified_vec from the elements in the given sequence.
    ///
    /// You use this initializer to create an identified_vec when you have a sequence of elements with unique
    /// ids. Passing a sequence with duplicate ids to this initializer results in a runtime error.
    ///
    /// - Parameter elements: A sequence of elements to use for the new identified_vec. Every element in
    ///   `elements` must have a unique id.
    /// - Returns: A new identified_vec initialized with the elements of `elements`.
    /// - Precondition: The sequence must not have duplicate ids.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    #[inline]
    pub fn from_iter<I>(unique_elements: I) -> Self
    where
        I: IntoIterator<Item = Item>,
    {
        let mut _self = Self::new();
        unique_elements
            .into_iter()
            .for_each(|e| _ = _self.append(e));
        return _self;
    }
}

pub type IdentifiedVecOf<Item> = IdentifiedVec<<Item as Identifiable>::ID, Item>;

impl<ID, Item> IdentifiedVec<ID, Item>
where
    ID: Eq + Hash + Clone + Debug,
    Item: Debug + Clone,
{
    /// Returns the number of elements in the `IdentifiedVec`, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        if cfg!(debug_assertions) {
            assert_eq!(self.order.len(), self.items.len());
        }
        self.order.len()
    }

    pub fn ids(&self) -> &Vec<ID> {
        &self.order
    }

    #[cfg(debug_assertions)]
    pub fn debug_str(&self) -> String {
        format!("order: {:?}\nitems: {:?}", self.order, self.items)
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self) {
        println!("{}", self.debug_str());
    }

    fn id(&self, of: &Item) -> ID {
        (self._id_of_item)(of)
    }

    #[inline]
    pub fn index_of_id(&self, id: &ID) -> Option<usize> {
        self.order.iter().position(|i| i == id)
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

    fn _update_value(&mut self, item: Item, for_key: ID, inserting_at: usize) {
        println!(
            "\n\n{}\n☑️ START OF INSERT\n📦Arguments:\nitem: {:?}, inserting at:{inserting_at}\n🔮State:\n{}\n{}",
            "=".repeat(60),
            item,
            self.debug_str(),
            "*".repeat(60),
        );
        println!("➕ Adding: {:?} at index {inserting_at}", item);
        self.order.insert(inserting_at, for_key.clone());
        self.items.insert(for_key, item);
    }

    /// Insert a new member to this identified_vec at the specified index, if the identified_vec doesn't already contain
    /// it.
    ///
    /// - Parameter item: The element to insert.
    /// - Returns: A pair `(inserted, index)`, where `inserted` is a Boolean value indicating whether
    ///   the operation added a new element, and `index` is the index of `item` in the resulting
    ///   identified_vec. If `inserted` is true, then the returned `index` may be different from the index
    ///   requested.
    ///
    /// - Complexity: The operation is expected to perform amortized O(`self.count`) copy, hash, and
    ///   compare operations on the `ID` type, if it implements high-quality hashing. (Insertions need
    ///   to make room in the storage identified_vec to add the inserted element.)
    #[inline]
    pub fn insert(&mut self, item: Item, at: usize) -> (bool, usize) {
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
        self._update_value(item, id, at);
        let output = (true, at);
        println!(
            "✅ END OF INSERT\noutput: {:?}\n{}\n✅✅✅",
            output,
            self.debug_str()
        );
        return output;
    }

    #[inline]
    pub fn update_or_insert(&mut self, item: Item, at: usize) {
        let id = self.id(&item);
        self._update_value(item, id, at)
    }

    fn end_index(&self) -> usize {
        self.len()
    }

    #[inline]
    pub fn append(&mut self, item: Item) -> (bool, usize) {
        self.insert(item, self.end_index())
    }

    #[inline]
    pub fn update_or_append(&mut self, item: Item) {
        self.update_or_insert(item, self.end_index())
    }

    #[inline]
    pub fn elements(&self) -> Vec<Item> {
        let mut items_ordered = Vec::<Item>::new();
        for id in &self.order {
            items_ordered.push(self.items.get(id).expect("item").clone());
        }
        items_ordered
    }

    #[inline]
    pub fn contains(&self, item: &Item) -> bool {
        self.items.contains_key(&self.id(&item))
    }

    #[inline]
    pub fn get(&self, id: &ID) -> Option<&Item> {
        self.items.get(id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: &ID) -> Option<&mut Item> {
        self.items.get_mut(id)
    }

    #[inline]
    pub fn remove(&mut self, id: &ID) -> Option<Item> {
        match self.index_of_id(id) {
            Some(index) => {
                self.order.remove(index);
                return self.items.remove(id);
            }
            None => {
                assert!(!self.items.contains_key(id));
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::{
        cell::{Cell, RefCell},
        fmt::Debug,
    };

    use super::{Identifiable, IdentifiedVec, IdentifiedVecOf};
    use maplit::hashmap;

    #[derive(Eq, PartialEq, Clone)]
    pub struct User {
        pub id: i32,
        pub name: RefCell<String>,
    }
    impl User {
        fn new(id: i32, name: &str) -> Self {
            if name.is_empty() {
                panic!("name cannot be empty")
            }
            Self {
                id,
                name: RefCell::new(name.to_string()),
            }
        }

        pub fn blob() -> Self {
            User::new(1, "Blob")
        }
        pub fn blob_jr() -> Self {
            User::new(2, "Blob, Jr.")
        }
        pub fn blob_sr() -> Self {
            User::new(3, "Blob, Sr.")
        }
    }
    impl Debug for User {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("User")
                .field("id", &self.id)
                .field("name", &self.name.borrow())
                .finish()
        }
    }
    impl Identifiable for User {
        type ID = i32;
        fn id(&self) -> Self::ID {
            self.id
        }
    }

    impl Identifiable for i32 {
        type ID = i32;
        fn id(&self) -> Self::ID {
            return self.clone();
        }
    }
    type SUT = IdentifiedVecOf<i32>;
    type Users = IdentifiedVecOf<User>;

    #[test]
    fn new_is_empty() {
        assert_eq!(SUT::new().len(), 0);
    }

    #[test]
    fn ids() {
        let identified_vec = SUT::from_iter([1, 2, 3]);
        assert_eq!(identified_vec.ids(), &[1, 2, 3])
    }

    #[test]
    fn elements() {
        let vec = vec![User::blob(), User::blob_jr(), User::blob_sr()];
        let identified_vec = Users::from_iter(vec.clone());
        assert_eq!(identified_vec.elements(), vec);
    }

    #[test]
    fn get() {
        let vec = vec![User::blob(), User::blob_jr(), User::blob_sr()];
        let mut identified_vec = Users::from_iter(vec.clone());
        assert_eq!(identified_vec.get(&1), Some(&User::blob()));
        assert_eq!(identified_vec.get(&2), Some(&User::blob_jr()));
        assert_eq!(identified_vec.get(&3), Some(&User::blob_sr()));

        // 1
        let mut id: &i32 = &1;
        identified_vec
            .get_mut(id)
            .unwrap()
            .name
            .borrow_mut()
            .push_str(", Esq.");
        assert_eq!(
            identified_vec.get(id),
            Some(&User::new(id.clone(), "Blob, Esq."))
        );

        // 2
        id = &2;
        identified_vec
            .get_mut(id)
            .unwrap()
            .name
            .borrow_mut()
            .drain(4..9);
        assert_eq!(identified_vec.get(id), Some(&User::new(id.clone(), "Blob")));

        // 3
        id = &3;
        identified_vec
            .get_mut(id)
            .unwrap()
            .name
            .borrow_mut()
            .drain(4..9);
        assert_eq!(identified_vec.get(id), Some(&User::new(id.clone(), "Blob")));

        identified_vec.remove(id);
        assert_eq!(identified_vec.get(id), None);
        identified_vec.append(User::new(4, "Blob, Sr."));
        assert_eq!(
            identified_vec.elements(),
            [
                User::new(1, "Blob, Esq."),
                User::new(2, "Blob"),
                User::new(4, "Blob, Sr."),
            ]
        );
    }

    #[test]
    fn contains_element() {
        let identified_vec = SUT::from_iter([1, 2, 3]);
        assert!(identified_vec.contains(&2))
    }

    #[test]
    fn index_id() {
        let identified_vec = SUT::from_iter([1, 2, 3]);
        assert_eq!(identified_vec.index_of_id(&2), Some(1));
    }

    #[test]
    fn remove_element() {
        let mut identified_vec = SUT::from_iter([1, 2, 3]);
        assert_eq!(identified_vec.remove(&2), Some(2));
        assert_eq!(identified_vec.elements(), [1, 3])
    }

    /*
       #[test]
       fn RemoveId() {
           let mut identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(identified_vec.remove(id: 2), 2)
           assert_eq!(identified_vec, [1, 3])
       }

       #[test]
       fn Codable() {
           let identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(
               try JSONDecoder().decode(IdentifiedArray.self, from: JSONEncoder().encode(identified_vec)),
               identified_vec
           )
           assert_eq!(
               try JSONDecoder().decode(IdentifiedArray.self, from: Data("[1,2,3]".utf8)),
               identified_vec
           )
           XCTAssertThrowsError(
               try JSONDecoder().decode(IdentifiedArrayOf<Int>.self, from: Data("[1,1,1]".utf8))
           ) { error in
               guard case let DecodingError.dataCorrupted(ctx) = error
               else { return XCTFail() }
               assert_eq!(ctx.debugDescription, "Duplicate element at offset 1")
           }
       }

       #[test]
       fn CustomDebugStringConvertible() {
           let identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(identified_vec.debugDescription, "IdentifiedArray<Int>([1, 2, 3])")
       }

       #[test]
       fn CustomReflectable() {
           let identified_vec = SUT::from_iter([1, 2, 3]);
           let mirror = Mirror(reflecting: identified_vec)
           assert_eq!(mirror.displayStyle, .collection)
           XCTAssert(mirror.superclassMirror == nil)
           assert_eq!(mirror.children.compactMap { $0.label }.isEmpty, true)
           assert_eq!(mirror.children.map { $0.value as? Int }, identified_vec.map { $0 })
       }

       #[test]
       fn CustomStringConvertible() {
           let identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(identified_vec.description, "[1, 2, 3]")
       }

       #[test]
       fn Hashable() {
           let identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(Set([identified_vec]), Set([identified_vec, identified_vec]))
       }

       #[test]
       fn InitUncheckedUniqueElements() {
           let identified_vec = IdentifiedArray(uncheckedUniqueElements: [1, 2, 3])
           assert_eq!(identified_vec, [1, 2, 3])
       }

       #[test]
       fn InitUniqueElementsSelf() {
           let identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(IdentifiedArray(uniqueElements: identified_vec), [1, 2, 3])
       }

       #[test]
       fn InitUniqueElementsSubSequence() {
           let identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(IdentifiedArray(uniqueElements: identified_vec[...]), [1, 2, 3])
       }

       #[test]
       fn InitUniqueElements() {
           let identified_vec = IdentifiedArray(uniqueElements: [1, 2, 3])
           assert_eq!(identified_vec, [1, 2, 3])
       }

       #[test]
       fn SelfInit() {
           let identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(IdentifiedArray(identified_vec), [1, 2, 3])
       }

       #[test]
       fn SubsequenceInit() {
           let identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(IdentifiedArray(identified_vec[...]), [1, 2, 3])
       }

       #[test]
       fn InitIDUniquingElements() {
           struct Model: Equatable {
               let id: Int
               let data: String
           }
           // Choose first element
           do {
               let identified_vec = IdentifiedArray(
                   [
                       Model(id: 1, data: "A"),
                       Model(id: 2, data: "B"),
                       Model(id: 1, data: "AAAA"),
                   ],
                   id: \.id
               ) { lhs, _ in lhs }

               assert_eq!(
                   identified_vec,
                   IdentifiedArray(
                       uniqueElements: [
                           Model(id: 1, data: "A"),
                           Model(id: 2, data: "B"),
                       ],
                       id: \.id
                   )
               )
           }
           // Choose later element
           do {
               let identified_vec = IdentifiedArray(
                   [
                       Model(id: 1, data: "A"),
                       Model(id: 2, data: "B"),
                       Model(id: 1, data: "AAAA"),
                   ],
                   id: \.id
               ) { _, rhs in rhs }

               assert_eq!(
                   identified_vec,
                   IdentifiedArray(
                       uniqueElements: [
                           Model(id: 1, data: "AAAA"),
                           Model(id: 2, data: "B"),
                       ], id: \.id))
           }
       }

       #[test]
       fn InitUniquingElements() {
           struct Model: Equatable, Identifiable {
               let id: Int
               let data: String
           }
           // Choose first element
           do {
               let identified_vec = IdentifiedArray(
                   [
                       Model(id: 1, data: "A"),
                       Model(id: 2, data: "B"),
                       Model(id: 1, data: "AAAA"),
                   ]
               ) { lhs, _ in lhs }

               assert_eq!(
                   identified_vec,
                   IdentifiedArray(
                       uniqueElements: [
                           Model(id: 1, data: "A"),
                           Model(id: 2, data: "B"),
                       ]
                   )
               )
           }
           // Choose later element
           do {
               let identified_vec = IdentifiedArray(
                   [
                       Model(id: 1, data: "A"),
                       Model(id: 2, data: "B"),
                       Model(id: 1, data: "AAAA"),
                   ]
               ) { _, rhs in rhs }

               assert_eq!(
                   identified_vec,
                   IdentifiedArray(
                       uniqueElements: [
                           Model(id: 1, data: "AAAA"),
                           Model(id: 2, data: "B"),
                       ]
                   )
               )
           }
       }

       #[test]
       fn Append() {
           let mut identified_vec = SUT::from_iter([1, 2, 3]);
           var (inserted, index) = identified_vec.append(4)
           assert_eq!(inserted, true)
           assert_eq!(index, 3)
           assert_eq!(identified_vec, [1, 2, 3, 4])
           (inserted, index) = identified_vec.append(2)
           assert_eq!(inserted, false)
           assert_eq!(index, 1)
           assert_eq!(identified_vec, [1, 2, 3, 4])
       }

       #[test]
       fn AppendContentsOf() {
           let mut identified_vec = SUT::from_iter([1, 2, 3]);
           identified_vec.append(contentsOf: [1, 4, 3, 5])
           assert_eq!(identified_vec, [1, 2, 3, 4, 5])
       }

       #[test]
       fn Insert() {
           let mut identified_vec = SUT::from_iter([1, 2, 3]);
           var (inserted, index) = identified_vec.insert(0, at: 0)
           assert_eq!(inserted, true)
           assert_eq!(index, 0)
           assert_eq!(identified_vec, [0, 1, 2, 3])
           (inserted, index) = identified_vec.insert(2, at: 0)
           assert_eq!(inserted, false)
           assert_eq!(index, 2)
           assert_eq!(identified_vec, [0, 1, 2, 3])
       }

       #[test]
       fn UpdateAt() {
           let mut identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(identified_vec.update(2, at: 1), 2)
       }

       #[test]
       fn UpdateOrAppend() {
           let mut identified_vec = SUT::from_iter([1, 2, 3]);
           assert_eq!(identified_vec.updateOrAppend(4), nil)
           assert_eq!(identified_vec, [1, 2, 3, 4])
           assert_eq!(identified_vec.updateOrAppend(2), 2)
       }

       #[test]
       fn UpdateOrInsert() {
           let mut identified_vec = SUT::from_iter([1, 2, 3]);
           var (originalMember, index) = identified_vec.updateOrInsert(0, at: 0)
           assert_eq!(originalMember, nil)
           assert_eq!(index, 0)
           assert_eq!(identified_vec, [0, 1, 2, 3])
           (originalMember, index) = identified_vec.updateOrInsert(2, at: 0)
           assert_eq!(originalMember, 2)
           assert_eq!(index, 2)
           assert_eq!(identified_vec, [0, 1, 2, 3])
       }

       #[test]
       fn Partition() {
           let mut identified_vec: IdentifiedArray = [1, 2]

           let index = identified_vec.partition { $0.id == 1 }

           assert_eq!(index, 1)
           assert_eq!(identified_vec, [2, 1])

           for id in identified_vec.ids {
               assert_eq!(id, identified_vec[id: id]?.id)
           }
       }

       #[test]
       fn MoveFromOffsetsToOffset() {
           let mut identified_vec = SUT::from_iter([1, 2, 3]);
           identified_vec.move(fromOffsets: [0, 2], toOffset: 0)
           assert_eq!(identified_vec, [1, 3, 2])

           identified_vec = [1, 2, 3]
           identified_vec.move(fromOffsets: [0, 2], toOffset: 1)
           assert_eq!(identified_vec, [1, 3, 2])

           identified_vec = [1, 2, 3]
           identified_vec.move(fromOffsets: [0, 2], toOffset: 2)
           assert_eq!(identified_vec, [2, 1, 3])
       }

       #[test]
       fn RemoveAtOffsets() {
           let mut identified_vec = SUT::from_iter([1, 2, 3]);
           identified_vec.remove(atOffsets: [0, 2])
           assert_eq!(identified_vec, [2])
       }
       #[test]
       fn Equatable() {
           struct Foo: Identifiable, Equatable {
               var id: String = "id"
               var value: String = "value"
           }
           // Create arrays using all of the initializers
           var arrays: [IdentifiedArray<String, Foo>] = [
               IdentifiedArray<String, Foo>(),
               IdentifiedArray<String, Foo>(uncheckedUniqueElements: [], id: \.id),
               IdentifiedArray<String, Foo>(uniqueElements: [], id: \.id),
               IdentifiedArray<String, Foo>(uncheckedUniqueElements: []),
               IdentifiedArray<String, Foo>(uniqueElements: []),
           ]
           arrays.forEach({ lhs in
               arrays.forEach({ rhs in
                   assert_eq!(lhs, rhs)
               })
           })
           // add an element to each identified_vec
           arrays.indices.forEach({
               arrays[$0].append(Foo())
           })
           arrays.forEach({ lhs in
               arrays.forEach({ rhs in
                   assert_eq!(lhs, rhs)
               })
           })
           // modify all arrays
           arrays.indices.forEach({
               arrays[$0].append(Foo(id: "id2", value: "\($0)"))
           })
           arrays.enumerated().forEach({ lhsIndex, lhs in
               arrays.enumerated().forEach({ rhsIndex, rhs in
                   guard rhsIndex != lhsIndex else { return }
                   XCTAssertNotEqual(lhs, rhs)
               })
           })
       }
    */
}
