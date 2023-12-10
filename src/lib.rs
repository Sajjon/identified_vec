//! A collection of unique identifiable elements which retains **insertion** order, inspired by [Pointfree's Swift Identified Collections](https://github.com/pointfreeco/swift-identified-collections).
//!
//! Similar to the standard `Vec`, the `IdentifiedVec` maintain their elements in a particular user-specified order. However, unlike `Vec`, the `IdentifiedVec` introduce the ability to uniquely identify elements, using a hash table to ensure that no two elements have the same identity, and to efficiently look up elements corresponding to specific identifiers.
//!
//! `IdentifiedVec` is a useful alternative to `Vec` when you need to be able to efficiently access unique elements by a stable identifier. It is also a useful alternative to `BTreeSet`, where the `Ord` trait requirement may be too strict, an a useful alternative to `HashSet` where `Hash` trait requirement may be too strict.
//!
//! You can create an identified vec with any element type that implements the `Identifiable` trait.
//!
//! # Example
//!
//! ```
//! extern crate identified_vec;
//! use identified_vec::{IsIdentifiableVec, IdentifiedVec, IdentifiedVecOf, Identifiable};
//! use std::cell::RefCell;
//!
//! #[derive(Eq, PartialEq, Clone, Debug)]
//! struct User {
//!     id: &'static str,
//!     name: RefCell<&'static str>,
//! }
//!
//! impl User {
//!     fn new(id: &'static str, name: &'static str) -> Self {
//!         Self {
//!             id,
//!             name: RefCell::new(name),
//!         }
//!     }
//!     fn name(&self) -> &'static str {
//!         *self.name.borrow()
//!     }
//! }
//!
//! impl Identifiable for User {
//!     type ID = &'static str;
//!     fn id(&self) -> Self::ID {
//!         self.id
//!     }
//! }
//!
//! let mut users = IdentifiedVecOf::<User>::from_iter([
//!     User::new("u_42", "Satoshi Nakamoto"),
//!     User::new("u_1337", "Leia Skywalker"),
//! ]);
//!
//! assert_eq!(
//!     users.get(&"u_42").map(|u| u.name()),
//!     Some("Satoshi Nakamoto")
//! );
//!
//! assert_eq!(
//!     users.get_at_index(1).map(|u| u.name()),
//!     Some("Leia Skywalker")
//! );
//!
//! users.append(User::new("u_237", "Alan Turing"));
//! assert_eq!(
//!     users.elements(),
//!     [
//!         User::new("u_42", "Satoshi Nakamoto"),
//!         User::new("u_1337", "Leia Skywalker"),
//!         User::new("u_237", "Alan Turing"),
//!     ]
//!     .iter()
//!     .collect::<Vec<&User>>()
//! );
//!
//! // Element with same ID is not appended:
//! users.append(User::new("u_42", "Tom Mervolo Dolder"));
//! assert_eq!(
//!     users.elements(),
//!     [
//!         User::new("u_42", "Satoshi Nakamoto"),
//!         User::new("u_1337", "Leia Skywalker"),
//!         User::new("u_237", "Alan Turing"),
//!     ]
//!     .iter()
//!     .collect::<Vec<&User>>()
//! );
//!
//! // Element with same ID replaces existing if an `update_*` method is used:
//! // e.g. `update_or_insert`:
//! users.update_or_insert(User::new("u_42", "Tom Mervolo Dolder"), 0);
//! assert_eq!(
//!     users.elements(),
//!     [
//!         User::new("u_42", "Tom Mervolo Dolder"),
//!         User::new("u_1337", "Leia Skywalker"),
//!         User::new("u_237", "Alan Turing"),
//!     ]
//!     .iter()
//!     .collect::<Vec<&User>>()
//! );
//!
//! // or `update_or_append`
//! users.update_or_append(User::new("u_237", "Marie Curie"));
//! assert_eq!(
//!     users.elements(),
//!     [
//!         User::new("u_42", "Tom Mervolo Dolder"),
//!         User::new("u_1337", "Leia Skywalker"),
//!         User::new("u_237", "Marie Curie"),
//!     ]
//!     .iter()
//!     .collect::<Vec<&User>>()
//! );
//!
//! // or mutate with `get_mut(id)`
//! *users.get_mut(&"u_1337").unwrap().name.get_mut() = "Yoda";
//! assert_eq!(
//!     users.elements(),
//!     [
//!         User::new("u_42", "Tom Mervolo Dolder"),
//!         User::new("u_1337", "Yoda"),
//!         User::new("u_237", "Marie Curie"),
//!     ]
//!     .iter()
//!     .collect::<Vec<&User>>()
//! );
//! ```
//!
//! Or you can provide a closure that describes an element's identity:
//!
//! ```
//! extern crate identified_vec;
//! use identified_vec::{IsIdentifiableVec, IdentifiedVec, IdentifiedVecOf, Identifiable};
//!
//! // closure which plucks out an ID from an element.
//! let numbers = IdentifiedVec::<u32, u32>::new_identifying_element(|e| *e);
//! ```

mod identifiable_trait;
mod is_id_vec_of;
mod primitives_identifiable;
mod serde_error;
mod vec;
mod vec_of;

pub mod identified_vec {
    //! A collection of unique identifiable elements which retains **insertion** order.
    pub use crate::vec::*;
}

pub mod identified_vec_of {
    //! The `Identifiable` trait allows you to use the
    //! `IdentifiedVecOf<User> instead of the more verbose
    //! `IdentifiedVec<SomeUserID, User>` but also allows you to
    //! skip the `id_of_element: fn(&Element) -> ID` closure when
    //! initializing a new identified vec.
    pub use crate::identifiable_trait::*;
    pub use crate::vec_of::*;

    #[cfg(feature = "id_prim")]
    pub use crate::primitives_identifiable::*;

    #[cfg(feature = "serde")]
    pub use crate::serde_error::*;

    #[cfg(feature = "is_id_vec_of")]
    pub use crate::is_id_vec_of::*;
}

pub use crate::identified_vec::*;
pub use crate::identified_vec_of::*;
pub use crate::vec::IsIdentifiableVec;
