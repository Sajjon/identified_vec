//! The `newtype_identified_vec` macro allows you to create
//! a newtype wrapping an `IdentifiedVecOf` of the item type
//! you pass in, but it gets super powers! It implements the
//! traits `IsIdentifiableVecOfVia`, which implements the trait
//! `IsIdentifiedVecOf`, meaning that the declared newtype,
//! gets all the methods and functions of a `IdentifiedVecOf`,
//! and if you use the `"serde"` feature, it is also
//! (de)serializable.
//!
//! You use it like so:
//! ```
//! extern crate identified_vec;
//! use identified_vec::{IsIdentifiableVecOfVia, ViaMarker, IsIdentifiedVec, IsIdentifiedVecOf, IdentifiedVec, IdentifiedVecOf, Identifiable, newtype_identified_vec};
//!
//! newtype_identified_vec!(of: u32, named: Ints);;
//!
//! let mut ints = Ints::new();
//! ints.append(5);
//! ```
//!
#[macro_export]
macro_rules! newtype_identified_vec {
    (of: $item_ty: ty, named: $struct_name: ident) => {

        paste::paste! {
            #[derive(std::fmt::Debug, Clone, Eq, PartialEq)]
            pub struct [<Proxy $struct_name>]<Element: identified_vec::Identifiable>(identified_vec::IdentifiedVecOf<Element>);
        }
        paste::paste! {
            impl<Element: identified_vec::Identifiable> identified_vec::ViaMarker for [<Proxy $struct_name>]<Element> {}
        }
        paste::paste! {
        impl<Element: identified_vec::Identifiable> identified_vec::IsIdentifiableVecOfVia<Element> for [<Proxy $struct_name>]<Element> {

            fn via_mut(&mut self) -> &mut identified_vec::IdentifiedVecOf<Element> {
                &mut self.0
            }

            fn via(&self) -> &identified_vec::IdentifiedVecOf<Element> {
                &self.0
            }

            fn from_identified_vec_of(
                identified_vec_of: identified_vec::IdentifiedVecOf<Element>,
            ) -> Self {
                Self(identified_vec_of)
            }
        }
        }
        paste::paste! {
        impl<Element: identified_vec::Identifiable + std::fmt::Debug> std::fmt::Display for [<Proxy $struct_name>]<Element> {

            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
        }

        paste::paste! {
        impl<Element: identified_vec::Identifiable> IntoIterator for [<Proxy $struct_name>]<Element> {

            type Item = Element;
            type IntoIter = identified_vec::identified_vec_into_iterator::IdentifiedVecIntoIterator<
                <Element as identified_vec::Identifiable>::ID,
                Element,
            >;

            fn into_iter(self) -> Self::IntoIter {
                Self::IntoIter::new(self.0)
            }
        }
        }

        paste::paste! {
        impl<Element> serde::Serialize for [<Proxy $struct_name>]<Element>
        where
            Element: serde::Serialize + identified_vec::Identifiable + std::fmt::Debug + Clone,
        {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
            where
                S: serde::Serializer,
            {
                identified_vec::IdentifiedVecOf::serialize(&self.0, serializer)
            }
        }
        }

        paste::paste! {
        impl<'de, Element> serde::de::Deserialize<'de> for [<Proxy $struct_name>]<Element>
        where
            Element:
                serde::de::Deserialize<'de> + identified_vec::Identifiable + std::fmt::Debug + Clone,
        {
            fn deserialize<D: serde::Deserializer<'de>>(
                deserializer: D,
            ) -> Result<Self, D::Error> {
                let id_vec_of =
                    identified_vec::IdentifiedVecOf::<Element>::deserialize(deserializer)?;
                use identified_vec::IsIdentifiableVecOfVia;
                return Ok(Self::from_identified_vec_of(id_vec_of));
            }
        }
        }

        paste::paste! {
            pub type $struct_name = [<Proxy $struct_name>]<$item_ty>;
        }
    };
}
