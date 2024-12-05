use crate::account::BorshAccountSchema;

use super::DataAccount;

/// Account whose data is serialized as [BorshAccountSchema].
pub type BorshAccount<'a, const WRITE: bool, const DISC_LEN: usize, T> =
    DataAccount<'a, WRITE, DISC_LEN, BorshAccountSchema<DISC_LEN, T>>;

/// Read-only account whose data is serialized as [BorshAccountSchema].
pub type ReadonlyBorshAccount<'a, const DISC_LEN: usize, T> = BorshAccount<'a, false, DISC_LEN, T>;

/// Writable account whose data is serialized as [BorshAccountSchema].
pub type WritableBorshAccount<'a, const DISC_LEN: usize, T> = BorshAccount<'a, true, DISC_LEN, T>;
