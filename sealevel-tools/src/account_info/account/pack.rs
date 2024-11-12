use crate::account::PackAccountSchema;

use super::DataAccount;

/// Account whose data is serialized as [PackAccountSchema].
pub type PackAccount<'a, const WRITE: bool, T> = DataAccount<'a, WRITE, 0, PackAccountSchema<T>>;

/// Read-only account whose data is serialized as [PackAccountSchema].
pub type ReadonlyPackAccount<'a, T> = PackAccount<'a, false, T>;

/// Writable account whose data is serialized as [PackAccountSchema].
pub type WritablePackAccount<'a, T> = PackAccount<'a, true, T>;
