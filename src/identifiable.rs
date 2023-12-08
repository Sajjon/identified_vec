use std::fmt::Debug;
use std::hash::Hash;

pub trait Identifiable: Debug + Clone {
    type ID: Eq + Hash + Clone + Debug;
    fn id(&self) -> Self::ID;
}
