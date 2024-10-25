use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
};

use crate::account_info::NextEnumeratedAccountOptions;

use super::{DataAccount, ProcessNextEnumeratedAccount};

/// Wrapper for [DataAccount] that deserializes data with [Pack]. This type warehouses the
/// deserialized data implementing [IsInitialized] and [Pack].
#[derive(Debug)]
pub struct PackDataAccount<'a, 'b, const WRITE: bool, T: IsInitialized + Pack> {
    pub account: DataAccount<'a, 'b, WRITE>,
    pub data: T,
}

impl<'a, 'b, const WRITE: bool, T: IsInitialized + Pack> TryFrom<DataAccount<'a, 'b, WRITE>>
    for PackDataAccount<'a, 'b, WRITE, T>
{
    type Error = ProgramError;

    fn try_from(account: DataAccount<'a, 'b, WRITE>) -> Result<Self, Self::Error> {
        let data = account.try_read_pack_data()?;

        Ok(Self { account, data })
    }
}

impl<'a, 'b, const WRITE: bool, T: IsInitialized + Pack> ProcessNextEnumeratedAccount<'a, 'b>
    for PackDataAccount<'a, 'b, WRITE, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        DataAccount::<'a, 'b, WRITE>::NEXT_ACCOUNT_OPTIONS;

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        let account = DataAccount::checked_new(account)?;

        Self::try_from(account).ok()
    }
}
