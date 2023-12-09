# `identified_vec`

[![codecov](https://codecov.io/github/Sajjon/identified_vec/graph/badge.svg?token=Em6TayrP8j)](https://codecov.io/github/Sajjon/identified_vec)

A collection of unique identifiable elements which retains **insertion** order, inspired by [Pointfree's Swift Identified Collections](https://github.com/pointfreeco/swift-identified-collections).

Similar to the standard `Vec`, the `IdentifiedVec` maintain their elements in a particular user-specified order. However, unlike `Vec`, the `IdentifiedVec` introduce the ability to uniquely identify elements, using a hash table to ensure that no two elements have the same identity, and to efficiently look up elements corresponding to specific identifiers.

`IdentifiedVec` is a useful alternative to `Vec` when you need to be able to efficiently access unique elements by a stable identifier. It is also a useful alternative to `BTreeSet`, where the `Ord` trait requirement may be too strict, an a useful alternative to `HashSet` where `Hash` trait requirement may be too strict.

You can create an identified vec with any element type that implements the `Identifiable` trait.

```rust
extern crate identified_vec;
use identified_vec::identified_vec::IdentifiedVec;
use identified_vec::identifiable::Identifiable;
use identified_vec::identified_vec_of::IdentifiedVecOf;
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

impl Identifiable for User {
    type ID = &'static str;
    fn id(&self) -> Self::ID {
        self.id
    }
}

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

// or mutate with `get_mut(id)`
*users.get_mut(&"u_1337").unwrap().name.get_mut() = "Yoda";
assert_eq!(
    users.elements(),
    [
        User::new("u_42", "Tom Mervolo Dolder"),
        User::new("u_1337", "Yoda"),
        User::new("u_237", "Marie Curie"),
    ]
    .iter()
    .collect::<Vec<&User>>()
);
```

Or you can provide a closure that describes an element's identity:

```rust
use identified_vec::identified_vec::IdentifiedVec;
use identified_vec::identifiable::Identifiable;
use identified_vec::identified_vec_of::IdentifiedVecOf;

let numbers = IdentifiedVec::<u32, u32>::new_identifying_element(|e| *e);
```

# Motivation

None of the std collections `BTreeSet` and `HashSet` retain insertion order, `Vec` retains insertion order, however, it allows for duplicates. So if you want a collection of unique elements (Set-like) that does retain insertion order, `IdentifiedVec` suits your needs. Even better, the elements does not need to be to impl `Hash` nor ` Ord`.

# Features

## Serde

The `IdentifiedVecOf` type (which `Element` impl `Identifiable` trait) is `serde::Serializable` and `serde::Deserializable` as `Vec`.

```toml
identified_vec = { version = "0.1.2", features = ["serde"] }
```

## Implementation Details

An identified vec consists of a `Vec` of `ID`s keeping insertion order and a `HashMap` of id-element pairs, for contsant time lookip of element given an ID.
