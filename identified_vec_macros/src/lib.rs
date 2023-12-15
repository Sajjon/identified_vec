#[macro_export]
macro_rules! newtype_identified_vec {
    (of: $item_ty: ty, named: $struct_name: ident) => {
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

        impl $struct_name {
            #[inline]
            pub fn items(&self) -> Vec<$item_ty> {
                self.0.items()
            }
        }
    };
}
