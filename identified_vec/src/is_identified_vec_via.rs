use std::borrow::BorrowMut;
use std::cell::RefMut;
use std::ops::Deref;

use crate::conflict_resolution_choice::ConflictResolutionChoice;
use crate::{Identifiable, IdentifiedVecOf, IsIdentifiableVec, IsIdentifiableVecOf};

/// https://stackoverflow.com/a/66537661/1311272
pub trait ViaMarker {}

pub trait IsIdentifiableVecOfVia<Element>: IsIdentifiableVecOf<Element> + ViaMarker
where
    Element: Identifiable,
{
    fn from_identified_vec_of(identified_vec_of: IdentifiedVecOf<Element>) -> Self;
    fn via_mut(&mut self) -> &mut IdentifiedVecOf<Element>;
    fn via(&self) -> &IdentifiedVecOf<Element>;
}

impl<Element, U> IsIdentifiableVecOf<Element> for U
where
    U: ViaMarker,
    Element: Identifiable,
    U: IsIdentifiableVecOfVia<Element>,
{
    fn new() -> Self {
        Self::from_identified_vec_of(IdentifiedVecOf::new())
    }

    fn from_iter<It>(unique_elements: It) -> Self
    where
        It: IntoIterator<Item = Element>,
    {
        Self::from_identified_vec_of(IdentifiedVecOf::from_iter(unique_elements))
    }
}

impl<Element, U> IsIdentifiableVec<Element, Element::ID> for U
where
    U: ViaMarker,
    Element: Identifiable,
    U: IsIdentifiableVecOfVia<Element>,
{
    fn new_identifying_element(
        id_of_element: fn(&Element) -> <Element as Identifiable>::ID,
    ) -> Self {
        Self::from_identified_vec_of(IdentifiedVecOf::new_identifying_element(id_of_element))
    }

    fn try_from_iter_select_unique_ids_with<Er, It>(
        elements: It,
        id_of_element: fn(&Element) -> <Element as Identifiable>::ID,
        combine: fn((usize, &Element, &Element)) -> Result<ConflictResolutionChoice, Er>,
    ) -> Result<Self, Er>
    where
        It: IntoIterator<Item = Element>,
    {
        IdentifiedVecOf::try_from_iter_select_unique_ids_with(elements, id_of_element, combine)
            .map(|via| Self::from_identified_vec_of(via))
    }

    fn from_iter_select_unique_ids_with<It>(
        elements: It,
        id_of_element: fn(&Element) -> <Element as Identifiable>::ID,
        combine: fn((usize, &Element, &Element)) -> ConflictResolutionChoice,
    ) -> Self
    where
        It: IntoIterator<Item = Element>,
    {
        Self::from_identified_vec_of(IdentifiedVecOf::from_iter_select_unique_ids_with(
            elements,
            id_of_element,
            combine,
        ))
    }

    fn ids(&self) -> Vec<<Element as Identifiable>::ID> {
        self.via().ids()
    }

    fn len(&self) -> usize {
        self.via().len()
    }

    fn index_of_id(&self, id: &<Element as Identifiable>::ID) -> Option<usize> {
        self.via().index_of_id(id)
    }

    fn elements(&self) -> Vec<&Element> {
        self.via().elements()
    }

    fn contains(&self, element: &Element) -> bool {
        self.via().contains(element)
    }

    fn contains_id(&self, id: &<Element as Identifiable>::ID) -> bool {
        self.via().contains_id(id)
    }

    fn get(&self, id: &<Element as Identifiable>::ID) -> Option<&Element> {
        self.via().get(id)
    }

    fn get_at_index(&self, index: usize) -> Option<&Element> {
        self.via().get_at_index(index)
    }

    fn append(&mut self, element: Element) -> (bool, usize) {
        self.via_mut().append(element)
    }

    fn append_other<It>(&mut self, other: It)
    where
        It: IntoIterator<Item = Element>,
    {
        self.via_mut().append_other(other)
    }

    fn update_or_append(&mut self, element: Element) -> Option<Element> {
        self.via_mut().update_or_append(element)
    }

    fn update_at(&mut self, element: Element, index: usize) -> Element {
        self.via_mut().update_at(element, index)
    }

    fn insert(&mut self, element: Element, at: usize) -> (bool, usize) {
        self.via_mut().insert(element, at)
    }

    fn update_or_insert(&mut self, element: Element, index: usize) -> (Option<Element>, usize) {
        self.via_mut().update_or_insert(element, index)
    }

    /////////////
    // Remove  //
    /////////////

    fn remove_by_id(&mut self, id: &<Element as Identifiable>::ID) -> Option<Element> {
        self.via_mut().remove_by_id(id)
    }

    fn remove(&mut self, element: &Element) -> Option<Element> {
        self.via_mut().remove(element)
    }

    fn remove_at(&mut self, index: usize) -> Element {
        self.via_mut().remove_at(index)
    }

    fn remove_at_offsets<It>(&mut self, offsets: It)
    where
        It: IntoIterator<Item = usize>,
    {
        self.via_mut().remove_at_offsets(offsets)
    }
}
