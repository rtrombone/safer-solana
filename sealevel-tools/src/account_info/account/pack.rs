use crate::account::PackAccountSchema;

use super::DataAccount;

pub type PackAccount<'a, const WRITE: bool, T> = DataAccount<'a, WRITE, 0, PackAccountSchema<T>>;
pub type PackReadonlyAccount<'a, T> = PackAccount<'a, false, T>;
pub type PackWritableAccount<'a, T> = PackAccount<'a, true, T>;
