# `identified_vec`

[![Code Coverage](https://codecov.io/github/Sajjon/identified_vec/graph/badge.svg?token=Em6TayrP8j)](https://codecov.io/github/Sajjon/identified_vec)
[![Crates.io](https://img.shields.io/crates/v/identified_vec.svg)](https://crates.io/crates/identified_vec)
[![Documentation](https://docs.rs/identified_vec/badge.svg)](https://docs.rs/identified_vec)
[![Rust](https://img.shields.io/badge/rust-1.73.0%2B-blue.svg?maxAge=3600)](https://github.com/Sajjon/identified_vec)

A collection of unique identifiable elements which retains **insertion** order, inspired by [Pointfree's Swift Identified Collections](https://github.com/pointfreeco/swift-identified-collections).

Similar to the standard `Vec`, the `IdentifiedVec` maintain their elements in a particular user-specified order. However, unlike `Vec`, the `IdentifiedVec` introduce the ability to uniquely identify elements, using a hash table to ensure that no two elements have the same identity, and to efficiently look up elements corresponding to specific identifiers.

`IdentifiedVec` is a useful alternative to `Vec` when you need to be able to efficiently access unique elements by a stable identifier. It is also a useful alternative to `BTreeSet`, where the `Ord` trait requirement may be too strict, an a useful alternative to `HashSet` where `Hash` trait requirement may be too strict.

You can create an identified vec with any element type that implements the `Identifiable` trait.

# Example

```rust
extern crate identified_vec;
use identified_vec::{IsIdentifiedVec, IdentifiedVec, Identifiable, IdentifiedVecOf};
use std::cell::RefCell;

#[derive(Eq, PartialEq, Clone, Debug)]
struct User {
    id: &'static str,
    name: RefCell<&'static str>,
}

impl User {
    fn new(id: &'static str, name: &'static str) -> Self {
        Self {
            id,
            name: RefCell::new(name),
        }
    }
    fn name(&self) -> &'static str {
        *self.name.borrow()
    }
}
```

## Identifiable

```rust
impl Identifiable for User {
    type ID = &'static str;
    fn id(&self) -> Self::ID {
        self.id
    }
}
```

## `from_iter`

```rust
let mut users = IdentifiedVecOf::<User>::from_iter([
    User::new("u_42", "Satoshi Nakamoto"),
    User::new("u_1337", "Leia Skywalker"),
]);

assert_eq!(
    users.get(&"u_42").map(|u| u.name()),
    Some("Satoshi Nakamoto")
);

assert_eq!(
    users.get_at_index(1).map(|u| u.name()),
    Some("Leia Skywalker")
);
```

## `append` & `elements()`

```rust
users.append(User::new("u_237", "Alan Turing"));
assert_eq!(
    users.elements(),
    [
        User::new("u_42", "Satoshi Nakamoto"),
        User::new("u_1337", "Leia Skywalker"),
        User::new("u_237", "Alan Turing"),
    ]
    .iter()
    .collect::<Vec<&User>>()
);

// Element with same ID is not appended:
users.append(User::new("u_42", "Tom Mervolo Dolder"));
assert_eq!(
    users.elements(),
    [
        User::new("u_42", "Satoshi Nakamoto"),
        User::new("u_1337", "Leia Skywalker"),
        User::new("u_237", "Alan Turing"),
    ]
    .iter()
    .collect::<Vec<&User>>()
);
```

## `update_or_insert`

```rust
// Element with same ID replaces existing if an `update_*` method is used:
// e.g. `update_or_insert`:
users.update_or_insert(User::new("u_42", "Tom Mervolo Dolder"), 0);
assert_eq!(
    users.elements(),
    [
        User::new("u_42", "Tom Mervolo Dolder"),
        User::new("u_1337", "Leia Skywalker"),
        User::new("u_237", "Alan Turing"),
    ]
    .iter()
    .collect::<Vec<&User>>()
);
```

## `update_or_append`

```rust
// or `update_or_append`
users.update_or_append(User::new("u_237", "Marie Curie"));
assert_eq!(
    users.elements(),
    [
        User::new("u_42", "Tom Mervolo Dolder"),
        User::new("u_1337", "Leia Skywalker"),
        User::new("u_237", "Marie Curie"),
    ]
    .iter()
    .collect::<Vec<&User>>()
);
```

Or you can provide a closure that describes an element's identity:

```rust
let numbers = IdentifiedVec::<u32, u32>::new_identifying_element(|e| *e);
```

# Motivation

None of the std collections `BTreeSet` and `HashSet` retain insertion order, `Vec` retains insertion order, however, it allows for duplicates. So if you want a collection of unique elements (Set-like) that does retain insertion order, `IdentifiedVec` suits your needs. Even better, the elements does not need to be to impl `Hash` nor ` Ord`.

## Flags

This crate has the following Cargo features:

- `serde`: Enables serde serialization support on `IdentifiedVecOf` type (which `Element` impl `Identifiable` trait).
- `id_prim`: Get impl of trait `Identifiable` for primitives: `i8`,.., `i128`, `u8`, ..., `u128` and `bool` (not so useful, allows for only two elements in `IdentifiedVecOf`, but who am I to discriminate.)

## Implementation Details

An identified vec consists of a `Vec` of `ID`s keeping insertion order and a `HashMap` of id-element pairs, for constant time lookup of element given an ID.

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
