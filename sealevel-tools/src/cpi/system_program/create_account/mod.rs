#[cfg(feature = "borsh")]
mod borsh;

#[cfg(feature = "borsh")]
pub use borsh::*;

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::Instruction,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::{allocate, assign, create_account, transfer},
    sysvar::Sysvar,
};

use crate::{account_info::DataAccount, error::SealevelToolsError};

/// When specified in [CreateAccount], either find the account by its key in the [AccountInfo]
/// slice (can be expensive) or use the provided [AccountInfo].
pub enum ToAccount<'a, 'b> {
    /// Use this key to find the [AccountInfo] as account to be created.
    Key(&'b Pubkey),

    /// Use this [AccountInfo] as the account to be created.
    Info(&'b AccountInfo<'a>),
}

impl<'a, 'b> From<&'b Pubkey> for ToAccount<'a, 'b> {
    fn from(pubkey: &'b Pubkey) -> Self {
        ToAccount::Key(pubkey)
    }
}

impl<'a, 'b> From<&'b AccountInfo<'a>> for ToAccount<'a, 'b> {
    fn from(info: &'b AccountInfo<'a>) -> Self {
        ToAccount::Info(info)
    }
}

/// Arguments for [try_create_account].
pub struct CreateAccount<'a, 'b, 'c> {
    /// The account that will pay for the rent.
    pub from_pubkey: &'c Pubkey,

    /// The account to be created.
    pub to: ToAccount<'a, 'c>,

    /// The space to allocate for the account.
    pub space: u64,

    /// The program to assign the account to.
    pub program_id: &'c Pubkey,

    /// The [AccountInfo] slice provided by the entrypoint.
    pub account_infos: &'c [AccountInfo<'a>],

    /// Seeds for the [Self::from_pubkey] signer if the payer is a System account managed by the
    /// program. Pass in [None] if the payer is passed in as a signer.
    pub from_signer_seeds: Option<&'c [&'b [u8]]>,

    /// Seeds for the [Self::to] signer if the account is a PDA. Pass in [None] if the account is
    /// passed in as a random keypair.
    pub to_signer_seeds: Option<&'c [&'b [u8]]>,
}

/// Create a new account. If the account already has lamports, it will be topped up to the required
/// rent, allocated with the specified amount of space and assigned to the specified program.
///
/// ### Example
///
/// ```
/// use std::ops::Deref;
///
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account_as, NextEnumeratedAccountOptions, DataAccount, Program,
///         Signer,
///     },
///     cpi::system_program::{try_create_account, CreateAccount},
/// };
/// use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
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
///         try_next_enumerated_account_as::<Signer<true>>(&mut accounts_iter, Default::default())?;
///
///     let (new_thing_addr, new_thing_bump) =
///         Pubkey::find_program_address(&[b"thing"], program_id);
///
///     // Next account must be writable data account matching PDA address.
///     let (_, new_account) = try_next_enumerated_account_as::<DataAccount<true>>(
///         &mut accounts_iter,
///         Default::default()
///     )?;
///
///     try_create_account(
///         CreateAccount {
///             from_pubkey: payer.key,
///             to: new_account.deref().into(),
///             space: 16,
///             program_id,
///             account_infos: accounts,
///             from_signer_seeds: None,
///             to_signer_seeds: Some(&[b"thing", &[new_thing_bump]]),
///         })?;
///
///     Ok(())
/// }
/// ```
pub fn try_create_account<'a, 'c>(
    CreateAccount {
        from_pubkey,
        to,
        account_infos,
        space,
        program_id,
        from_signer_seeds,
        to_signer_seeds,
    }: CreateAccount<'a, '_, 'c>,
) -> Result<DataAccount<'a, 'c, true>, ProgramError> {
    let rent_required = Rent::get().map(|rent| rent.minimum_balance(space as usize))?;
    let to_info = match to {
        ToAccount::Key(to_pubkey) => account_infos
            .iter()
            .find(|info| info.key == to_pubkey)
            .ok_or_else(|| {
                SealevelToolsError::CpiSystemProgramCreateAccount(format!(
                    "Cannot find {to_pubkey}"
                ))
            })?,
        ToAccount::Info(to_info) => to_info,
    };

    let current_lamports = to_info.lamports();

    if current_lamports == 0 {
        handle_invoke_signed_from_to(
            &create_account(from_pubkey, to_info.key, rent_required, space, program_id),
            account_infos,
            from_signer_seeds,
            to_signer_seeds,
        )?;
    } else {
        let lamport_diff = rent_required.saturating_sub(current_lamports);

        if lamport_diff != 0 {
            // Transfer remaining lamports.
            handle_invoke_signed_from_to(
                &transfer(from_pubkey, to_info.key, lamport_diff),
                account_infos,
                from_signer_seeds,
                to_signer_seeds,
            )?;
        }

        if space != 0 {
            // Allocate space.
            invoke_signed(
                &allocate(to_info.key, space),
                account_infos,
                &[to_signer_seeds.unwrap_or_default()],
            )?;
        }

        // Assign to specified program.
        invoke_signed(
            &assign(to_info.key, program_id),
            account_infos,
            &[to_signer_seeds.unwrap_or_default()],
        )?;
    }

    // We know that this account was writable, so we are safe to instantiate it like this.
    Ok(DataAccount(to_info))
}

fn handle_invoke_signed_from_to(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    from_signer_seeds: Option<&[&[u8]]>,
    to_signer_seeds: Option<&[&[u8]]>,
) -> ProgramResult {
    match (from_signer_seeds, to_signer_seeds) {
        (Some(from_signer_seeds), Some(to_signer_seeds)) => invoke_signed(
            instruction,
            account_infos,
            &[from_signer_seeds, to_signer_seeds],
        ),
        (None, Some(to_signer_seeds)) => {
            invoke_signed(instruction, account_infos, &[to_signer_seeds])
        }
        (Some(from_signer_seeds), None) => {
            invoke_signed(instruction, account_infos, &[from_signer_seeds])
        }
        (None, None) => invoke_signed(instruction, account_infos, &[]),
    }
}
