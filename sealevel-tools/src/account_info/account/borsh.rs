use crate::account::BorshAccountSchema;

use super::DataAccount;

pub type BorshAccount<'a, const WRITE: bool, const DISC_LEN: usize, T> =
    DataAccount<'a, WRITE, DISC_LEN, BorshAccountSchema<DISC_LEN, T>>;
pub type BorshReadonlyAccount<'a, const DISC_LEN: usize, T> = BorshAccount<'a, false, DISC_LEN, T>;
pub type BorshWritableAccount<'a, const DISC_LEN: usize, T> = BorshAccount<'a, true, DISC_LEN, T>;
