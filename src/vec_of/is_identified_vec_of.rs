use crate::{ConflictResolutionChoice, Identifiable, IsIdentifiedVec};

pub trait IsIdentifiedVecOf<Element: Identifiable>: IsIdentifiedVec<Element, Element::ID> {
    /// Constructs a new, empty `IdentifiedVec<ID, Element>`, using `id()` on `Element`
    /// as id function.
    fn new() -> Self;

    fn from_iter<It>(unique_elements: It) -> Self
    where
        It: IntoIterator<Item = Element>;

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
    ///   - combine: Closure trying to combine elements `(index, first, last)` with duplicate ids, returning which element to use, by use of ConflictResolutionChoice (`ChooseFirst` or `ChooseLast`), or `Err` if you prefer.
    /// - Returns: A new `identified_vec` initialized with the unique elements of `elements`.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    fn try_from_iter_select_unique_with<E, I>(
        elements: I,
        combine: fn((usize, &Element, &Element)) -> Result<ConflictResolutionChoice, E>,
    ) -> Result<Self, E>
    where
        I: IntoIterator<Item = Element>,
    {
        Self::try_from_iter_select_unique_ids_with(elements, |e| e.id(), combine)
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
    ///   - combine: Closure used combine elements `(index, first, last)` with duplicate ids, returning which element to use, by use of ConflictResolutionChoice (`ChooseFirst` or `ChooseLast`)
    /// - Returns: A new `identified_vec` initialized with the unique elements of `elements`.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    fn from_iter_select_unique_with<I>(
        elements: I,
        combine: fn((usize, &Element, &Element)) -> ConflictResolutionChoice,
    ) -> Self
    where
        I: IntoIterator<Item = Element>,
    {
        Self::from_iter_select_unique_ids_with(elements, |e| e.id(), combine)
    }
}
