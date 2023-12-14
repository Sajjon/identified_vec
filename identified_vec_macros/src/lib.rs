#[macro_export]
macro_rules! newtype_identified_vec {
    (of: $item_ty: ty, named: $struct_name: ident) => {
        pub struct $struct_name(IdentifiedVecOf<$item_ty>);

        impl $struct_name {
            /// Constructs a new, empty `IdentifiedVec<ID, Element>`, using `id()` on `Element`
            /// as id function.
            pub fn new() -> Self {
                Self(IdentifiedVecOf::new())
            }
        }
    };
}
