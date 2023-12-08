use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use anyerror::AnyError;

/// A collection of unique elements preserving **insertion order**,
/// do NOT use this if you need something memory efficient, this collection
/// is not optimized for that.
#[derive(Debug, Clone)]
pub struct IdentifiedVec<ID, Element>
where
    Element: Debug + Clone,
    ID: Eq + Hash + Clone + Debug,
{
    /// The holder of the insertion order
    pub(crate) order: Vec<ID>,

    /// The storage of elements.
    pub(crate) elements: HashMap<ID, Element>,

    /// Function which extracts the ID of an Element.
    pub(crate) _id_of_element: fn(&Element) -> ID,
}

///////////////////////
////  Constructors  ///
///////////////////////
impl<ID, Element> IdentifiedVec<ID, Element>
where
    Element: Debug + Clone,
    ID: Eq + Hash + Clone + Debug,
{
    /// Constructs a new, empty `IdentifiedVec<ID, Element>` with the specified
    /// `id_of_element` closure
    pub fn new_identifying_element(id_of_element: fn(&Element) -> ID) -> Self {
        Self {
            order: Vec::new(),
            elements: HashMap::new(),
            _id_of_element: id_of_element,
        }
    }

    /// Creates a new `identified_vec` from the elements in the given sequence, using a combining closure to
    /// determine the element for any elements with duplicate identity.
    ///
    /// You use this initializer to create an `identified_vec` when you have an arbitrary sequence of elements
    /// that may not have unique ids. This initializer calls the `combine` closure with the current
    /// and new elements for any duplicate ids. Pass a closure as `combine` that returns the element
    /// to use in the resulting `identified_vec`: The closure can choose between the two elements, combine them
    /// to produce a new element, or even throw an error.
    ///
    /// - Parameters:
    ///   - elements: A sequence of elements to use for the new `identified_vec`.
    ///   - id_of_element: The function which extracts the identifier for an element,
    ///   - combine: Closure trying to combine elements `(cur, new)` with duplicate ids, returning which element to use, or `Err``
    /// - Returns: A new `identified_vec` initialized with the unique elements of `elements`.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    #[inline]
    pub fn new_from_iter_try_uniquing_ids_with<I>(
        elements: I,
        id_of_element: fn(&Element) -> ID,
        combine: fn(usize, Element, Element) -> Result<Element, AnyError>,
    ) -> Result<Self, AnyError>
    where
        I: IntoIterator<Item = Element>,
    {
        let mut _order = Vec::<ID>::new();
        let mut _elements = HashMap::<ID, Element>::new();

        for element in elements.into_iter() {
            let id = id_of_element(&element);
            match _elements.get(&id) {
                Some(existing) => match combine(_order.len(), existing.to_owned(), element) {
                    Err(e) => return Err(e),
                    Ok(selected) => {
                        _elements.entry(id.clone()).and_modify(|e| *e = selected);
                    }
                },
                None => {
                    _elements.insert(id.clone(), element);
                    _order.push(id);
                }
            };
        }

        Ok(Self {
            order: _order,
            _id_of_element: id_of_element,
            elements: _elements,
        })
    }

    /// Creates a new `identified_vec` from the elements in the given sequence, using a combining closure to
    /// determine the element for any elements with duplicate identity.
    ///
    /// You use this initializer to create an `identified_vec` when you have an arbitrary sequence of elements
    /// that may not have unique ids. This initializer calls the `combine` closure with the current
    /// and new elements for any duplicate ids. Pass a closure as `combine` that returns the element
    /// to use in the resulting `identified_vec`: The closure can choose between the two elements, combine them
    /// to produce a new element, or even throw an error.
    ///
    /// - Parameters:
    ///   - elements: A sequence of elements to use for the new `identified_vec`.
    ///   - id_of_element: The function which extracts the identifier for an element,
    ///   - combine: Closure used combine elements `(cur, new)` with duplicate ids, returning which element to use.
    /// - Returns: A new `identified_vec` initialized with the unique elements of `elements`.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    #[inline]
    pub fn new_from_iter_uniquing_ids_with<I>(
        elements: I,
        id_of_element: fn(&Element) -> ID,
        combine: fn(usize, Element, Element) -> Element,
    ) -> Self
    where
        I: IntoIterator<Item = Element>,
    {
        let mut _order = Vec::<ID>::new();
        let mut _elements = HashMap::<ID, Element>::new();

        for element in elements.into_iter() {
            let id = id_of_element(&element);
            match _elements.get(&id) {
                Some(existing) => {
                    let selected = combine(_order.len(), existing.to_owned(), element);
                    _elements.entry(id.clone()).and_modify(|e| *e = selected);
                }
                None => {
                    _elements.insert(id.clone(), element);
                    _order.push(id);
                }
            };
        }

        Self {
            order: _order,
            _id_of_element: id_of_element,
            elements: _elements,
        }
    }
}

///////////////////////
////  Public Get    ///
///////////////////////
impl<ID, Element> IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
    Element: Debug + Clone,
{
    pub fn ids(&self) -> &Vec<ID> {
        &self.order
    }

    /// Returns the number of elements in the `identified_vec`, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        if cfg!(debug_assertions) {
            assert_eq!(self.order.len(), self.elements.len());
        }
        self.order.len()
    }

    /// Returns the index for the given id.
    ///
    /// If an element identified by the given id is found in the `identified_vec`, this method returns an index
    /// into the array that corresponds to the element.
    ///
    /// ```
    /// extern crate identified_vec;
    /// use identified_vec::identified_vec::IdentifiedVec;
    /// use identified_vec::identifiable::Identifiable;
    /// use identified_vec::identified_vec_of::IdentifiedVecOf;
    ///
    /// #[derive(Eq, PartialEq, Clone, Debug, Hash)]
    /// struct User {
    ///     id: &'static str,
    /// }
    ///
    /// impl User {
    ///     fn new(id: &'static str) -> Self {
    ///         Self { id }
    ///     }
    /// }
    ///
    /// impl Identifiable for User {
    ///     type ID = &'static str;
    ///     fn id(&self) -> Self::ID {
    ///         self.id
    ///     }
    /// }
    ///
    /// let mut users =
    ///     IdentifiedVecOf::<User>::from_iter([User::new("u_42"), User::new("u_1729")]);
    ///
    /// assert_eq!(users.index_of_id(&"u_1729"), Some(1));
    /// assert_eq!(users.index_of_id(&"u_1337"), None);
    /// ```
    ///
    /// - Parameter id: The id to find in the `identified_vec`.
    /// - Returns: The index for the element identified by `id` if found in the `identified_vec`; otherwise,
    ///   `nil`.
    /// - Complexity: Expected to be O(1) on average, if `ID` implements high-quality hashing.
    #[inline]
    pub fn index_of_id(&self, id: &ID) -> Option<usize> {
        self.order.iter().position(|i| i == id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: &ID) -> Option<&mut Element> {
        self.elements.get_mut(id)
    }

    #[inline]
    pub fn elements(&self) -> Vec<Element> {
        let mut elements_ordered = Vec::<Element>::new();
        for id in &self.order {
            elements_ordered.push(self.elements.get(id).expect("element").clone());
        }
        elements_ordered
    }

    #[inline]
    pub fn contains(&self, element: &Element) -> bool {
        self.elements.contains_key(&self.id(&element))
    }

    #[inline]
    pub fn contains_id(&self, id: &ID) -> bool {
        self.elements.contains_key(id)
    }

    #[inline]
    pub fn get(&self, id: &ID) -> Option<&Element> {
        self.elements.get(id)
    }
}

///////////////////////
////  Public Insert ///
///////////////////////
impl<ID, Element> IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
    Element: Debug + Clone,
{
    /// Append a new member to the end of the `identified_vec`, if the `identified_vec` doesn't already contain it.
    ///
    /// - Parameter item: The element to add to the `identified_vec`.
    /// - Returns: A pair `(inserted, index)`, where `inserted` is a Boolean value indicating whether
    ///   the operation added a new element, and `index` is the index of `item` in the resulting
    ///   `identified_vec`.
    /// - Complexity: The operation is expected to perform O(1) copy, hash, and compare operations on
    ///   the `ID` type, if it implements high-quality hashing.
    #[inline]
    pub fn append(&mut self, element: Element) -> (bool, usize) {
        self.insert(element, self.end_index())
    }

    /// Append the contents of an iterator to the end of the set, excluding elements that are already
    /// members.
    ///
    /// - Parameter elements: A finite sequence of elements to append.
    /// - Complexity: The operation is expected to perform amortized O(1) copy, hash, and compare
    ///   operations on the `Element` type, if it implements high-quality hashing.
    #[inline]
    pub fn append_other<I>(&mut self, other: I)
    where
        I: IntoIterator<Item = Element>,
    {
        other.into_iter().for_each(|i| _ = self.append(i))
    }

    /// Adds the given element to the `identified_vec` unconditionally, either appending it to the `identified_vec``, or
    /// replacing an existing value if it's already present.
    ///
    /// - Parameter item: The value to append or replace.
    /// - Returns: The original element that was replaced by this operation, or `None` if the value was
    ///   appended to the end of the collection.
    /// - Complexity: The operation is expected to perform amortized O(1) copy, hash, and compare
    ///   operations on the `ID` type, if it implements high-quality hashing.
    #[inline]
    pub fn update_or_append(&mut self, element: Element) -> Option<Element> {
        let id = self.id(&element);
        self._update_value(element, id)
    }

    /// Replace the member at the given index with a new value of the same identity.
    ///
    /// - Parameter item: The new value that should replace the original element. `item` must match
    ///   the identity of the original value.
    /// - Parameter index: The index of the element to be replaced.
    /// - Returns: The original element that was replaced.
    /// - Complexity: Amortized O(1).
    #[inline]
    pub fn update_at(&mut self, element: Element, index: usize) -> Element {
        let old_id = self
            .order
            .get(index)
            .expect("Expected element at index {index}");
        let id = self.id(&element);
        assert_eq!(
            &id, old_id,
            "The replacement item must match the identity of the original"
        );
        return self
            ._update_value_inserting_at(element, id, index)
            .0
            .expect("Replaced old value");
    }

    /// Insert a new member to this identified_vec at the specified index, if the identified_vec doesn't already contain
    /// it.
    ///
    /// - Parameter element: The element to insert.
    /// - Returns: A pair `(inserted, index)`, where `inserted` is a Boolean value indicating whether
    ///   the operation added a new element, and `index` is the index of `element` in the resulting
    ///   identified_vec. If `inserted` is true, then the returned `index` may be different from the index
    ///   requested.
    ///
    /// - Complexity: The operation is expected to perform amortized O(`self.count`) copy, hash, and
    ///   compare operations on the `ID` type, if it implements high-quality hashing. (Insertions need
    ///   to make room in the storage identified_vec to add the inserted element.)
    #[inline]
    pub fn insert(&mut self, element: Element, at: usize) -> (bool, usize) {
        let id = self.id(&element);
        if let Some(existing) = self.index_of_id(&id) {
            return (false, existing.clone());
        }
        self._update_value_inserting_at(element, id, at);
        (true, at)
    }

    /// Adds the given element into the set unconditionally, either inserting it at the specified
    /// index, or replacing an existing value if it's already present.
    ///
    /// - Parameter item: The value to append or replace.
    /// - Parameter index: The index at which to insert the new member if `item` isn't already in the
    ///   set.
    /// - Returns: The original element that was replaced by this operation, or `None` if the value was
    ///   newly inserted into the collection.
    /// - Complexity: The operation is expected to perform amortized O(1) copy, hash, and compare
    ///   operations on the `ID` type, if it implements high-quality hashing.
    #[inline]
    pub fn update_or_insert(&mut self, element: Element, index: usize) -> (Option<Element>, usize) {
        let id = self.id(&element);
        self._update_value_inserting_at(element, id, index)
    }
}

///////////////////////
//// Public Remove  ///
///////////////////////
impl<ID, Element> IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
    Element: Debug + Clone,
{
    /// Removes the element identified by the given id from the `identified_vec`.
    ///
    /// ```
    /// extern crate identified_vec;
    /// use identified_vec::identified_vec::IdentifiedVec;
    /// use identified_vec::identifiable::Identifiable;
    /// use identified_vec::identified_vec_of::IdentifiedVecOf;
    ///
    /// #[derive(Eq, PartialEq, Clone, Debug, Hash)]
    /// struct User {
    ///     id: &'static str,
    /// }
    ///
    /// impl User {
    ///     fn new(id: &'static str) -> Self {
    ///         Self { id }
    ///     }
    /// }
    ///
    /// impl Identifiable for User {
    ///     type ID = &'static str;
    ///     fn id(&self) -> Self::ID {
    ///         self.id
    ///     }
    /// }
    ///
    /// let mut users =
    ///     IdentifiedVecOf::<User>::from_iter([User::new("u_42"), User::new("u_1729")]);
    ///
    /// assert_eq!(users.remove_by_id(&"u_1729"), Some(User::new("u_1729")));
    /// assert_eq!(users.elements(), [User::new("u_42")]);
    /// assert_eq!(users.remove_by_id(&"u_1337"), None);
    /// assert_eq!(users.len(), 1);
    /// ```
    ///
    /// - Parameter id: The id of the element to be removed from the `identified_vec`.
    /// - Returns: The element that was removed, or `None` if the element was not present in the array.
    /// - Complexity: O(`count`)
    #[inline]
    pub fn remove_by_id(&mut self, id: &ID) -> Option<Element> {
        match self.index_of_id(id) {
            Some(index) => {
                self.order.remove(index);
                return self.elements.remove(id);
            }
            None => {
                assert!(!self.elements.contains_key(id));
                return None;
            }
        }
    }

    /// Removes the given element from the `indentified_vec`.
    ///
    /// If the element is found in the `indentified_vec`, this method returns the element.
    ///
    /// If the element isn't found in the `indentified_vec`, `remove` returns `None`.
    ///
    /// - Parameter element: The element to remove.
    /// - Returns: The value that was removed, or `None` if the element was not present in the `indentified_vec`.
    /// - Complexity: O(`count`)
    #[inline]
    pub fn remove(&mut self, element: &Element) -> Option<Element> {
        self.remove_by_id(&self.id(element))
    }

    /// Removes and returns the element at the specified position.
    ///
    /// All the elements following the specified position are moved to close the resulting gap.
    ///
    /// - Parameter index: The position of the element to remove.
    /// - Returns: The removed element.
    /// - Precondition: `index` must be a valid index of the collection that is not equal to the
    ///   collection's end index.
    /// - Complexity: O(`count`)
    #[inline]
    pub fn remove_at(&mut self, index: usize) -> Element {
        let id = self
            .order
            .get(index)
            .expect("Precondition failure, index out of bounds");
        let removed = self.elements.remove(id).expect("Element of existing id");
        self.order.remove(index);
        return removed;
    }

    /// Removes all the elements at the specified `offsets` from the `identified_vec`.
    ///
    /// - Parameter offsets: The offsets of all elements to be removed.
    /// - Complexity: O(*n*) where *n* is the length of the `identified_vec`.
    pub fn remove_at_offsets<I>(&mut self, offsets: I)
    where
        I: IntoIterator<Item = usize>,
    {
        let mut internal_offset = 0;
        offsets.into_iter().for_each(|i| {
            _ = self.remove_at(i - internal_offset);
            internal_offset += 1;
        })
    }
}

///////////////////////
////      Eq        ///
///////////////////////
impl<ID, Element> PartialEq for IdentifiedVec<ID, Element>
where
    Element: PartialEq + Debug + Clone,
    ID: Eq + Hash + Clone + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.elements() == other.elements()
    }
}

impl<ID, Element> Eq for IdentifiedVec<ID, Element>
where
    Element: Eq + Debug + Clone,
    ID: Eq + Hash + Clone + Debug,
{
}

///////////////////////
////      Hash      ///
///////////////////////
impl<ID, Element> Hash for IdentifiedVec<ID, Element>
where
    Element: Hash + Debug + Clone,
    ID: Eq + Hash + Clone + Debug,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.elements().hash(state);
    }
}

///////////////////////
////      Display   ///
///////////////////////
impl<ID, Element> Display for IdentifiedVec<ID, Element>
where
    Element: Debug + Clone,
    ID: Eq + Hash + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.elements().fmt(f)
    }
}

///////////////////////
////    PRIVATE     ///
///////////////////////
impl<ID, Element> IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
    Element: Debug + Clone,
{
    /// Next index for element appended
    fn end_index(&self) -> usize {
        self.len()
    }

    /// Returns the ID of an Element
    fn id(&self, of: &Element) -> ID {
        (self._id_of_element)(of)
    }

    /// Inserting ID at an index, returning if it did, if not, the index of the existing.
    fn _insert_id_at(&mut self, id: ID, index: usize) -> (bool, usize) {
        match self.index_of_id(&id) {
            Some(existing) => (false, existing),
            None => {
                self.order.insert(index, id);
                (true, index)
            }
        }
    }

    fn _update_value(&mut self, element: Element, for_key: ID) -> Option<Element> {
        let value = element;
        let key = for_key;

        if self.contains_id(&key) {
            let old = self.elements.get(&key).cloned();
            self.elements.entry(key.clone()).and_modify(|e| *e = value);
            return old;
        }

        self.elements.insert(key.clone(), value);
        self.order.push(key);
        return None;
    }

    fn _update_value_inserting_at(
        &mut self,
        element: Element,
        for_key: ID,
        index: usize,
    ) -> (Option<Element>, usize) {
        let id = for_key;
        let value = element;

        let (inserted, offset) = self._insert_id_at(id.clone(), index);
        if inserted {
            assert_eq!(offset, index);
            self.elements.insert(id.clone(), value);
            return (None, offset);
        }
        let old = self.elements.get(&id).expect("existing element").clone();
        self.elements.entry(id.clone()).and_modify(|e| *e = value);
        return (Some(old), offset);
    }
}

///////////////////////
////    DEBUG       ///
///////////////////////
impl<ID, Element> IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
    Element: Debug + Clone,
{
    #[cfg(debug_assertions)]
    pub fn debug(&self) {
        println!("{}", self.debug_str());
    }

    #[cfg(debug_assertions)]
    pub fn debug_str(&self) -> String {
        format!("order: {:?}\nelements: {:?}", self.order, self.elements)
    }
}

#[cfg(test)]
mod tests {

    use std::{cell::RefCell, collections::HashSet, fmt::Debug};

    use crate::{
        identifiable::Identifiable, identified_vec::IdentifiedVec,
        identified_vec_of::IdentifiedVecOf,
    };

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

        identified_vec.remove_by_id(id);
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

    #[test]
    fn remove_by_id() {
        let mut identified_vec = SUT::from_iter([1, 2, 3]);
        assert_eq!(identified_vec.remove_by_id(&2), Some(2));
        assert_eq!(identified_vec.elements(), [1, 3])
    }

    #[test]
    fn constructor_id_uniquing_elements() {
        #[derive(Eq, PartialEq, Clone, Hash, Debug)]
        struct Model {
            id: i32,
            data: &'static str,
        }
        impl Model {
            fn new(id: i32, data: &'static str) -> Self {
                Self { id, data }
            }
        }

        let conservative = IdentifiedVec::<i32, Model>::new_from_iter_uniquing_ids_with(
            [
                Model::new(1, "A"),
                Model::new(2, "B"),
                Model::new(1, "AAAA"),
            ],
            |e| e.id,
            |_, cur, _| cur,
        );

        assert_eq!(
            conservative.elements(),
            [Model::new(1, "A"), Model::new(2, "B")]
        );

        let progressive = IdentifiedVec::<i32, Model>::new_from_iter_uniquing_ids_with(
            [
                Model::new(1, "A"),
                Model::new(2, "B"),
                Model::new(1, "AAAA"),
            ],
            |e| e.id,
            |_, _, new| new,
        );

        assert_eq!(
            progressive.elements(),
            [Model::new(1, "AAAA"), Model::new(2, "B")]
        )
    }

    #[test]
    fn constructor_uniquing_elements() {
        #[derive(Eq, PartialEq, Clone, Hash, Debug)]
        struct Model {
            id: i32,
            data: &'static str,
        }
        impl Model {
            fn new(id: i32, data: &'static str) -> Self {
                Self { id, data }
            }
        }
        impl Identifiable for Model {
            type ID = i32;

            fn id(&self) -> Self::ID {
                self.id
            }
        }

        let conservative = IdentifiedVecOf::<Model>::new_from_iter_uniquing_with(
            [
                Model::new(1, "A"),
                Model::new(2, "B"),
                Model::new(1, "AAAA"),
            ],
            |_, cur, _| cur,
        );

        assert_eq!(
            conservative.elements(),
            [Model::new(1, "A"), Model::new(2, "B")]
        );

        let progressive = IdentifiedVecOf::<Model>::new_from_iter_uniquing_with(
            [
                Model::new(1, "A"),
                Model::new(2, "B"),
                Model::new(1, "AAAA"),
            ],
            |_, _, new| new,
        );

        assert_eq!(
            progressive.elements(),
            [Model::new(1, "AAAA"), Model::new(2, "B")]
        )
    }

    #[test]
    fn append() {
        let mut identified_vec = SUT::from_iter([1, 2, 3]);
        let (mut inserted, mut index) = identified_vec.append(4);
        assert!(inserted);
        assert_eq!(index, 3);
        assert_eq!(identified_vec.elements(), [1, 2, 3, 4]);
        (inserted, index) = identified_vec.append(2);
        assert_eq!(inserted, false);
        assert_eq!(index, 1);
        assert_eq!(identified_vec.elements(), [1, 2, 3, 4]);
    }

    #[test]
    fn append_other() {
        let mut identified_vec = SUT::from_iter([1, 2, 3]);
        identified_vec.append_other([1, 4, 3, 5]);
        assert_eq!(identified_vec.elements(), [1, 2, 3, 4, 5])
    }

    #[test]
    fn insert() {
        let mut identified_vec = SUT::from_iter([1, 2, 3]);
        let (mut inserted, mut index) = identified_vec.insert(0, 0);
        assert!(inserted);
        assert_eq!(index, 0);
        assert_eq!(identified_vec.elements(), [0, 1, 2, 3]);
        (inserted, index) = identified_vec.insert(2, 0);
        assert_eq!(inserted, false);
        assert_eq!(index, 2);
        assert_eq!(identified_vec.elements(), [0, 1, 2, 3]);
    }

    #[test]
    fn update_at() {
        let mut identified_vec = SUT::from_iter([1, 2, 3]);
        assert_eq!(identified_vec.update_at(2, 1), 2)
    }

    #[test]
    fn update_or_append() {
        let mut identified_vec = SUT::from_iter([1, 2, 3]);
        assert_eq!(identified_vec.update_or_append(4), None);
        assert_eq!(identified_vec.elements(), [1, 2, 3, 4]);
        assert_eq!(identified_vec.update_or_append(2), Some(2));
    }

    #[test]
    fn update_or_insert() {
        let mut identified_vec = SUT::from_iter([1, 2, 3]);
        let (mut original_member, mut index) = identified_vec.update_or_insert(0, 0);
        assert_eq!(original_member, None);
        assert_eq!(index, 0);
        assert_eq!(identified_vec.elements(), [0, 1, 2, 3]);
        (original_member, index) = identified_vec.update_or_insert(2, 0);
        assert_eq!(original_member, Some(2));
        assert_eq!(index, 2);
        assert_eq!(identified_vec.elements(), [0, 1, 2, 3])
    }

    #[test]
    fn remove_at_offsets() {
        let mut identified_vec = SUT::from_iter([1, 2, 3]);
        identified_vec.remove_at_offsets([0, 2]);
        assert_eq!(identified_vec.elements(), [2])
    }

    #[test]
    fn serde() {
        let identified_vec = SUT::from_iter([1, 2, 3]);
        assert_eq!(
            serde_json::to_value(identified_vec.clone())
                .and_then(|j| serde_json::from_value::<SUT>(j))
                .unwrap(),
            identified_vec
        );
        assert_eq!(
            serde_json::from_str::<SUT>("[1,2,3]").unwrap(),
            identified_vec
        );
        assert_eq!(serde_json::to_string(&identified_vec).unwrap(), "[1,2,3]");
        assert_eq!(
            serde_json::from_str::<SUT>("[1,1,1]").expect_err("should fail").to_string(),
            "identified_vec::serde_error::IdentifiedVecOfSerdeFailure: Duplicate element at offset 1"
        );
    }

    #[test]
    fn eq() {
        #[derive(Eq, PartialEq, Clone, Hash, Debug)]
        struct Foo {
            id: &'static str,
            value: String,
        }
        impl Foo {
            fn with(id: &'static str, value: String) -> Self {
                Self { id, value }
            }
            fn new() -> Self {
                Self::with("id", "value".to_string())
            }
        }
        impl Identifiable for Foo {
            type ID = &'static str;

            fn id(&self) -> Self::ID {
                self.id
            }
        }

        // Create `IdentifiedVec` using all of the initializers
        let mut vecs: Vec<IdentifiedVecOf<Foo>> = vec![
            IdentifiedVecOf::new(),
            IdentifiedVecOf::new_identifying_element(|e| e.id()),
            // IdentifiedVecOf::new_from_iter_uniquing_with([], |_, _, last| last),
            // IdentifiedVecOf::new_from_iter_uniquing_ids_with([], |e| e.id(), |_, _, last| last),
            // IdentifiedVecOf::new_from_iter_try_uniquing_ids_with([], |e| e.id(), |_,_,last| Ok(last)),
        ];

        vecs.iter().for_each(|l| {
            vecs.iter().for_each(|r| {
                assert_eq!(l, r);
            })
        });

        // add an element to each identified_vec
        vecs.iter_mut().for_each(|v| _ = v.append(Foo::new()));

        vecs.iter().for_each(|l| {
            vecs.iter().for_each(|r| {
                assert_eq!(l, r);
            })
        });

        // modify all arrays
        vecs.iter_mut()
            .enumerate()
            .for_each(|(i, v)| _ = v.append(Foo::with("id2", format!("{i}"))));

        vecs.iter().enumerate().for_each(|l| {
            vecs.iter().enumerate().for_each(|r| {
                if l.0 != r.0 {
                    // println!("l='{}', r='{}'", l, r);
                    assert_ne!(l, r)
                }
            })
        });
    }

    #[test]
    fn display() {
        let identified_vec = SUT::from_iter([1, 2, 3]);
        assert_eq!(format!("{}", identified_vec), "[1, 2, 3]");
    }

    #[test]
    fn hash() {
        let identified_vec = SUT::from_iter([1, 2, 3]);
        assert_eq!(
            HashSet::<IdentifiedVec<i32, i32>>::from_iter([identified_vec.clone()]),
            HashSet::from_iter([identified_vec.clone(), identified_vec])
        )
    }
}
