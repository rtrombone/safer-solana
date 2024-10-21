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

use crate::{
    account_info::DataAccount,
    error::SealevelToolsError,
    types::{InputAccount, InputAuthority},
};

/// Arguments for [try_create_account].
pub struct CreateAccount<'a, 'b, 'c> {
    /// The account that will pay for the rent.
    ///
    /// NOTE: Seeds for the [Self::payer] signer if the payer is a System account managed by the
    /// program. Pass in [None] if the payer is passed in as a signer.
    pub payer: InputAuthority<'a, 'b, 'c>,

    /// The account to be created.
    ///
    /// NOTE: Seeds for the [Self::to] signer if the account is a PDA. Pass in [None] if the account
    /// is passed in as a random keypair.
    pub to: InputAuthority<'a, 'b, 'c>,

    /// The space to allocate for the account.
    pub space: u64,

    /// The program to assign the account to.
    pub program_id: &'c Pubkey,

    /// The [AccountInfo] slice provided by the entrypoint.
    pub account_infos: &'c [AccountInfo<'a>],
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
///         try_next_enumerated_account, NextEnumeratedAccountOptions, DataAccount, Program,
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
///     try_create_account(
///         CreateAccount {
///             payer: payer.as_input_authority(),
///             to: new_account.as_input_authority(Some(&[b"thing", &[new_thing_bump]])),
///             space: 16,
///             program_id,
///             account_infos: accounts,
///         })?;
///
///     Ok(())
/// }
/// ```
pub fn try_create_account<'a, 'c>(
    CreateAccount {
        payer:
            InputAuthority {
                account: payer,
                signer_seeds: from_signer_seeds,
            },
        to:
            InputAuthority {
                account: to,
                signer_seeds: to_signer_seeds,
            },
        account_infos,
        space,
        program_id,
    }: CreateAccount<'a, '_, 'c>,
) -> Result<DataAccount<'a, 'c, true>, ProgramError> {
    let from_pubkey = payer.key();

    let rent_required = Rent::get().map(|rent| rent.minimum_balance(space as usize))?;
    let to_info = match to {
        InputAccount::Key(to_pubkey) => account_infos
            .iter()
            .find(|info| info.key == to_pubkey)
            .ok_or_else(|| {
                SealevelToolsError::CpiSystemProgramCreateAccount(format!(
                    "Cannot find {to_pubkey}"
                ))
            })?,
        InputAccount::Info(to_info) => to_info,
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
