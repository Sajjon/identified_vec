# `identified_vec`
An collection of unique identifiable elements which retains **insertion** order, inspired by [Pointfree's Swift Identified Collections](https://github.com/pointfreeco/swift-identified-collections).

Similar to the standard `Vec`, identified vecs maintain their elements in a particular user-specified order. However, unlike `Vec`, `IdentifiedVec` introduce the ability to uniquely identify elements, using a hash table to ensure that no two elements have the same identity, and to efficiently look up elements corresponding to specific identifiers.

`IdentifiedVec` is a useful alternative to `Vec` when you need to be able to efficiently access unique elements by a stable identifier. It is also a useful alternative to `BTreeSet`, where the `Ord` trait requirement may be too strict, an a useful alternative to `HashSet` where `Hash` trait requirement may be too strict.

You can create an identified vec with any element type that implements the `Identifiable` trait.

```rust
use identified_vec::identified_vec::IdentifiedVec;
use identified_vec::identifiable::Identifiable;
use identified_vec::identified_vec_of::IdentifiedVecOf;

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
struct User {
	id: &'static str,
}

impl User {
	fn new(id: &'static str) -> Self {
		Self { id }
	}
}

impl Identifiable for User {
	type ID = &'static str;
	fn id(&self) -> Self::ID {
		self.id
	}
}

let users = IdentifiedVecOf::<User>::from_iter([
	User::new("u_42"), 
	User::new("u_1729")
]);

assert_eq!(users.index_of_id(&"u_1729"), Some(1));
```

Or you can provide a closure that describes an element's identity:

```rust
use identified_vec::identified_vec::IdentifiedVec;
use identified_vec::identifiable::Identifiable;
use identified_vec::identified_vec_of::IdentifiedVecOf;

let numbers = IdentifiedVec::<u32, u32>::new_identifying_element(|e| *e);
```

# Motivation
None of the std collections `BTreeSet` and `HashSet` retain insertion order, `Vec` retains insertion order, however, it allows for duplicates. So if you want a collection of unique elements (Set-like) that does retain insertion order, `IdentifiedVec` suits your needs. Even better, the elements does not need to be to impl `Hash` nor ` Ord``.

## Implementation Details

An identified vec consists of a `Vec` of `ID`s keeping insertion order and a `HashMap` of id-element pairs, for contsant time lookip of element given an ID.