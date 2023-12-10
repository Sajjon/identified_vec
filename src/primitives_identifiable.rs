#![cfg(feature = "id_prim")]

use crate::identified_vec_of::Identifiable;

macro_rules! impl_id {
    ($primitive_type:ident) => {
        impl Identifiable for $primitive_type {
            type ID = $primitive_type;
            fn id(&self) -> Self::ID {
                *self
            }
        }
    };
}

impl_id!(i8);
impl_id!(i16);
impl_id!(i32);
impl_id!(i64);
impl_id!(i128);
impl_id!(u8);
impl_id!(u16);
impl_id!(u32);
impl_id!(u64);
impl_id!(u128);
impl_id!(bool);
