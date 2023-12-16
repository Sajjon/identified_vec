#[macro_export]
macro_rules! newtype_identified_vec {
    (of: $item_ty: ty, named: $struct_name: ident) => {
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct $struct_name(IdentifiedVecOf<$item_ty>);

        impl ViaMarker for $struct_name {}
        impl IsIdentifiableVecOfVia<$item_ty> for $struct_name {
            fn via_mut(&mut self) -> &mut IdentifiedVecOf<$item_ty> {
                &mut self.0
            }

            fn via(&self) -> &IdentifiedVecOf<$item_ty> {
                &self.0
            }

            fn from_identified_vec_of(identified_vec_of: IdentifiedVecOf<$item_ty>) -> Self {
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
            type IntoIter =
                identified_vec::IdentifiedVecIntoIterator<<$item_ty as Identifiable>::ID, $item_ty>;

            fn into_iter(self) -> Self::IntoIter {
                Self::IntoIter::new(self.0)
            }
        }

        #[cfg(any(test, feature = "serde"))]
        impl Serialize for $struct_name
        where
            $item_ty: Serialize + Identifiable + Debug + Clone,
        {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
            where
                S: Serializer,
            {
                IdentifiedVecOf::serialize(&self.0, serializer)
            }
        }

        #[cfg(any(test, feature = "serde"))]
        impl<'de> Deserialize<'de> for $struct_name
        where
            $item_ty: Deserialize<'de> + Identifiable + Debug + Clone,
        {
            #[cfg(not(tarpaulin_include))] // false negative
            fn deserialize<D: Deserializer<'de>>(
                deserializer: D,
            ) -> Result<$struct_name, D::Error> {
                let id_vec_of = IdentifiedVecOf::<$item_ty>::deserialize(deserializer)?;
                return Ok(Self::from_identified_vec_of(id_vec_of));
            }
        }
    };
}
