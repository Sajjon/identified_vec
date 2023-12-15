#[macro_export]
macro_rules! newtype_identified_vec {
    (of: $item_ty: ty, named: $struct_name: ident) => {
        use std::cell::RefMut;
        // pub struct $struct_name<'a>(RefMut<'a, IdentifiedVecOf<$item_ty>>);
        pub struct $struct_name(IdentifiedVecOf<$item_ty>);

        // impl<'a> ViaMarker for $struct_name<'a> {}
        // impl<'a> IsIdentifiableVecOf<$item_ty> for $struct_name<'a> {}
        // impl<'a> IsIdentifiableVecOfVia<$item_ty> for $struct_name<'a> {
        //     fn via_mut(&self) -> RefMut<'a, IdentifiedVecOf<$item_ty>> {
        //         // &self.0
        //         todo!()
        //     }
        //     fn from_identified_vec_of(identified_vec_of: IdentifiedVecOf<$item_ty>) -> Self {
        //         Self(RefMut::new(identified_vec_of))
        //     }
        // }
    };
}
