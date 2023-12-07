use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Identifiable: Debug + Clone {
    type ID: Eq + Hash + Clone + Debug;
    fn id(&self) -> Self::ID;
}

/// A fast collection of unique elements preserving **insertion order**,
/// do NOT use this if you need something memory efficient, this collection
/// is not optimized for that. It is only optimized for speed.
pub struct IdentifiedVec<ID, Element>
where
    Element: Debug + Clone,
    ID: Eq + Hash + Clone + Debug,
{
    /// The holder of the insertion order
    order: Vec<ID>,

    /// The storage of elements.
    elements: HashMap<ID, Element>,

    _id_of_element: fn(&Element) -> ID,
}

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
        combine: fn(Element, Element) -> Result<Element, ()>,
    ) -> Result<Self, ()>
    where
        I: IntoIterator<Item = Element>,
    {
        let mut _order = Vec::<ID>::new();
        let mut _elements = HashMap::<ID, Element>::new();

        for element in elements.into_iter() {
            let id = id_of_element(&element);
            match _elements.get(&id) {
                Some(existing) => match combine(existing.to_owned(), element) {
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
        combine: fn(Element, Element) -> Element,
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
                    let selected = combine(existing.to_owned(), element);
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

impl<Element> IdentifiedVec<Element::ID, Element>
where
    Element: Identifiable,
{
    /// Constructs a new, empty `IdentifiedVec<ID, Element>`, using `id()` on `Element`
    /// as id function.
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            elements: HashMap::new(),
            _id_of_element: |i| i.id(),
        }
    }

    /// Creates a new `IdentifiedVec` from the elements in the given sequence.
    ///
    /// You use this initializer to create an `IdentifiedVec` when you have a sequence of elements with unique
    /// ids. Passing a sequence with duplicate ids to this initializer results in a runtime error.
    ///
    /// - Parameter elements: A sequence of elements to use for the new `IdentifiedVec`. Every element in
    ///   `elements` must have a unique id.
    /// - Returns: A new `IdentifiedVec` initialized with the elements of `elements`.
    /// - Precondition: The sequence must not have duplicate ids.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    #[inline]
    pub fn from_iter<I>(unique_elements: I) -> Self
    where
        I: IntoIterator<Item = Element>,
    {
        let mut _self = Self::new();
        unique_elements
            .into_iter()
            .for_each(|e| _ = _self.append(e));
        return _self;
    }

    /// Creates a new `identified_vec` from the elements in the given sequence, using a combining closure to
    /// determine the element for any elements with duplicate ids.
    ///
    /// You use this initializer to create an `identified_vec` when you have an arbitrary sequence of elements
    /// that may not have unique ids. This initializer calls the `combine` closure with the current
    /// and new elements for any duplicate ids. Pass a closure as `combine` that returns the element
    /// to use in the resulting `identified_vec`: The closure can choose between the two elements, combine them
    /// to produce a new element, or even throw an error.
    ///
    /// - Parameters:
    ///   - elements: A sequence of elements to use for the new `identified_vec`.
    ///   - combine: Closure trying to combine elements `(cur, new)` with duplicate ids, returning which element to use, or `Err``
    /// - Returns: A new `identified_vec` initialized with the unique elements of `elements`.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    #[inline]
    pub fn new_from_iter_try_uniquing_with<I>(
        elements: I,
        combine: fn(Element, Element) -> Result<Element, ()>,
    ) -> Result<Self, ()>
    where
        I: IntoIterator<Item = Element>,
    {
        Self::new_from_iter_try_uniquing_ids_with(elements, |e| e.id(), combine)
    }

    /// Creates a new `identified_vec` from the elements in the given sequence, using a combining closure to
    /// determine the element for any elements with duplicate ids.
    ///
    /// You use this initializer to create an `identified_vec` when you have an arbitrary sequence of elements
    /// that may not have unique ids. This initializer calls the `combine` closure with the current
    /// and new elements for any duplicate ids. Pass a closure as `combine` that returns the element
    /// to use in the resulting `identified_vec`: The closure can choose between the two elements, combine them
    /// to produce a new element, or even throw an error.
    ///
    /// - Parameters:
    ///   - elements: A sequence of elements to use for the new `identified_vec`.
    ///   - combine: Closure to combine elements `(cur, new)` with duplicate ids, returning which element to use
    /// - Returns: A new `identified_vec` initialized with the unique elements of `elements`.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    #[inline]
    pub fn new_from_iter_uniquing_with<I>(
        elements: I,
        combine: fn(Element, Element) -> Element,
    ) -> Self
    where
        I: IntoIterator<Item = Element>,
    {
        Self::new_from_iter_uniquing_ids_with(elements, |e| e.id(), combine)
    }
}

pub type IdentifiedVecOf<Element> = IdentifiedVec<<Element as Identifiable>::ID, Element>;

impl<ID, Element> IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
    Element: Debug + Clone,
{
    /// Returns the number of elements in the `IdentifiedVec`, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        if cfg!(debug_assertions) {
            assert_eq!(self.order.len(), self.elements.len());
        }
        self.order.len()
    }

    pub fn ids(&self) -> &Vec<ID> {
        &self.order
    }

    #[cfg(debug_assertions)]
    pub fn debug_str(&self) -> String {
        format!("order: {:?}\nelements: {:?}", self.order, self.elements)
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self) {
        println!("{}", self.debug_str());
    }

    fn id(&self, of: &Element) -> ID {
        (self._id_of_element)(of)
    }

    #[inline]
    pub fn index_of_id(&self, id: &ID) -> Option<usize> {
        self.order.iter().position(|i| i == id)
    }

    fn _update_value(&mut self, element: Element, for_key: ID, inserting_at: usize) {
        self.order.insert(inserting_at, for_key.clone());
        self.elements.insert(for_key, element);
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
        self._update_value(element, id, at);
        (true, at)
    }

    #[inline]
    pub fn update_or_insert(&mut self, element: Element, at: usize) {
        let id = self.id(&element);
        self._update_value(element, id, at)
    }

    fn end_index(&self) -> usize {
        self.len()
    }

    #[inline]
    pub fn append(&mut self, element: Element) -> (bool, usize) {
        self.insert(element, self.end_index())
    }

    #[inline]
    pub fn update_or_append(&mut self, element: Element) {
        self.update_or_insert(element, self.end_index())
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
    pub fn get(&self, id: &ID) -> Option<&Element> {
        self.elements.get(id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: &ID) -> Option<&mut Element> {
        self.elements.get_mut(id)
    }

    /// Removes the element identified by the given id from the `identified_vec`.
    ///
    /// ```
    /// extern crate identified_vec;
    /// use identified_vec::identified_vec::{IdentifiedVec, Identifiable, IdentifiedVecOf};
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
    /// - Returns: The element that was removed, or `nil` if the element was not present in the array.
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

    /// Removes the given element from the array.
    ///
    /// If the element is found in the array, this method returns the element.
    ///
    /// If the element isn't found in the array, `remove` returns `None`.
    ///
    /// - Parameter element: The element to remove.
    /// - Returns: The value that was removed, or `None` if the element was not present in the array.
    /// - Complexity: O(`count`)
    #[inline]
    pub fn remove(&mut self, element: &Element) -> Option<Element> {
        self.remove_by_id(&self.id(element))
    }
}

#[cfg(test)]
mod tests {

    use std::{cell::RefCell, fmt::Debug};

    use crate::identified_vec::IdentifiedVec;

    use super::{Identifiable, IdentifiedVecOf};

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
            |cur, _| cur,
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
            |_, new| new,
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
            |cur, _| cur,
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
            |_, new| new,
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

    /*
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
    */
}
