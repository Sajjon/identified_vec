use std::collections::HashMap;

#[cfg(feature = "serde")]
use std::fmt::Debug;

#[cfg(feature = "serde")]
use super::errors::IdentifiedVecOfSerdeFailure;
use crate::{ConflictResolutionChoice, IdentifiedVec, IsIdentifiedVec, IsIdentifiedVecOf};

#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::identifiable_trait::Identifiable;

///////////////////////
/// IdentifiedVecOf ///
///////////////////////

/// A type alias for `IdentifiedVec<Element::ID, Element>`, this is the
/// preferred and most powerful collection type of this crate, requires
/// that your `Element`s impl the `Identifiable` trait. Using this collection
/// allows you to skip passing the `id_of_element: fn(&Element) -> ID` closure
/// which you otherwise need to pass when initializing an `IdentifiedVec`. Using
/// `IdentifiedVecOf` together with feature "serde" also gives serde
/// serialization/deserialization as if it were a `Vec<Element>`, given that
/// `Element` implements serde serialization/deserialization of course.
pub type IdentifiedVecOf<Element> = IdentifiedVec<<Element as Identifiable>::ID, Element>;

impl<Element> IsIdentifiedVecOf<Element> for IdentifiedVecOf<Element>
where
    Element: Identifiable,
{
    /// Constructs a new, empty `IdentifiedVec<ID, Element>`, using `id()` on `Element`
    /// as id function.
    fn new() -> Self {
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
    fn from_iter<It>(unique_elements: It) -> Self
    where
        It: IntoIterator<Item = Element>,
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
    ///   - combine: Closure trying to combine elements `(index, first, last)` with duplicate ids, returning which element to use, by use of ConflictResolutionChoice (`ChooseFirst` or `ChooseLast`), or `Err` if you prefer.
    /// - Returns: A new `identified_vec` initialized with the unique elements of `elements`.
    /// - Complexity: Expected O(*n*) on average, where *n* is the count of elements, if `ID`
    ///   implements high-quality hashing.
    #[inline]
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
    #[inline]
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

///////////////////////
////    SERDE       ///
///////////////////////
#[cfg(feature = "serde")]
impl<Element> Serialize for IdentifiedVecOf<Element>
where
    Element: Serialize + Identifiable + Debug + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        Vec::serialize(&self.elements(), serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, Element> Deserialize<'de> for IdentifiedVecOf<Element>
where
    Element: Deserialize<'de> + Identifiable + Debug + Clone,
{
    #[cfg(not(tarpaulin_include))] // false negative
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<IdentifiedVecOf<Element>, D::Error> {
        let elements = Vec::<Element>::deserialize(deserializer)?;
        IdentifiedVecOf::<Element>::try_from_iter_select_unique_with(elements, |(idx, _, _)| {
            Err(IdentifiedVecOfSerdeFailure::DuplicateElementsAtIndex(idx))
        })
        .map_err(de::Error::custom)
    }
}
