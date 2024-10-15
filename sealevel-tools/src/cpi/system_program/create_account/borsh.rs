use borsh::BorshSerialize;
use solana_program::program_error::ProgramError;

use crate::{
    account::{try_write_borsh_data, AccountWriter},
    account_info::DataAccount,
};

pub fn try_create_borsh_data_account<'a, 'c, const N: usize>(
    args: super::CreateAccount<'a, '_, 'c>,
    account_data: &impl BorshSerialize,
    discriminator: Option<&[u8; N]>,
) -> Result<DataAccount<'a, 'c, true>, ProgramError> {
    let to_account = super::try_create_account(args)?;

    try_write_borsh_data(
        account_data,
        &mut AccountWriter::new(&to_account),
        discriminator,
    )?;

    Ok(to_account)
}
