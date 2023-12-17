/// Representation of a choice in a conflict resolution
/// where two elements with the same Self::ID exists, if `ChooseFirst`,
/// is specified the first/current/existing value will be used, but
/// if `ChooseLast` is specified then the new/last will be replace
/// the first/current/existing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConflictResolutionChoice {
    ChooseFirst,
    ChooseLast,
}
