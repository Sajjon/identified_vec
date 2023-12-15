use crate::conflict_resolution_choice::ConflictResolutionChoice;
use std::fmt::Debug;
use std::hash::Hash;

pub trait IsIdentifiableVec<Element, ID>: Sized
where
    ID: Eq + Hash + Clone + Debug,
{
    /// Constructs a new, empty `IdentifiedVec<I, E>` with the specified
    /// `id_of_element` closure
    fn new_identifying_element(id_of_element: fn(&Element) -> ID) -> Self;

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
    fn try_from_iter_select_unique_ids_with<Er, It>(
        elements: It,
        id_of_element: fn(&Element) -> ID,
        combine: fn((usize, &Element, &Element)) -> Result<ConflictResolutionChoice, Er>,
    ) -> Result<Self, Er>
    where
        It: IntoIterator<Item = Element>;

    fn from_iter_select_unique_ids_with<It>(
        elements: It,
        id_of_element: fn(&Element) -> ID,
        combine: fn((usize, &Element, &Element)) -> ConflictResolutionChoice,
    ) -> Self
    where
        It: IntoIterator<Item = Element>;

    ///The ids contained in this `identified_vec`, as an `Vec<ID>` (cloned)
    fn ids(&self) -> Vec<ID>;

    /// Returns the number of elements in the `identified_vec`, also referred to as its 'length'.
    fn len(&self) -> usize;

    /// Returns the index for the given id.
    ///
    /// If an element identified by the given id is found in the `identified_vec`, this method returns an index
    /// into the array that corresponds to the element.
    ///
    /// ```
    /// extern crate identified_vec;
    /// use identified_vec::{IsIdentifiableVec, IdentifiedVec, Identifiable, IdentifiedVecOf};
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
    fn index_of_id(&self, id: &ID) -> Option<usize>;

    /// Returns a mutable reference to the element identified by `id` if any, else None.
    ///
    /// - Parameter id: The id to find in the `identified_vec`.
    /// - Returns: The mutable reference to the element identified by `id` if found in the `identified_vec`; otherwise,
    ///   `None`.
    /// - Complexity: Expected to be O(1) on average, if `ID` implements high-quality hashing.
    fn get_mut(&mut self, id: &ID) -> Option<&mut Element>;

    fn elements(&self) -> Vec<&Element>;

    /// Returns `true` if the `identified_vec` contains the `element.`
    fn contains(&self, element: &Element) -> bool;

    /// Returns `true if the `identified_vec` contains an element for the specified `id`
    fn contains_id(&self, id: &ID) -> bool;

    /// Returns a reference to the element corresponding to the `id`, if found, else `None`.
    fn get(&self, id: &ID) -> Option<&Element>;

    /// Returns a reference to the element at index if found, else `None`.
    fn get_at_index(&self, index: usize) -> Option<&Element>;

    /// Append a new member to the end of the `identified_vec`, if the `identified_vec` doesn't already contain it.
    ///
    /// - Parameter item: The element to add to the `identified_vec`.
    /// - Returns: A pair `(inserted, index)`, where `inserted` is a Boolean value indicating whether
    ///   the operation added a new element, and `index` is the index of `item` in the resulting
    ///   `identified_vec`.
    /// - Complexity: The operation is expected to perform O(1) copy, hash, and compare operations on
    ///   the `ID` type, if it implements high-quality hashing.
    fn append(&mut self, element: Element) -> (bool, usize);

    /// Append the contents of an iterator to the end of the set, excluding elements that are already
    /// members.
    ///
    /// - Parameter elements: A finite sequence of elements to append.
    /// - Complexity: The operation is expected to perform amortized O(1) copy, hash, and compare
    ///   operations on the `Element` type, if it implements high-quality hashing.
    fn append_other<It>(&mut self, other: It)
    where
        It: IntoIterator<Item = Element>;

    /// Adds the given element to the `identified_vec` unconditionally, either appending it to the `identified_vec``, or
    /// replacing an existing value if it's already present.
    ///
    /// - Parameter item: The value to append or replace.
    /// - Returns: The original element that was replaced by this operation, or `None` if the value was
    ///   appended to the end of the collection.
    /// - Complexity: The operation is expected to perform amortized O(1) copy, hash, and compare
    ///   operations on the `ID` type, if it implements high-quality hashing.
    fn update_or_append(&mut self, element: Element) -> Option<Element>;

    /// Replace the member at the given index with a new value of the same identity.
    ///
    /// - Parameter item: The new value that should replace the original element. `item` must match
    ///   the identity of the original value.
    /// - Parameter index: The index of the element to be replaced.
    /// - Returns: The original element that was replaced.
    fn update_at(&mut self, element: Element, index: usize) -> Element;

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
    fn insert(&mut self, element: Element, at: usize) -> (bool, usize);

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
    fn update_or_insert(&mut self, element: Element, index: usize) -> (Option<Element>, usize);

    /////////////
    // Remove  //
    /////////////

    /// Removes the element identified by the given id from the `identified_vec`.
    ///
    /// ```
    /// extern crate identified_vec;
    /// use identified_vec::{IsIdentifiableVec, IdentifiedVec, Identifiable, IdentifiedVecOf};
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
    fn remove_by_id(&mut self, id: &ID) -> Option<Element>;

    /// Removes the given element from the `identified_vec`.
    ///
    /// If the element is found in the `identified_vec`, this method returns the element.
    ///
    /// If the element isn't found in the `identified_vec`, `remove` returns `None`.
    ///
    /// - Parameter element: The element to remove.
    /// - Returns: The value that was removed, or `None` if the element was not present in the `identified_vec`.
    /// - Complexity: O(`count`)
    fn remove(&mut self, element: &Element) -> Option<Element>;

    /// Removes and returns the element at the specified position.
    ///
    /// All the elements following the specified position are moved to close the resulting gap.
    ///
    /// - Parameter index: The position of the element to remove.
    /// - Returns: The removed element.
    /// - Precondition: `index` must be a valid index of the collection that is not equal to the
    ///   collection's end index.
    /// - Complexity: O(`count`)
    fn remove_at(&mut self, index: usize) -> Element;

    /// Removes all the elements at the specified `offsets` from the `identified_vec`.
    ///
    /// - Parameter offsets: The offsets of all elements to be removed.
    /// - Complexity: O(*n*) where *n* is the length of the `identified_vec`.
    fn remove_at_offsets<It>(&mut self, offsets: It)
    where
        It: IntoIterator<Item = usize>;
}
