use super::ConflictResolutionChoice;
use crate::identified_vec_into_iterator::IdentifiedVecIntoIterator;
use crate::iterators::identified_vec_iterator::IdentifiedVecIterator;
use crate::Error;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use super::is_identified_vec::IsIdentifiedVec;

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
/// use identified_vec::{IdentifiedVec, Identifiable, IdentifiedVecOf, IsIdentifiedVec, IsIdentifiedVecOf};
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
/// // E with same I is not appended:
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
/// // E with same I replaces existing if an `update_*` method is used:
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
/// ```
///
/// Or you can provide a closure that describes an element's identity:
///
/// ```
/// /// extern crate identified_vec;
/// use identified_vec::{IdentifiedVec, Identifiable, IdentifiedVecOf, IsIdentifiedVec, IsIdentifiedVecOf};
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
/// `IdentifiedVec` is highly sensitive to the quality of hashing implemented by the `I`
/// type. Failing to correctly implement hashing can easily lead to unacceptable performance, with
/// the severity of the effect increasing with the size of the underlying hash table.
///
/// In particular, if a certain set of elements all produce the same hash value, then hash table
/// lookups regress to searching an element in an unsorted array, i.e., a linear operation. To
/// ensure hashed collection types exhibit their target performance, it is important to ensure that
/// such collisions cannot be induced merely by adding a particular list of members to the set.
///
/// When `I` implements `Hash` correctly, testing for membership in an ordered set is expected
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
pub struct IdentifiedVec<I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    /// The holder of the insertion order
    pub(crate) order: Vec<I>,

    /// The storage of elements.
    pub(crate) elements: HashMap<I, E>,

    /// Function which extracts the I of an E.
    pub(crate) _id_of_element: fn(&E) -> I,
}

impl<I, E> IsIdentifiedVec<E, I> for IdentifiedVec<I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    ////////////////////
    //  Constructors  //
    ////////////////////

    /// Constructs a new, empty `IdentifiedVec<I, E>` with the specified
    /// `id_of_element` closure
    #[inline]
    fn new_identifying_element(id_of_element: fn(&E) -> I) -> Self {
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
    ///   - combine: Closure trying to combine elements `(index, first, last)` with duplicate ids, returning which element to use, by use of ConflictResolutionChoice (`ChooseFirst` or `ChooseLast`), or `Err` if you prefer.
    /// - Returns: A new `identified_vec` initialized with the unique elements of `elements`.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `I`
    ///   implements high-quality hashing.
    #[cfg(not(tarpaulin_include))] // false negative
    #[inline]
    fn try_from_iter_select_unique_ids_with<Er, It>(
        elements: It,
        id_of_element: fn(&E) -> I,
        combine: fn((usize, &E, &E)) -> Result<ConflictResolutionChoice, Er>,
    ) -> Result<Self, Er>
    where
        It: IntoIterator<Item = E>,
    {
        let mut _order = Vec::<I>::new();
        let mut _elements = HashMap::<I, E>::new();

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
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `I`
    ///   implements high-quality hashing.
    #[cfg(not(tarpaulin_include))] // false negative
    #[inline]
    fn from_iter_select_unique_ids_with<It>(
        elements: It,
        id_of_element: fn(&E) -> I,
        combine: fn((usize, &E, &E)) -> ConflictResolutionChoice,
    ) -> Self
    where
        It: IntoIterator<Item = E>,
    {
        let mut _order = Vec::<I>::new();
        let mut _elements = HashMap::<I, E>::new();

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

    ////////////////////
    //  Public Get    //
    ////////////////////

    /// A read-only collection view for the ids contained in this `identified_vec`, as an `&Vec<I>`.
    ///
    /// - Complexity: O(1)
    #[inline]
    fn ids(&self) -> Vec<I> {
        self.order.clone()
    }

    /// Returns the number of elements in the `identified_vec`, also referred to as its 'length'.
    #[inline]
    fn len(&self) -> usize {
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
    /// use identified_vec::{IsIdentifiedVec, IsIdentifiedVecOf, IdentifiedVec, Identifiable, IdentifiedVecOf};
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
    /// - Complexity: Expected to be O(1) on average, if `I` implements high-quality hashing.
    #[inline]
    fn index_of_id(&self, id: &I) -> Option<usize> {
        self.order.iter().position(|i| i == id)
    }

    ////////////////////
    //  Public Get    //
    ////////////////////

    /// A read-only collection of references to the elements contained in this array, as a `Vec<&Elements>`.
    ///
    /// N.B. that this method is not constant time.
    ///
    /// If `E` implements `Clone` you can use `self.items()` which returns a `Vec<E>`, of cloned elements.
    ///
    /// - Complexity: O(n)
    #[inline]
    fn elements(&self) -> Vec<&E> {
        self.iter().collect()
    }

    /// Returns `true` if the `identified_vec` contains the `element.`
    #[inline]
    fn contains(&self, element: &E) -> bool {
        self.elements.contains_key(&self.id(&element))
    }

    /// Returns `true if the `identified_vec` contains an element for the specified `id`
    #[inline]
    fn contains_id(&self, id: &I) -> bool {
        self.elements.contains_key(id)
    }

    /// Returns a reference to the element corresponding to the `id`, if found, else `None`.
    #[inline]
    fn get(&self, id: &I) -> Option<&E> {
        self.elements.get(id)
    }

    /// Returns a reference to the element at index if found, else `None`.
    #[inline]
    fn get_at_index(&self, index: usize) -> Option<&E> {
        self.order.get(index).and_then(|id| self.get(id))
    }

    /// Append a new member to the end of the `identified_vec`, if the `identified_vec` doesn't already contain it.
    ///
    /// - Parameter item: The element to add to the `identified_vec`.
    /// - Returns: A pair `(inserted, index)`, where `inserted` is a Boolean value indicating whether
    ///   the operation added a new element, and `index` is the index of `item` in the resulting
    ///   `identified_vec`.
    /// - Complexity: The operation is expected to perform O(1) copy, hash, and compare operations on
    ///   the `I` type, if it implements high-quality hashing.
    #[inline]
    fn append(&mut self, element: E) -> (bool, usize) {
        self.insert(element, self.end_index())
    }

    /// Append the contents of an iterator to the end of the set, excluding elements that are already
    /// members.
    ///
    /// - Parameter elements: A finite sequence of elements to append.
    /// - Complexity: The operation is expected to perform amortized O(1) copy, hash, and compare
    ///   operations on the `E` type, if it implements high-quality hashing.
    #[inline]
    fn append_other<It>(&mut self, other: It)
    where
        It: IntoIterator<Item = E>,
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
    ///   operations on the `I` type, if it implements high-quality hashing.
    #[inline]
    fn update_or_append(&mut self, element: E) -> Option<E> {
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
    fn update_at(&mut self, element: E, index: usize) -> E {
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

    /// Returns `false` if no element of `id` was found, otherwise if found, this
    /// existing element gets updated by `mutate` closure and this function returns
    /// `true`.
    #[inline]
    fn update_with<F>(&mut self, id: &I, mut mutate: F) -> bool
    where
        F: FnMut(&mut E),
    {
        if !self.contains_id(id) {
            return false;
        }
        let mut existing = self.elements.remove(id).expect("Element for existing id");
        mutate(&mut existing);
        self.elements.insert(id.clone(), existing);
        true
    }

    #[allow(unused_mut)]
    #[inline]
    fn try_update_with<F, Er>(&mut self, id: &I, mut mutate: F) -> Result<bool, Er>
    where
        F: FnMut(E) -> Result<E, Er>,
    {
        if !self.contains_id(id) {
            return Ok(false);
        }
        let mut existing = self.elements.remove(id).expect("Element for existing id");
        mutate(existing).map(|updated| {
            self.elements.insert(id.clone(), updated);
            true
        })
    }

    /// Try to update the given element to the `identified_vec` if a element with the same ID is already present.
    ///
    /// - Parameter item: The value to append or replace.
    /// - Returns: A Result with either the original element that was replaced by this operation, or a Error, `Error::ExpectedElementNotPresent`, specifying that the expected element is not present within the collection.
    /// - Complexity: The operation is expected to perform amortized O(1) copy, hash, and compare
    ///   operations on the `ID` type, if it implements high-quality hashing.
    #[inline]
    fn try_update(&mut self, element: E) -> Result<E, Error> {
        let id = self.id(&element);
        if self.get(&id).is_none() {
            return Err(Error::ExpectedElementNotPresent(format!("{:#?}", id)));
        }

        Ok(self
            ._update_value(element, id)
            .expect("Failed to update value"))
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
    ///   compare operations on the `I` type, if it implements high-quality hashing. (Insertions need
    ///   to make room in the storage identified_vec to add the inserted element.)
    #[cfg(not(tarpaulin_include))] // false negative
    #[inline]
    fn insert(&mut self, element: E, at: usize) -> (bool, usize) {
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
    ///   operations on the `I` type, if it implements high-quality hashing.
    #[inline]
    fn update_or_insert(&mut self, element: E, index: usize) -> (Option<E>, usize) {
        let id = self.id(&element);
        self._update_value_inserting_at(element, id, index)
    }

    ////////////////////
    // Public Remove  //
    ////////////////////
    /// Removes the element identified by the given id from the `identified_vec`.
    ///
    /// ```
    /// extern crate identified_vec;
    /// use identified_vec::{IsIdentifiedVec, IsIdentifiedVecOf, IdentifiedVec, Identifiable, IdentifiedVecOf};
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
    fn remove_by_id(&mut self, id: &I) -> Option<E> {
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
    fn remove(&mut self, element: &E) -> Option<E> {
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
    fn remove_at(&mut self, index: usize) -> E {
        let id = self
            .order
            .get(index)
            .expect("Precondition failure, index out of bounds");
        let removed = self.elements.remove(id).expect("E of existing id");
        self.order.remove(index);
        return removed;
    }

    /// Removes all the elements at the specified `offsets` from the `identified_vec`.
    ///
    /// - Parameter offsets: The offsets of all elements to be removed.
    /// - Complexity: O(*n*) where *n* is the length of the `identified_vec`.
    #[inline]
    fn remove_at_offsets<It>(&mut self, offsets: It)
    where
        It: IntoIterator<Item = usize>,
    {
        let mut internal_offset = 0;
        offsets.into_iter().for_each(|i| {
            _ = self.remove_at(i - internal_offset);
            internal_offset += 1;
        })
    }

    /// Try append a new member to the end of the `identified_vec`, if the `identified_vec` already contains the element a Error will be returned.
    ///
    /// - Parameter item: The element to add to the `identified_vec`.
    /// - Returns: Either a Ok() with a pair `(inserted, index)`, where `inserted` is a Boolean value indicating whether
    ///   the operation added a new element, and `index` is the index of `item` in the resulting
    ///   `identified_vec`. If the given ID pre-exists within the collection the function call returns `Error::ElementWithSameIDFound`.
    /// - Complexity: The operation is expected to perform O(1) copy, hash, and compare operations on
    ///   the `ID` type, if it implements high-quality hashing.
    #[inline]
    fn try_append_new(&mut self, element: E) -> Result<(bool, usize), Error> {
        let id = self.id(&element);

        if self.contains_id(&id) {
            return Err(Error::ElementWithSameIDFound(format!("{:#?}", id)));
        }

        Ok(self.append(element))
    }

    #[inline]
    fn iter(&self) -> IdentifiedVecIterator<I, E> {
        IdentifiedVecIterator::new(self)
    }
}

pub trait ItemsCloned<Element>
where
    Element: Clone,
{
    fn items(&self) -> Vec<Element>;
}

impl<I, E> ItemsCloned<E> for IdentifiedVec<I, E>
where
    E: Clone,
    I: Eq + Hash + Clone + Debug,
{
    /// A read-only collection of clones of the elements contained in this array, as a `Vec<Elements>`.
    ///
    /// N.B. that this method is not constant time.
    ///
    /// Use `self.elements()` if you are looking for a collection of references.
    ///
    /// - Complexity: O(n)
    #[inline]
    fn items(&self) -> Vec<E> {
        self.iter().map(|e| e.clone()).collect()
    }
}

impl<I, E> IntoIterator for IdentifiedVec<I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    type Item = E;
    type IntoIter = IdentifiedVecIntoIterator<I, E>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl<I, E> IdentifiedVec<I, E> where I: Eq + Hash + Clone + Debug {}

///////////////////////
////      Eq        ///
///////////////////////
impl<I, E> PartialEq for IdentifiedVec<I, E>
where
    E: PartialEq,
    I: Eq + Hash + Clone + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.elements() == other.elements()
    }
}

impl<I, E> Eq for IdentifiedVec<I, E>
where
    E: Eq,
    I: Eq + Hash + Clone + Debug,
{
}

///////////////////////
////      Hash      ///
///////////////////////
impl<I, E> Hash for IdentifiedVec<I, E>
where
    E: Hash,
    I: Eq + Hash + Clone + Debug,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.elements().hash(state);
    }
}

///////////////////////
////      Display   ///
///////////////////////
impl<I, E> Display for IdentifiedVec<I, E>
where
    E: Debug,
    I: Eq + Hash + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.elements().fmt(f)
    }
}

///////////////////////
////    PRIVATE     ///
///////////////////////
impl<I, E> IdentifiedVec<I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    /// Next index for element appended
    #[inline]
    fn end_index(&self) -> usize {
        self.len()
    }

    /// Returns the I of an E
    #[inline]
    fn id(&self, of: &E) -> I {
        (self._id_of_element)(of)
    }

    /// Inserting I at an index, returning if it did, if not, the index of the existing.
    #[cfg(not(tarpaulin_include))] // false negative
    #[inline]
    fn _insert_id_at(&mut self, id: I, index: usize) -> (bool, usize) {
        match self.index_of_id(&id) {
            Some(existing) => (false, existing),
            None => {
                self.order.insert(index, id);
                (true, index)
            }
        }
    }

    #[inline]
    fn _update_value(&mut self, element: E, for_key: I) -> Option<E> {
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
        element: E,
        for_key: I,
        index: usize,
    ) -> (Option<E>, usize) {
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

impl<ID, Element> IdentifiedVec<ID, Element>
where
    ID: Eq + Hash + Clone + Debug,
    Element: Eq + Debug,
{
    /// Try append a new unique member to the end of the `identified_vec`, if the `identified_vec` already contains the Value or ID a Error will be returned.
    ///
    /// - Parameter item: The element to add to the `identified_vec`.
    /// - Returns: Either a Ok() with a pair `(inserted, index)`, where `inserted` is a Boolean value indicating whether
    ///   the operation added a new element, and `index` is the index of `item` in the resulting
    ///   `identified_vec`. If the given ID already exist `Error::ElementWithSameIDFound` will be returned and if the value pre-exists within the collection the function call returns `Error::ElementWithSameValueFound`.
    /// - Complexity: The operation is expected to perform O(1) copy, hash, and compare operations on
    ///   the `ID` type, if it implements high-quality hashing.
    #[inline]
    pub fn try_append_unique_element(&mut self, element: Element) -> Result<(bool, usize), Error> {
        let id = self.id(&element);

        if let Some(value) = self.get(&id) {
            if value == &element {
                return Err(Error::ElementWithSameValueFound(format!("{:?}", value)));
            } else {
                return Err(Error::ElementWithSameIDFound(format!("{:?}", id)));
            }
        }

        Ok(self.append(element))
    }
}
