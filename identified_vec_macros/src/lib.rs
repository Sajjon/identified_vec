#[macro_export]
macro_rules! newtype_identified_vec {
    (of: $item_ty: ty, named: $struct_name: ident) => {
        use identified_vec::IdentifiedVecIntoIterator;

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

        impl IntoIterator for $struct_name {
            type Item = $item_ty;
            type IntoIter = IdentifiedVecIntoIterator<<$item_ty as Identifiable>::ID, $item_ty>;

            fn into_iter(self) -> Self::IntoIter {
                Self::IntoIter::new(self.0)
            }
        }

        // impl<I, E> $struct_name<I, E>
        // where
        //     I: Eq + Hash + Clone + Debug,
        // {
        //     pub fn iter(&self) -> IdentifiedVecIterator<I, E> {
        //         IdentifiedVecIterator::new(self)
        //     }
        // }

        // impl IntoIterator for $struct_name {
        //     type Item = $item_ty;
        //     type IntoIter = IdentifiedVecIntoIterator<<$item_ty as Identifiable>::ID, $item_ty>;

        //     fn into_iter(self) -> Self::IntoIter {
        //         Self::IntoIter::new(self)
        //     }
        // }
    };
}
