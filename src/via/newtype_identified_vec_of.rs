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
        #[derive(std::fmt::Debug, Clone, Eq, PartialEq)]
        pub struct $struct_name(identified_vec::IdentifiedVecOf<$item_ty>);

        impl identified_vec::ViaMarker for $struct_name {}
        impl identified_vec::IsIdentifiableVecOfVia<$item_ty> for $struct_name {
            fn via_mut(&mut self) -> &mut identified_vec::IdentifiedVecOf<$item_ty> {
                &mut self.0
            }

            fn via(&self) -> &identified_vec::IdentifiedVecOf<$item_ty> {
                &self.0
            }

            fn from_identified_vec_of(
                identified_vec_of: identified_vec::IdentifiedVecOf<$item_ty>,
            ) -> Self {
                Self(identified_vec_of)
            }
        }

        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl IntoIterator for $struct_name {
            type Item = $item_ty;
            type IntoIter = identified_vec::identified_vec_into_iterator::IdentifiedVecIntoIterator<
                <$item_ty as identified_vec::Identifiable>::ID,
                $item_ty,
            >;

            fn into_iter(self) -> Self::IntoIter {
                Self::IntoIter::new(self.0)
            }
        }

        #[cfg(any(test, feature = "serde"))]
        impl serde::Serialize for $struct_name
        where
            $item_ty: serde::Serialize + identified_vec::Identifiable + std::fmt::Debug + Clone,
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

        #[cfg(any(test, feature = "serde"))]
        impl<'de> serde::Deserialize<'de> for $struct_name
        where
            $item_ty:
                serde::Deserialize<'de> + identified_vec::Identifiable + std::fmt::Debug + Clone,
        {
            #[cfg(not(tarpaulin_include))] // false negative
            fn deserialize<D: serde::Deserializer<'de>>(
                deserializer: D,
            ) -> Result<$struct_name, D::Error> {
                let id_vec_of =
                    identified_vec::IdentifiedVecOf::<$item_ty>::deserialize(deserializer)?;
                use identified_vec::IsIdentifiableVecOfVia;
                return Ok(Self::from_identified_vec_of(id_vec_of));
            }
        }
    };
}
