#![cfg(feature = "id_prim")]

use crate::Identifiable;

// macro_rules! impl_identifiable {
//     ($primitive_type:ident) => {
//         impl Identifiable for $primitive_type {
//             type ID = $primitive_type;
//             fn id(&self) -> Self::ID {
//                 return self.clone();
//             }
//         }
//     };
// }

impl Identifiable for i32 {
    type ID = i32;
    fn id(&self) -> Self::ID {
        *self
    }
}
