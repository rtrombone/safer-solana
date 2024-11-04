//! [NoStdAccountInfo] utilities.

mod account;
mod close;

pub use account::*;
pub use close::*;

#[cfg(feature = "alloc")]
use alloc::format;

use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::SealevelToolsError;

/// Optional arguments for [try_next_enumerated_account_info], which specify constraints for the next
/// [NoStdAccountInfo].
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
/// [SealevelToolsError::ACCOUNT_INFO] is returned, as well as a program
/// log indicating the specific constraint that was violated.
///
/// # Example
///
/// ```
/// use sealevel_tools::account_info::{
///     try_next_enumerated_account_info, NextEnumeratedAccountOptions
/// };
/// use solana_nostd_entrypoint::NoStdAccountInfo;
/// use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
///
/// fn process_instruction(
///      program_id: &Pubkey,
///      accounts: &[NoStdAccountInfo],
///      instruction_data: &[u8],
/// ) -> ProgramResult {
///     let mut accounts_iter = accounts.iter().enumerate();
///
///     // Next account must be the clock sysvar.
///     let (index, account) = try_next_enumerated_account_info(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&solana_program::sysvar::clock::ID),
///             ..Default::default()
///         })?;
///
///     // Next account must be writable.
///     let (index, account) = try_next_enumerated_account_info(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             is_writable: Some(true),
///             ..Default::default()
///         })?;
///
///     Ok(())
/// }
/// ```
#[inline(always)]
pub fn try_next_enumerated_account_info<'a, I>(
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
    I: Iterator<Item = (usize, &'a NoStdAccountInfo)>,
{
    let (index, account) = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;

    if let Some(key) = key {
        if account.key() != key {
            #[cfg(feature = "alloc")]
            return Err(SealevelToolsError::AccountInfo(&[
                format!("Account key mismatch at index {}...", index).as_str(),
                format!("  Found: {}", account.key()).as_str(),
                format!("  Expected: {}", key).as_str(),
            ])
            .into());
            #[cfg(not(feature = "alloc"))]
            return Err(
                SealevelToolsError::AccountInfo(&["Account does not match expected key"]).into(),
            );
        }
    }

    if let Some(any_of_keys) = any_of_keys {
        if !any_of_keys.contains(&account.key()) {
            #[cfg(feature = "alloc")]
            return Err(SealevelToolsError::AccountInfo(&[
                format!("Account key mismatch at index {}...", index).as_str(),
                format!("  Found: {}", account.key()).as_str(),
                format!("  Expected one of: {:?}", any_of_keys).as_str(),
            ])
            .into());
            #[cfg(not(feature = "alloc"))]
            return Err(SealevelToolsError::AccountInfo(&[
                "Account does not match one of expected keys",
            ])
            .into());
        }
    }

    if let Some(owner) = owner {
        if account.owner() != owner {
            #[cfg(feature = "alloc")]
            return Err(SealevelToolsError::AccountInfo(&[
                format!("Account owner mismatch at index {}...", index).as_str(),
                format!("  Found: {}", account.owner()).as_str(),
                format!("  Expected: {:?}", owner).as_str(),
            ])
            .into());
            #[cfg(not(feature = "alloc"))]
            return Err(SealevelToolsError::AccountInfo(&[
                "Account does not match expected owner",
            ])
            .into());
        }
    }

    if let Some(any_of_owners) = any_of_owners {
        if !any_of_owners.contains(&account.owner()) {
            #[cfg(feature = "alloc")]
            return Err(SealevelToolsError::AccountInfo(&[
                format!("Account owner mismatch at index {}...", index).as_str(),
                format!("  Found: {}", account.owner()).as_str(),
                format!("  Expected one of: {:?}", any_of_owners).as_str(),
            ])
            .into());
            #[cfg(not(feature = "alloc"))]
            return Err(SealevelToolsError::AccountInfo(&[
                "Account does not match one of expected owners",
            ])
            .into());
        }
    }

    if let Some((seeds, owner)) = seeds {
        let (expected_key, _) = Pubkey::find_program_address(seeds, owner);

        if *account.key() != expected_key {
            #[cfg(feature = "alloc")]
            return Err(SealevelToolsError::AccountInfo(&[
                format!("PDA key mismatch at index {}...", index).as_str(),
                format!("  Found: {}", account.key()).as_str(),
                format!("  Expected: {}", expected_key).as_str(),
            ])
            .into());
            #[cfg(not(feature = "alloc"))]
            return Err(
                SealevelToolsError::AccountInfo(&["Account does not match derived key"]).into(),
            );
        }
    }

    if let Some(is_signer) = is_signer {
        if account.is_signer() != is_signer {
            #[cfg(feature = "alloc")]
            return if is_signer {
                Err(SealevelToolsError::AccountInfo(&[format!(
                    "Expected signer at index {}",
                    index
                )
                .as_str()])
                .into())
            } else {
                Err(SealevelToolsError::AccountInfo(&[format!(
                    "Did not expect signer at index {}",
                    index
                )
                .as_str()])
                .into())
            };
            #[cfg(not(feature = "alloc"))]
            return if is_signer {
                Err(SealevelToolsError::AccountInfo(&["Expected signer"]).into())
            } else {
                Err(SealevelToolsError::AccountInfo(&["Did not expect signer"]).into())
            };
        }
    }

    if let Some(is_writable) = is_writable {
        if account.is_writable() != is_writable {
            #[cfg(feature = "alloc")]
            return if is_writable {
                Err(SealevelToolsError::AccountInfo(&[format!(
                    "Expected writable at index {}",
                    index
                )
                .as_str()])
                .into())
            } else {
                Err(SealevelToolsError::AccountInfo(&[format!(
                    "Expected read-only at index {}",
                    index
                )
                .as_str()])
                .into())
            };
            #[cfg(not(feature = "alloc"))]
            return if is_writable {
                Err(SealevelToolsError::AccountInfo(&["Expected writable"]).into())
            } else {
                Err(SealevelToolsError::AccountInfo(&["Expected read-only"]).into())
            };
        }
    }

    if let Some(executable) = executable {
        if account.executable() != executable {
            #[cfg(feature = "alloc")]
            return if executable {
                Err(SealevelToolsError::AccountInfo(&[format!(
                    "Expected executable at index {}",
                    index
                )
                .as_str()])
                .into())
            } else {
                Err(SealevelToolsError::AccountInfo(&[format!(
                    "Did not expect executable at index {}",
                    index
                )
                .as_str()])
                .into())
            };
            #[cfg(not(feature = "alloc"))]
            return if executable {
                Err(SealevelToolsError::AccountInfo(&["Expected executable"]).into())
            } else {
                Err(SealevelToolsError::AccountInfo(&["Did not expect executable"]).into())
            };
        }
    }

    Ok((index, account))
}
