use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

/// Representation of a choice in a conflict resolution
/// where two elements with the same ID exists, if `ChooseFirst`,
/// is specified the first/current/existing value will be used, but
/// if `ChooseLast` is specified then the new/last will be replace
/// the first/current/existing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConflictResolutionChoice {
    ChooseFirst,
    ChooseLast,
}

/// An ordered collection of identifiable elements.
///
/// Similar to the standard `Vec`, identified vecs maintain their elements in a particular
/// user-specified order, and they support efficient random access traversal of their members.
/// However, unlike `Vec`, identified vecs introduce the ability to uniquely identify elements,
/// using a hash table to ensure that no two elements have the same identity, and to efficiently
/// look up elements corresponding to specific identifiers.
///
/// `IdentifiedVec` is a useful alternative to `Vec` when you need to be able to efficiently
/// access unique elements by a stable identifier. It is also a useful alternative to `BTreeSet`,
/// where the `Ord` requirement may be too strict, an a useful alternative to `HashSet` where
/// `Hash` requirement may be too strict.
///
/// You can create an identified vec with any element type that implements the `Identifiable`
/// trait.
///
/// ```
/// extern crate identified_vec;
/// use identified_vec::{IdentifiedVec, Identifiable, IdentifiedVecOf};
/// use std::cell::RefCell;
///
/// #[derive(Eq, PartialEq, Clone, Debug)]
/// struct User {
///     id: &'static str,
///     name: RefCell<&'static str>,
/// }
///
/// impl User {
///     fn new(id: &'static str, name: &'static str) -> Self {
///         Self {
///             id,
///             name: RefCell::new(name),
///         }
///     }
///     fn name(&self) -> &'static str {
///         *self.name.borrow()
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
/// let mut users = IdentifiedVecOf::<User>::from_iter([
///     User::new("u_42", "Satoshi Nakamoto"),
///     User::new("u_1337", "Leia Skywalker"),
/// ]);
///
/// assert_eq!(
///     users.get(&"u_42").map(|u| u.name()),
///     Some("Satoshi Nakamoto")
/// );
/// assert_eq!(
///     users.get_at_index(1).map(|u| u.name()),
///     Some("Leia Skywalker")
/// );
/// users.append(User::new("u_237", "Alan Turing"));
/// assert_eq!(
///     users.elements(),
///     [
///         User::new("u_42", "Satoshi Nakamoto"),
///         User::new("u_1337", "Leia Skywalker"),
///         User::new("u_237", "Alan Turing"),
///     ]
///     .iter()
///     .collect::<Vec<&User>>()
/// );
///
/// // Element with same ID is not appended:
/// users.append(User::new("u_42", "Tom Mervolo Dolder"));
/// assert_eq!(
///     users.elements(),
///     [
///         User::new("u_42", "Satoshi Nakamoto"),
///         User::new("u_1337", "Leia Skywalker"),
///         User::new("u_237", "Alan Turing"),
///     ]
///     .iter()
///     .collect::<Vec<&User>>()
/// );
///
/// // Element with same ID replaces existing if an `update_*` method is used:
/// // e.g. `update_or_insert`:
/// users.update_or_insert(User::new("u_42", "Tom Mervolo Dolder"), 0);
/// assert_eq!(
///     users.elements(),
///     [
///         User::new("u_42", "Tom Mervolo Dolder"),
///         User::new("u_1337", "Leia Skywalker"),
///         User::new("u_237", "Alan Turing"),
///     ]
///     .iter()
///     .collect::<Vec<&User>>()
/// );
///
/// // or `update_or_append`
/// users.update_or_append(User::new("u_237", "Marie Curie"));
/// assert_eq!(
///     users.elements(),
///     [
///         User::new("u_42", "Tom Mervolo Dolder"),
///         User::new("u_1337", "Leia Skywalker"),
///         User::new("u_237", "Marie Curie"),
///     ]
///     .iter()
///     .collect::<Vec<&User>>()
/// );
///
/// // or mutate with `get_mut(id)`
/// *users.get_mut(&"u_1337").unwrap().name.get_mut() = "Yoda";
/// assert_eq!(
///     users.elements(),
///     [
///         User::new("u_42", "Tom Mervolo Dolder"),
///         User::new("u_1337", "Yoda"),
///         User::new("u_237", "Marie Curie"),
///     ]
///     .iter()
///     .collect::<Vec<&User>>()
/// );
/// ```
///
/// Or you can provide a closure that describes an element's identity:
///
/// ```
/// /// extern crate identified_vec;
/// use identified_vec::{IdentifiedVec, Identifiable, IdentifiedVecOf};
///
/// let numbers = IdentifiedVec::<u32, u32>::new_identifying_element(|e| *e);
/// ```
///
/// # Motivation
/// None of the std collections `BTreeSet` and `HashSet` retain insertion order, `Vec` retains
/// insertion order, however, it allows for duplicates. So if you want a collection of unique
/// elements (Set-like) that does retain insertion order, `IdentifiedVec` suits your needs.
/// Even better, the elements does not need to be to impl `Hash` nor `Ord``.
///
/// # Performance
///
/// Like the standard `HashMap` type, the performance of hashing operations in
/// `IdentifiedVec` is highly sensitive to the quality of hashing implemented by the `ID`
/// type. Failing to correctly implement hashing can easily lead to unacceptable performance, with
/// the severity of the effect increasing with the size of the underlying hash table.
///
/// In particular, if a certain set of elements all produce the same hash value, then hash table
/// lookups regress to searching an element in an unsorted array, i.e., a linear operation. To
/// ensure hashed collection types exhibit their target performance, it is important to ensure that
/// such collisions cannot be induced merely by adding a particular list of members to the set.
///
/// When `ID` implements `Hash` correctly, testing for membership in an ordered set is expected
/// to take O(1) equality checks on average. Hash collisions can still occur organically, so the
/// worst-case lookup performance is technically still O(*n*) (where *n* is the size of the set);
/// however, long lookup chains are unlikely to occur in practice.
///
/// ## Implementation Details
///
/// An identified vec consists of a Vec and a HashMap of id-element pairs. An element's id
/// should not be mutated in place, as it will drift from its associated dictionary key. Identified
/// bec is designed to avoid this invariant. Mutating an element's id will result in a runtime error.
#[derive(Debug, Clone)]
pub struct IdentifiedVec<ID, Element>
where
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
    ID: Eq + Hash + Clone + Debug,
{
    /// Constructs a new, empty `IdentifiedVec<ID, Element>` with the specified
    /// `id_of_element` closure
    #[inline]
    pub fn new_identifying_element(id_of_element: fn(&Element) -> ID) -> Self {
        Self {
            order: Vec::new(),
            elements: HashMap::new(),
            _id_of_element: id_of_element,
        }
    }
}

///////////////////////
////  Constructors  ///
///////////////////////
impl<ID, Element> IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
{
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
    ///   - combine: Closure trying to combine elements `(index, first, last)` with duplicate ids, returning which element to use, by use of ConflictResolutionChoice (`ChooseFirst` or `ChooseLast`), or `Err` if you prefer.
    /// - Returns: A new `identified_vec` initialized with the unique elements of `elements`.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    #[cfg(not(tarpaulin_include))] // false negative
    #[inline]
    pub fn try_from_iter_select_unique_ids_with<E, I>(
        elements: I,
        id_of_element: fn(&Element) -> ID,
        combine: fn((usize, &Element, &Element)) -> Result<ConflictResolutionChoice, E>,
    ) -> Result<Self, E>
    where
        I: IntoIterator<Item = Element>,
    {
        let mut _order = Vec::<ID>::new();
        let mut _elements = HashMap::<ID, Element>::new();

        for element in elements.into_iter() {
            let id = id_of_element(&element);
            match _elements.remove(&id) {
                Some(existing) => match combine((_order.len(), &existing, &element)) {
                    Err(e) => return Err(e),
                    Ok(choice) => match choice {
                        ConflictResolutionChoice::ChooseFirst => {
                            _elements.insert(id.clone(), existing)
                        }
                        ConflictResolutionChoice::ChooseLast => {
                            _elements.insert(id.clone(), element)
                        }
                    },
                },
                None => {
                    _elements.insert(id.clone(), element);
                    _order.push(id);
                    None
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
    ///   - combine: Closure used combine elements `(index, first, last)` with duplicate ids, returning which element to use, by use of ConflictResolutionChoice (`ChooseFirst` or `ChooseLast`)
    /// - Returns: A new `identified_vec` initialized with the unique elements of `elements`.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    #[cfg(not(tarpaulin_include))] // false negative
    #[inline]
    pub fn from_iter_select_unique_ids_with<I>(
        elements: I,
        id_of_element: fn(&Element) -> ID,
        combine: fn((usize, &Element, &Element)) -> ConflictResolutionChoice,
    ) -> Self
    where
        I: IntoIterator<Item = Element>,
    {
        let mut _order = Vec::<ID>::new();
        let mut _elements = HashMap::<ID, Element>::new();

        for element in elements.into_iter() {
            let id = id_of_element(&element);
            match _elements.remove(&id) {
                Some(existing) => match combine((_order.len(), &existing, &element)) {
                    ConflictResolutionChoice::ChooseFirst => _elements.insert(id.clone(), existing),
                    ConflictResolutionChoice::ChooseLast => _elements.insert(id.clone(), element),
                },
                None => {
                    _elements.insert(id.clone(), element);
                    _order.push(id);
                    None
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
{
    /// A read-only collection view for the ids contained in this `identified_vec`, as an `&Vec<ID>`.
    ///
    /// - Complexity: O(1)
    #[inline]
    pub fn ids(&self) -> &Vec<ID> {
        &self.order
    }

    /// Returns the number of elements in the `identified_vec`, also referred to as its 'length'.
    #[inline]
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
    /// use identified_vec::{IdentifiedVec, Identifiable, IdentifiedVecOf};
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
    ///   `None`.
    /// - Complexity: Expected to be O(1) on average, if `ID` implements high-quality hashing.
    #[inline]
    pub fn index_of_id(&self, id: &ID) -> Option<usize> {
        self.order.iter().position(|i| i == id)
    }

    /// Returns a mutable reference to the element identified by `id` if any, else None.
    ///
    /// - Parameter id: The id to find in the `identified_vec`.
    /// - Returns: The mutable reference to the element identified by `id` if found in the `identified_vec`; otherwise,
    ///   `None`.
    /// - Complexity: Expected to be O(1) on average, if `ID` implements high-quality hashing.
    #[inline]
    pub fn get_mut(&mut self, id: &ID) -> Option<&mut Element> {
        self.elements.get_mut(id)
    }

    /// A read-only collection view for the elements contained in this array, as a `Vec<Elements>`.
    ///
    /// - Complexity: O(n)
    #[inline]
    pub fn elements(&self) -> Vec<&Element> {
        let mut elements_ordered = Vec::<&Element>::new();
        for id in &self.order {
            elements_ordered.push(self.elements.get(id).expect("element"));
        }
        elements_ordered
    }

    /// Returns `true` if the `identified_vec` contains the `element.`
    #[inline]
    pub fn contains(&self, element: &Element) -> bool {
        self.elements.contains_key(&self.id(&element))
    }

    /// Returns `true if the `identified_vec` contains an element for the specified `id`
    #[inline]
    pub fn contains_id(&self, id: &ID) -> bool {
        self.elements.contains_key(id)
    }

    /// Returns a reference to the element corresponding to the `id`, if found, else `None`.
    #[inline]
    pub fn get(&self, id: &ID) -> Option<&Element> {
        self.elements.get(id)
    }

    /// Returns a reference to the element at index if found, else `None`.
    #[inline]
    pub fn get_at_index(&self, index: usize) -> Option<&Element> {
        self.order.get(index).and_then(|id| self.get(id))
    }
}

/// An iterator over the items of an `IdentifiedVec`.
pub struct IdentifiedVecIterator<'a, ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
{
    identified_vec: &'a IdentifiedVec<ID, Element>,
    index: usize,
}

impl<'a, ID, Element> Iterator for IdentifiedVecIterator<'a, ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
{
    type Item = &'a Element;

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

impl<ID, Element> IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
{
    pub fn iter(&self) -> IdentifiedVecIterator<ID, Element> {
        IdentifiedVecIterator {
            identified_vec: self,
            index: 0,
        }
    }
}

/// An owning iterator over the items of an `IdentifiedVec`.
pub struct IdentifiedVecIntoIterator<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
{
    identified_vec: IdentifiedVec<ID, Element>,
}

impl<ID, Element> Iterator for IdentifiedVecIntoIterator<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
{
    type Item = Element;

    fn next(&mut self) -> Option<Self::Item> {
        if self.identified_vec.len() == 0 {
            return None;
        }
        let result = self.identified_vec.remove_at(0);
        Some(result)
    }
}

impl<ID, Element> IntoIterator for IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
{
    type Item = Element;
    type IntoIter = IdentifiedVecIntoIterator<ID, Element>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            identified_vec: self,
        }
    }
}

///////////////////////
////  Public Insert ///
///////////////////////
impl<ID, Element> IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
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
    #[cfg(not(tarpaulin_include))] // false negative
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
{
    /// Removes the element identified by the given id from the `identified_vec`.
    ///
    /// ```
    /// extern crate identified_vec;
    /// use identified_vec::{IdentifiedVec, Identifiable, IdentifiedVecOf};
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
    /// assert_eq!(users.elements(), [&User::new("u_42")]);
    /// assert_eq!(users.remove_by_id(&"u_1337"), None);
    /// assert_eq!(users.len(), 1);
    /// ```
    ///
    /// - Parameter id: The id of the element to be removed from the `identified_vec`.
    /// - Returns: The element that was removed, or `None` if the element was not present in the array.
    /// - Complexity: O(`count`)
    #[cfg(not(tarpaulin_include))] // false negative
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

    /// Removes the given element from the `identified_vec`.
    ///
    /// If the element is found in the `identified_vec`, this method returns the element.
    ///
    /// If the element isn't found in the `identified_vec`, `remove` returns `None`.
    ///
    /// - Parameter element: The element to remove.
    /// - Returns: The value that was removed, or `None` if the element was not present in the `identified_vec`.
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
    #[inline]
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
    Element: PartialEq,
    ID: Eq + Hash + Clone + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.elements() == other.elements()
    }
}

impl<ID, Element> Eq for IdentifiedVec<ID, Element>
where
    Element: Eq,
    ID: Eq + Hash + Clone + Debug,
{
}

///////////////////////
////      Hash      ///
///////////////////////
impl<ID, Element> Hash for IdentifiedVec<ID, Element>
where
    Element: Hash,
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
    Element: Debug,
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
{
    /// Next index for element appended
    #[inline]
    fn end_index(&self) -> usize {
        self.len()
    }

    /// Returns the ID of an Element
    #[inline]
    fn id(&self, of: &Element) -> ID {
        (self._id_of_element)(of)
    }

    /// Inserting ID at an index, returning if it did, if not, the index of the existing.
    #[cfg(not(tarpaulin_include))] // false negative
    #[inline]
    fn _insert_id_at(&mut self, id: ID, index: usize) -> (bool, usize) {
        match self.index_of_id(&id) {
            Some(existing) => (false, existing),
            None => {
                self.order.insert(index, id);
                (true, index)
            }
        }
    }

    #[inline]
    fn _update_value(&mut self, element: Element, for_key: ID) -> Option<Element> {
        let value = element;
        let key = for_key;

        let maybe_old = self.elements.remove(&key);
        self.elements.insert(key.clone(), value);

        if maybe_old.is_some() {
            return maybe_old;
        } else {
            self.order.push(key);
            None
        }
    }

    #[inline]
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
        let old = self.elements.remove(&id).expect("existing element");
        self.elements.insert(id, value);
        return (Some(old), offset);
    }
}

///////////////////////
////    DEBUG       ///
///////////////////////
impl<ID, Element> IdentifiedVec<ID, Element>
where
    Element: Debug,
    ID: Eq + Hash + Clone + Debug,
{
    #[cfg(not(tarpaulin_include))]
    #[cfg(debug_assertions)]
    pub fn debug(&self) {
        println!("{}", self.debug_str());
    }

    #[cfg(debug_assertions)]
    pub fn debug_str(&self) -> String {
        format!("order: {:?}\nelements: {:?}", self.order, self.elements)
    }
}
