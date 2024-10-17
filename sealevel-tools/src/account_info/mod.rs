//! [AccountInfo] utilities.

mod process;

pub use process::*;

use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::error::SealevelToolsError;

/// Optional arguments for [try_next_enumerated_account], which specify constraints for the next
/// [AccountInfo].
#[derive(Debug, Default)]
pub struct NextEnumeratedAccountOptions<'a, 'b> {
    /// If provided, the next account's key must equal this pubkey.
    pub key: Option<&'a Pubkey>,

    /// If provided, the next account's key must be one of these pubkeys.
    pub any_of_keys: Option<&'b [&'a Pubkey]>,

    /// If provided, the next account's owner must equal this pubkey.
    pub owner: Option<&'a Pubkey>,

    /// If provided, the next account's owner must be one of these pubkeys.
    pub any_of_owners: Option<&'b [&'a Pubkey]>,

    /// If provided, the next account's key must be derived from these seeds and owner.
    pub seeds: Option<(
        &'b [&'a [u8]], // seeds
        &'b Pubkey,     // owner
    )>,

    /// If provided, the next account's `is_signer` must equal this value.
    pub is_signer: Option<bool>,

    /// If provided, the next account's `is_writable` must equal this value.
    pub is_writable: Option<bool>,

    /// If provided, the next account's `executable` must equal this value.
    pub executable: Option<bool>,
}

/// Similar to [next_account_info](solana_program::account_info::next_account_info), but using an
/// enumerated iterator and optional constraints.
///
/// If any of the constraints are violated, a custom program error code with
/// [SealevelToolsError::ACCOUNT_INFO_NEXT_ENUMERATED_ACCOUNT] is returned, as well as a program
/// log indicating the specific constraint that was violated.
///
/// # Example
///
/// ```
/// use sealevel_tools::account_info::{try_next_enumerated_account, NextEnumeratedAccountOptions};
/// use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
///
/// fn process_instruction(
///      program_id: &Pubkey,
///      accounts: &[AccountInfo],
///      instruction_data: &[u8],
/// ) -> ProgramResult {
///     let mut accounts_iter = accounts.iter().enumerate();
///
///     // Next account must be the clock sysvar.
///     let (index, account) = try_next_enumerated_account(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&solana_program::sysvar::clock::ID),
///             ..Default::default()
///         })?;
///
///     // Next account must be writable.
///     let (index, account) = try_next_enumerated_account(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             is_writable: Some(true),
///             ..Default::default()
///         })?;
///
///     Ok(())
/// }
/// ```
pub fn try_next_enumerated_account<'a, 'b, 'c, I>(
    iter: &mut I,
    NextEnumeratedAccountOptions {
        key,
        any_of_keys,
        owner,
        any_of_owners,
        seeds,
        is_signer,
        is_writable,
        executable,
    }: NextEnumeratedAccountOptions,
) -> Result<I::Item, ProgramError>
where
    I: Iterator<Item = (usize, &'c AccountInfo<'a>)>,
{
    let (index, account) = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;

    if let Some(key) = key {
        if account.key != key {
            return Err(SealevelToolsError::NextEnumeratedAccount(
                index,
                format!("Found key={}, expected={}", account.key, key),
            )
            .into());
        }
    }

    if let Some(any_of_keys) = any_of_keys {
        if !any_of_keys.contains(&account.key) {
            return Err(SealevelToolsError::NextEnumeratedAccount(
                index,
                format!(
                    "Found key={}, expected one of {:?}",
                    account.key, any_of_keys
                ),
            )
            .into());
        }
    }

    if let Some(owner) = owner {
        if account.owner != owner {
            msg!(
                "ProgramError caused by account index={}. Found owner={}, expected={}.",
                index,
                account.owner,
                owner,
            );
            return Err(SealevelToolsError::NextEnumeratedAccount(
                index,
                format!("Found owner={}, expected={}", account.owner, owner),
            )
            .into());
        }
    }

    if let Some(any_of_owners) = any_of_owners {
        if !any_of_owners.contains(&account.owner) {
            return Err(SealevelToolsError::NextEnumeratedAccount(
                index,
                format!(
                    "Found owner={}, expected one of {:?}",
                    account.owner, any_of_owners
                ),
            )
            .into());
        }
    }

    if let Some((seeds, owner)) = seeds {
        let (expected_key, _) = Pubkey::find_program_address(seeds, owner);

        if *account.key != expected_key {
            return Err(SealevelToolsError::NextEnumeratedAccount(
                index,
                format!("Found key={}, derived={}", account.key, expected_key),
            )
            .into());
        }
    }

    if let Some(is_signer) = is_signer {
        if account.is_signer != is_signer {
            return Err(SealevelToolsError::NextEnumeratedAccount(
                index,
                format!("Exected is_signer={}", is_signer),
            )
            .into());
        }
    }

    if let Some(is_writable) = is_writable {
        if account.is_writable != is_writable {
            return Err(SealevelToolsError::NextEnumeratedAccount(
                index,
                format!("Expected is_writable={}", is_writable),
            )
            .into());
        }
    }

    if let Some(executable) = executable {
        if executable != account.executable {
            return Err(SealevelToolsError::NextEnumeratedAccount(
                index,
                format!("Expected executable={}", executable),
            )
            .into());
        }
    }

    Ok((index, account))
}

/// Like [try_next_enumerated_account], but processes the account as a specific type implementing
/// [ProcessNextEnumeratedAccount].
///
/// ### Example
///
/// ```
/// use sealevel_tools::account_info::{
///     try_next_enumerated_account_as, NextEnumeratedAccountOptions, DataAccount, Program, Signer,
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
///     let (index, payer) =
///         try_next_enumerated_account_as::<Signer<true>>(&mut accounts_iter, Default::default())?;
///
///     // Next account must be read-only data account.
///     let (index, readonly_account) = try_next_enumerated_account_as::<DataAccount<false>>(
///         &mut accounts_iter,
///         Default::default()
///     )?;
///
///     // Next account must be System program.
///     let (index, system_program) = try_next_enumerated_account_as::<Program>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&solana_program::system_program::ID),
///             ..Default::default()
///         })?;
///
///     Ok(())
/// }
/// ```
pub fn try_next_enumerated_account_as<'a, 'b, 'c, T>(
    iter: &mut impl Iterator<Item = (usize, &'c AccountInfo<'a>)>,
    NextEnumeratedAccountOptions {
        key,
        any_of_keys,
        owner,
        any_of_owners,
        seeds,
        is_signer,
        is_writable,
        executable,
    }: NextEnumeratedAccountOptions,
) -> Result<(usize, T), ProgramError>
where
    'a: 'c,
    T: ProcessNextEnumeratedAccount<'a, 'c>,
{
    let (index, account) = try_next_enumerated_account(
        iter,
        NextEnumeratedAccountOptions {
            key: key.or(T::NEXT_ACCOUNT_OPTIONS.key),
            any_of_keys: any_of_keys.or(T::NEXT_ACCOUNT_OPTIONS.any_of_keys),
            owner: owner.or(T::NEXT_ACCOUNT_OPTIONS.owner),
            any_of_owners: any_of_owners.or(T::NEXT_ACCOUNT_OPTIONS.any_of_owners),
            seeds: seeds.or(T::NEXT_ACCOUNT_OPTIONS.seeds),
            is_signer: is_signer.or(T::NEXT_ACCOUNT_OPTIONS.is_signer),
            is_writable: is_writable.or(T::NEXT_ACCOUNT_OPTIONS.is_writable),
            executable: executable.or(T::NEXT_ACCOUNT_OPTIONS.executable),
        },
    )?;

    let processed = T::checked_new(account).ok_or_else(|| {
        SealevelToolsError::NextEnumeratedAccount(
            index,
            format!("Cannot process account as {}", std::any::type_name::<T>()),
        )
    })?;

    Ok((index, processed))
}
