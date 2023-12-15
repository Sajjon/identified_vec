use crate::identifiable_trait::Identifiable;
use crate::is_identifiable_vec::IsIdentifiableVec;

pub trait IsIdentifiableVecOf<Element: Identifiable>:
    IsIdentifiableVec<Element, Element::ID>
{
}
