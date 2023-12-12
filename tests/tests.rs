#![cfg(feature = "id_prim")]

use std::{cell::RefCell, collections::HashSet, fmt::Debug, ops::Deref};

use identified_vec::{
    ConflictResolutionChoice, Error, Identifiable, IdentifiedVec, IdentifiedVecOf,
    IdentifiedVecOfSerdeFailure,
};

#[derive(Eq, PartialEq, Clone)]
pub struct User {
    pub id: u16,
    pub name: RefCell<String>,
}

impl User {
    fn new(id: u16, name: &str) -> Self {
        if name.is_empty() {
            panic!("name cannot be empty")
        }
        Self {
            id,
            name: RefCell::new(name.to_string()),
        }
    }

    pub fn blob() -> Self {
        User::new(1, "Blob")
    }
    pub fn blob_jr() -> Self {
        User::new(2, "Blob, Jr.")
    }
    pub fn blob_sr() -> Self {
        User::new(3, "Blob, Sr.")
    }
}

impl Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("name", &self.name.borrow())
            .finish()
    }
}

impl Identifiable for User {
    type ID = u16;
    fn id(&self) -> Self::ID {
        self.id
    }
}

type SUT = IdentifiedVecOf<u32>;
type Users = IdentifiedVecOf<User>;

#[test]
fn new_is_empty() {
    assert_eq!(SUT::new().len(), 0);
}

#[test]
fn ids() {
    let identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(identified_vec.ids(), &[1, 2, 3])
}

#[test]
fn debug_str() {
    let identified_vec = SUT::from_iter([1, 2, 3]);
    assert!(identified_vec
        .debug_str()
        .starts_with("order: [1, 2, 3]\nelements: {"),)
}

#[test]
fn elements() {
    let vec = vec![User::blob(), User::blob_jr(), User::blob_sr()];
    let identified_vec = Users::from_iter(vec.clone());
    assert_eq!(
        identified_vec.elements(),
        vec![&User::blob(), &User::blob_jr(), &User::blob_sr()]
    );
}

#[test]
fn into_iter() {
    let vec = vec![User::blob(), User::blob_jr(), User::blob_sr()];
    let identified_vec = Users::from_iter(vec.clone());
    for (idx, element) in identified_vec.into_iter().enumerate() {
        assert_eq!(vec[idx], element)
    }
}

#[test]
fn iter() {
    let vec = vec![User::blob(), User::blob_jr(), User::blob_sr()];
    let identified_vec = Users::from_iter(vec.clone());
    for (idx, element) in identified_vec.iter().enumerate() {
        assert_eq!(&vec[idx], element)
    }
}

#[test]
fn get() {
    let vec = vec![User::blob(), User::blob_jr(), User::blob_sr()];
    let mut identified_vec = Users::from_iter(vec.clone());
    assert_eq!(identified_vec.get(&1), Some(&User::blob()));
    assert_eq!(identified_vec.get(&2), Some(&User::blob_jr()));
    assert_eq!(identified_vec.get(&3), Some(&User::blob_sr()));

    // 1
    let mut id: &u16 = &1;
    identified_vec
        .get_mut(id)
        .unwrap()
        .name
        .borrow_mut()
        .push_str(", Esq.");
    assert_eq!(
        identified_vec.get(id),
        Some(&User::new(id.clone(), "Blob, Esq."))
    );

    // 2
    id = &2;
    identified_vec
        .get_mut(id)
        .unwrap()
        .name
        .borrow_mut()
        .drain(4..9);
    assert_eq!(identified_vec.get(id), Some(&User::new(id.clone(), "Blob")));

    // 3
    id = &3;
    identified_vec
        .get_mut(id)
        .unwrap()
        .name
        .borrow_mut()
        .drain(4..9);
    assert_eq!(identified_vec.get(id), Some(&User::new(id.clone(), "Blob")));

    identified_vec.remove_by_id(id);
    assert_eq!(identified_vec.get(id), None);
    identified_vec.append(User::new(4, "Blob, Sr."));
    assert_eq!(
        identified_vec.elements(),
        [
            User::new(1, "Blob, Esq."),
            User::new(2, "Blob"),
            User::new(4, "Blob, Sr."),
        ]
        .iter()
        .collect::<Vec<&User>>()
    );
}

#[test]
fn contains_element() {
    let identified_vec = SUT::from_iter([1, 2, 3]);
    assert!(identified_vec.contains(&2))
}

#[test]
fn remove_by_id_not_present() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    assert!(identified_vec.remove_by_id(&5).is_none());
}

#[test]
fn get_at_index() {
    let identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(identified_vec.get_at_index(2), Some(&3));
    assert_eq!(identified_vec.get_at_index(999), None);
}

#[test]
fn contains_id() {
    let identified_vec = SUT::from_iter([1, 2, 3]);
    assert!(identified_vec.contains_id(&1));
    assert_eq!(identified_vec.contains_id(&999), false);
}

#[test]
fn index_id() {
    let identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(identified_vec.index_of_id(&2), Some(1));
}

#[test]
fn remove_element() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(identified_vec.remove(&2), Some(2));
    assert_eq!(identified_vec.items(), [1, 3]);
}

#[test]
fn remove_by_id() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(identified_vec.remove_by_id(&2), Some(2));
    assert_eq!(identified_vec.items(), [1, 3]);
}

#[test]
fn constructor_from_iter_select_unique_ids_with() {
    #[derive(Eq, PartialEq, Clone, Hash, Debug)]
    struct Model {
        id: i32,
        data: &'static str,
    }
    impl Model {
        fn new(id: i32, data: &'static str) -> Self {
            Self { id, data }
        }
    }

    let conservative = IdentifiedVec::<i32, Model>::from_iter_select_unique_ids_with(
        [
            Model::new(1, "A"),
            Model::new(2, "B"),
            Model::new(1, "AAAA"),
        ],
        |e| e.id,
        |_| ConflictResolutionChoice::ChooseFirst,
    );

    assert_eq!(
        conservative.items(),
        [Model::new(1, "A"), Model::new(2, "B")]
    );

    let progressive = IdentifiedVec::<i32, Model>::from_iter_select_unique_ids_with(
        [
            Model::new(1, "A"),
            Model::new(2, "B"),
            Model::new(1, "AAAA"),
        ],
        |e| e.id,
        |_| ConflictResolutionChoice::ChooseLast,
    );

    assert_eq!(
        progressive.items(),
        [Model::new(1, "AAAA"), Model::new(2, "B")]
    )
}

#[test]
fn constructor_from_iter_select_unique_with() {
    #[derive(Eq, PartialEq, Clone, Hash, Debug)]
    struct Model {
        id: i32,
        data: &'static str,
    }
    impl Model {
        fn new(id: i32, data: &'static str) -> Self {
            Self { id, data }
        }
    }
    impl Identifiable for Model {
        type ID = i32;

        fn id(&self) -> Self::ID {
            self.id
        }
    }

    let conservative = IdentifiedVecOf::<Model>::from_iter_select_unique_with(
        [
            Model::new(1, "A"),
            Model::new(2, "B"),
            Model::new(1, "AAAA"),
        ],
        |_| ConflictResolutionChoice::ChooseFirst,
    );

    assert_eq!(
        conservative.items(),
        [Model::new(1, "A"), Model::new(2, "B")]
    );

    assert_eq!(
        conservative.items(),
        [Model::new(1, "A"), Model::new(2, "B")]
    );

    let progressive = IdentifiedVecOf::<Model>::from_iter_select_unique_with(
        [
            Model::new(1, "A"),
            Model::new(2, "B"),
            Model::new(1, "AAAA"),
        ],
        |_| ConflictResolutionChoice::ChooseLast,
    );

    assert_eq!(
        progressive.items(),
        [Model::new(1, "AAAA"), Model::new(2, "B")]
    )
}

#[test]
fn append() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    let (mut inserted, mut index) = identified_vec.append(4);
    assert!(inserted);
    assert_eq!(index, 3);
    assert_eq!(identified_vec.items(), [1, 2, 3, 4]);
    (inserted, index) = identified_vec.append(2);
    assert_eq!(inserted, false);
    assert_eq!(index, 1);
    assert_eq!(identified_vec.items(), [1, 2, 3, 4]);
}

#[test]
fn try_append_unique_element() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    let result = identified_vec.try_append_unique_element(4);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().1, 3);
    assert_eq!(identified_vec.items(), [1, 2, 3, 4]);

    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    let result = identified_vec.try_append_unique_element(2);
    assert!(result.is_err());
    assert_eq!(result, Err(Error::ElementWithSameValueFound));
    assert_eq!(identified_vec.items(), [1, 2, 3]);
}

#[test]
fn try_append() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    let result = identified_vec.try_append(4);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().1, 3);
    assert_eq!(identified_vec.items(), [1, 2, 3, 4]);

    let mut identified_vec: Users = IdentifiedVecOf::new();
    identified_vec.append(User::blob());
    identified_vec.append(User::blob_jr());
    identified_vec.append(User::blob_sr());
    let result = identified_vec.try_append(User::new(2, "Blob Jr Jr"));
    assert!(result.is_err());
    assert_eq!(result, Err(Error::ElementWithSameIDFound));
    assert_eq!(
        identified_vec.items(),
        [User::blob(), User::blob_jr(), User::blob_sr()]
    );

    let mut identified_vec: Users = IdentifiedVecOf::new();
    identified_vec.append(User::blob());
    identified_vec.append(User::blob_jr());
    identified_vec.append(User::blob_sr());
    let result = identified_vec.try_append(User::new(4, "Blob Jr Jr"));
    assert!(result.is_ok());
    assert_eq!(result, Ok((true, 3)));
    assert_eq!(
        identified_vec.items(),
        [
            User::blob(),
            User::blob_jr(),
            User::blob_sr(),
            User::new(4, "Blob Jr Jr")
        ]
    );
}

#[test]
fn append_other() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    identified_vec.append_other([1, 4, 3, 5]);
    assert_eq!(identified_vec.items(), [1, 2, 3, 4, 5])
}

#[test]
fn insert() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    let (mut inserted, mut index) = identified_vec.insert(0, 0);
    assert!(inserted);
    assert_eq!(index, 0);
    assert_eq!(identified_vec.items(), [0, 1, 2, 3]);
    (inserted, index) = identified_vec.insert(2, 0);
    assert_eq!(inserted, false);
    assert_eq!(index, 2);
    assert_eq!(identified_vec.items(), [0, 1, 2, 3]);
}

#[test]
fn update_at() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(identified_vec.update_at(2, 1), 2)
}

#[test]
#[should_panic(expected = "Expected element at index {index}")]
fn update_at_expect_panic_unknown_index() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    identified_vec.update_at(0, 999);
}

#[test]
#[should_panic(expected = "The replacement item must match the identity of the original")]
fn update_at_expect_panic_other_id() {
    let mut identified_vec = IdentifiedVecOf::<User>::new();
    identified_vec.append(User::new(32, "Zelda"));
    assert_eq!(
        identified_vec
            .get_at_index(0)
            .unwrap()
            .name
            .borrow()
            .deref(),
        "Zelda"
    );
    identified_vec.update_at(User::new(999, "Zelda"), 0);
}

#[test]
fn update_or_append() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(identified_vec.update_or_append(4), None);
    assert_eq!(identified_vec.items(), [1, 2, 3, 4]);
    assert_eq!(identified_vec.update_or_append(2), Some(2));
}

#[test]
fn try_update() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(
        identified_vec.try_update(4),
        Err(Error::ExpectedElementNotPresent)
    );
    assert_eq!(identified_vec.items(), [1, 2, 3]);

    let mut identified_vec: Users = IdentifiedVecOf::new();
    identified_vec.append(User::blob());
    identified_vec.append(User::blob_jr());
    identified_vec.append(User::blob_sr());
    let result = identified_vec.try_update(User::new(2, "Blob Jr Sr"));
    assert!(result.is_ok());
    assert_eq!(result, Ok(User::blob_jr()));
    assert_eq!(
        identified_vec.items(),
        [User::blob(), User::new(2, "Blob Jr Sr"), User::blob_sr()]
    );
}

#[test]
fn update_or_insert() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    let (mut original_member, mut index) = identified_vec.update_or_insert(0, 0);
    assert_eq!(original_member, None);
    assert_eq!(index, 0);
    assert_eq!(identified_vec.items(), [0, 1, 2, 3]);
    (original_member, index) = identified_vec.update_or_insert(2, 0);
    assert_eq!(original_member, Some(2));
    assert_eq!(index, 2);
    assert_eq!(identified_vec.items(), [0, 1, 2, 3])
}

#[test]
fn remove_at_offsets() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    identified_vec.remove_at_offsets([0, 2]);
    assert_eq!(identified_vec.items(), [2])
}

#[test]
#[should_panic(expected = "Precondition failure, index out of bounds")]
fn remove_at_out_of_bounds() {
    let mut identified_vec = SUT::from_iter([1, 2, 3]);
    identified_vec.remove_at(999);
}

#[test]
fn serde() {
    let identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(
        serde_json::to_value(identified_vec.clone())
            .and_then(|j| serde_json::from_value::<SUT>(j))
            .unwrap(),
        identified_vec
    );
    assert_eq!(
        serde_json::from_str::<SUT>("[1,2,3]").unwrap(),
        identified_vec
    );
    assert_eq!(serde_json::to_string(&identified_vec).unwrap(), "[1,2,3]");
    assert_eq!(
        serde_json::from_str::<SUT>("[1,1,1]")
            .expect_err("should fail")
            .to_string(),
        "Duplicate element at offset 1"
    );

    assert!(serde_json::from_str::<SUT>("invalid").is_err(),);
}

#[test]
fn serde_via_vec() {
    let vec = vec![1, 2, 3];
    let json_from_vec = serde_json::to_value(vec).unwrap();
    let mut identified_vec = serde_json::from_value::<SUT>(json_from_vec).unwrap();
    identified_vec.append(9);
    let json_from_identified_vec = serde_json::to_value(identified_vec).unwrap();
    let vec_from_json = serde_json::from_value::<Vec<i32>>(json_from_identified_vec).unwrap();
    assert_eq!(vec_from_json, vec![1, 2, 3, 9]);
}

#[test]
fn eq() {
    #[derive(Eq, PartialEq, Clone, Hash, Debug)]
    struct Foo {
        id: &'static str,
        value: String,
    }
    impl Foo {
        fn with(id: &'static str, value: String) -> Self {
            Self { id, value }
        }
        fn new() -> Self {
            Self::with("id", "value".to_string())
        }
    }
    impl Identifiable for Foo {
        type ID = &'static str;

        fn id(&self) -> Self::ID {
            self.id
        }
    }

    // Create `IdentifiedVec` using all of the initializers
    let mut vecs: Vec<IdentifiedVecOf<Foo>> = vec![
        IdentifiedVecOf::new(),
        IdentifiedVecOf::new_identifying_element(|e| e.id()),
        IdentifiedVecOf::from_iter_select_unique_with([], |_| ConflictResolutionChoice::ChooseLast),
        IdentifiedVecOf::from_iter_select_unique_ids_with(
            [],
            |e| e.id(),
            |_| ConflictResolutionChoice::ChooseLast,
        ),
        IdentifiedVecOf::try_from_iter_select_unique_ids_with(
            [],
            |e: &Foo| e.id(),
            |_| Result::<_, ()>::Ok(ConflictResolutionChoice::ChooseLast),
        )
        .unwrap(),
    ];

    assert_eq!(
        IdentifiedVecOf::try_from_iter_select_unique_ids_with(
            [Foo::new(), Foo::new()],
            |e: &Foo| e.id(),
            |_| Err(IdentifiedVecOfSerdeFailure::DuplicateElementsAtIndex(1)),
        ),
        Err(IdentifiedVecOfSerdeFailure::DuplicateElementsAtIndex(1))
    );

    assert_eq!(
        IdentifiedVecOf::try_from_iter_select_unique_with([Foo::new(), Foo::new()], |_| Err(
            IdentifiedVecOfSerdeFailure::DuplicateElementsAtIndex(1)
        ),),
        Err(IdentifiedVecOfSerdeFailure::DuplicateElementsAtIndex(1))
    );

    vecs.iter().for_each(|l| {
        vecs.iter().for_each(|r| {
            assert_eq!(l, r);
        })
    });

    // add an element to each identified_vec
    vecs.iter_mut().for_each(|v| _ = v.append(Foo::new()));

    vecs.iter().for_each(|l| {
        vecs.iter().for_each(|r| {
            assert_eq!(l, r);
        })
    });

    // modify all arrays
    vecs.iter_mut()
        .enumerate()
        .for_each(|(i, v)| _ = v.append(Foo::with("id2", format!("{i}"))));

    vecs.iter().enumerate().for_each(|l| {
        vecs.iter().enumerate().for_each(|r| {
            if l.0 != r.0 {
                // println!("l='{}', r='{}'", l, r);
                assert_ne!(l, r)
            }
        })
    });
}

#[test]
fn display() {
    let identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(format!("{}", identified_vec), "[1, 2, 3]");
}

#[test]
fn hash() {
    let identified_vec = SUT::from_iter([1, 2, 3]);
    assert_eq!(
        HashSet::<IdentifiedVec<u32, u32>>::from_iter([identified_vec.clone()]),
        HashSet::from_iter([identified_vec.clone(), identified_vec])
    )
}
