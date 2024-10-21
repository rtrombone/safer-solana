use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{account_info::NextEnumeratedAccountOptions, discriminator::Discriminate};

use super::{DataAccount, ProcessNextEnumeratedAccount};

#[derive(Debug)]
pub struct BorshDataAccount<
    'a,
    'b,
    const WRITE: bool,
    const N: usize,
    T: Discriminate<N> + BorshDeserialize,
> {
    pub account: DataAccount<'a, 'b, WRITE>,
    pub data: T,
}

impl<'a, 'b, const WRITE: bool, const N: usize, T: Discriminate<N> + BorshDeserialize>
    TryFrom<DataAccount<'a, 'b, WRITE>> for BorshDataAccount<'a, 'b, WRITE, N, T>
{
    type Error = ProgramError;

    fn try_from(account: DataAccount<'a, 'b, WRITE>) -> Result<Self, Self::Error> {
        let data = account.try_read_borsh_data(Some(&T::DISCRIMINATOR))?;

        Ok(Self { account, data })
    }
}

impl<'a, 'b, const N: usize, T: Discriminate<N> + BorshDeserialize>
    BorshDataAccount<'a, 'b, true, N, T>
{
    pub fn try_write_data(&self) -> ProgramResult
    where
        T: BorshSerialize,
    {
        let Self { account, data } = self;

        crate::account::try_write_borsh_data(
            data,
            &mut crate::account::AccountWriter::new(account),
            Some(&T::DISCRIMINATOR),
        )
        .map_err(Into::into)
    }
}

impl<'a, 'b, const WRITE: bool, const N: usize, T: Discriminate<N> + BorshDeserialize>
    ProcessNextEnumeratedAccount<'a, 'b> for BorshDataAccount<'a, 'b, WRITE, N, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        DataAccount::<'a, 'b, WRITE>::NEXT_ACCOUNT_OPTIONS;

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        let account = DataAccount::checked_new(account)?;

        Self::try_from(account).ok()
    }
}
