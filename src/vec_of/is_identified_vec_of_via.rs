use crate::{
    identified_vec_iterator::IdentifiedVecIterator, ConflictResolutionChoice, Error,
    IsIdentifiedVec, IsIdentifiedVecOf, ItemsCloned,
};

use super::{identifiable_trait::Identifiable, identified_vec_of::IdentifiedVecOf};

/// https://stackoverflow.com/a/66537661/1311272
pub trait ViaMarker {}

pub trait IsIdentifiableVecOfVia<Element>:
    IsIdentifiedVecOf<Element> + IntoIterator<Item = Element> + ViaMarker
where
    Element: Identifiable,
{
    fn from_identified_vec_of(identified_vec_of: IdentifiedVecOf<Element>) -> Self;
    fn via_mut(&mut self) -> &mut IdentifiedVecOf<Element>;
    fn via(&self) -> &IdentifiedVecOf<Element>;
}

impl<Element, U> IsIdentifiedVecOf<Element> for U
where
    U: ViaMarker,
    Element: Identifiable,
    U: IsIdentifiableVecOfVia<Element>,
{
    #[inline]
    fn new() -> Self {
        Self::from_identified_vec_of(IdentifiedVecOf::new())
    }

    #[inline]
    fn from_iter<It>(unique_elements: It) -> Self
    where
        It: IntoIterator<Item = Element>,
    {
        Self::from_identified_vec_of(IdentifiedVecOf::from_iter(unique_elements))
    }
}

impl<Element, U> ItemsCloned<Element> for U
where
    U: ViaMarker,
    Element: Identifiable + Clone,
    U: IsIdentifiableVecOfVia<Element>,
{
    fn items(&self) -> Vec<Element> {
        self.via().items()
    }
}

impl<Element, U> IsIdentifiedVec<Element, Element::ID> for U
where
    U: ViaMarker,
    Element: Identifiable,
    U: IsIdentifiableVecOfVia<Element>,
{
    #[inline]
    fn new_identifying_element(
        id_of_element: fn(&Element) -> <Element as Identifiable>::ID,
    ) -> Self {
        Self::from_identified_vec_of(IdentifiedVecOf::new_identifying_element(id_of_element))
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    fn ids(&self) -> Vec<<Element as Identifiable>::ID> {
        self.via().ids()
    }

    #[inline]
    fn len(&self) -> usize {
        self.via().len()
    }

    #[inline]
    fn index_of_id(&self, id: &<Element as Identifiable>::ID) -> Option<usize> {
        self.via().index_of_id(id)
    }

    #[inline]
    fn elements(&self) -> Vec<&Element> {
        self.via().elements()
    }

    #[inline]
    fn contains(&self, element: &Element) -> bool {
        self.via().contains(element)
    }

    #[inline]
    fn contains_id(&self, id: &<Element as Identifiable>::ID) -> bool {
        self.via().contains_id(id)
    }

    #[inline]
    fn get(&self, id: &<Element as Identifiable>::ID) -> Option<&Element> {
        self.via().get(id)
    }

    #[inline]
    fn get_at_index(&self, index: usize) -> Option<&Element> {
        self.via().get_at_index(index)
    }

    #[inline]
    fn append(&mut self, element: Element) -> (bool, usize) {
        self.via_mut().append(element)
    }

    #[inline]
    fn append_other<It>(&mut self, other: It)
    where
        It: IntoIterator<Item = Element>,
    {
        self.via_mut().append_other(other)
    }

    #[inline]
    fn update_or_append(&mut self, element: Element) -> Option<Element> {
        self.via_mut().update_or_append(element)
    }

    #[inline]
    fn update_at(&mut self, element: Element, index: usize) -> Element {
        self.via_mut().update_at(element, index)
    }

    #[inline]
    fn insert(&mut self, element: Element, at: usize) -> (bool, usize) {
        self.via_mut().insert(element, at)
    }

    #[inline]
    fn update_or_insert(&mut self, element: Element, index: usize) -> (Option<Element>, usize) {
        self.via_mut().update_or_insert(element, index)
    }

    #[inline]
    fn try_update(&mut self, element: Element) -> Result<Element, Error> {
        self.via_mut().try_update(element)
    }

    #[allow(unused_mut)]
    #[inline]
    fn update_with<F>(&mut self, id: &<Element as Identifiable>::ID, mut mutate: F) -> bool
    where
        F: FnMut(&mut Element),
    {
        self.via_mut().update_with(id, mutate)
    }

    #[inline]
    fn try_append_new(&mut self, element: Element) -> Result<(bool, usize), Error> {
        self.via_mut().try_append_new(element)
    }

    /////////////
    // Remove  //
    /////////////
    #[inline]
    fn remove_by_id(&mut self, id: &<Element as Identifiable>::ID) -> Option<Element> {
        self.via_mut().remove_by_id(id)
    }

    #[inline]
    fn remove(&mut self, element: &Element) -> Option<Element> {
        self.via_mut().remove(element)
    }

    #[inline]
    fn remove_at(&mut self, index: usize) -> Element {
        self.via_mut().remove_at(index)
    }

    #[inline]
    fn remove_at_offsets<It>(&mut self, offsets: It)
    where
        It: IntoIterator<Item = usize>,
    {
        self.via_mut().remove_at_offsets(offsets)
    }

    #[inline]
    fn iter(&self) -> IdentifiedVecIterator<<Element as Identifiable>::ID, Element> {
        self.via().iter()
    }
}
