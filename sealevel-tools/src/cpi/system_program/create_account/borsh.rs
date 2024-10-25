use borsh::BorshSerialize;
use solana_program::program_error::ProgramError;

use crate::{
    account::{try_write_borsh_data, AccountWriter},
    account_info::DataAccount,
    discriminator::Discriminate,
};

/// Create a new data account and write borsh-serialized data to it. If the account requires a
/// discriminator, it will be serialized before this data.
///
/// ### Example
///
/// ```
/// use std::ops::Deref;
///
/// use borsh::{BorshDeserialize, BorshSerialize};
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, NextEnumeratedAccountOptions, DataAccount, Program,
///         Signer,
///     },
///     cpi::system_program::{try_create_borsh_data_account, CreateAccount},
///     discriminator::{Discriminate, Discriminator},
/// };
/// use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
///
/// #[derive(Debug, BorshDeserialize, BorshSerialize)]
/// pub struct Thing {
///     pub data: u64,
/// }
///
///
/// impl Discriminate<8> for Thing {
///     const DISCRIMINATOR: [u8; 8] = Discriminator::Sha2(b"account:Thing").to_bytes();
/// }
///
/// fn process_instruction(
///      program_id: &Pubkey,
///      accounts: &[AccountInfo],
///      instruction_data: &[u8],
/// ) -> ProgramResult {
///     let mut accounts_iter = accounts.iter().enumerate();
///
///     // Next account must writable signer (A.K.A. our payer).
///     let (_, payer) =
///         try_next_enumerated_account::<Signer<true>>(&mut accounts_iter, Default::default())?;
///
///     let (new_thing_addr, new_thing_bump) =
///         Pubkey::find_program_address(&[b"thing"], program_id);
///
///     // Next account must be writable data account matching PDA address.
///     let (_, new_account) = try_next_enumerated_account::<DataAccount<true>>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&new_thing_addr),
///             ..Default::default()
///         },
///     )?;
///
///     try_create_borsh_data_account(
///         CreateAccount {
///             payer: payer.as_cpi_authority(),
///             to: new_account.as_cpi_authority(Some(&[b"thing", &[new_thing_bump]])),
///             space: 16,
///             program_id,
///             account_infos: accounts,
///         },
///         &Thing { data: 420 },
///     )?;
///
///     Ok(())
/// }
/// ```
pub fn try_create_borsh_data_account<
    'a,
    'c,
    const N: usize,
    T: Discriminate<N> + BorshSerialize,
>(
    args: super::CreateAccount<'a, '_, 'c>,
    account_data: &T,
) -> Result<DataAccount<'a, 'c, true>, ProgramError> {
    let to_account = super::try_create_account(args)?;

    try_write_borsh_data(
        account_data,
        &mut AccountWriter::new(&to_account),
        Some(&T::DISCRIMINATOR),
    )?;

    Ok(to_account)
}
