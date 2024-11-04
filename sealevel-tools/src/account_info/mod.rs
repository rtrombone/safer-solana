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

pub const DEFAULT_NEXT_ENUMERATED_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
    NextEnumeratedAccountOptions {
        key: None,
        any_of_keys: None,
        owner: None,
        any_of_owners: None,
        seeds: None,
        is_signer: None,
        is_writable: None,
        executable: None,
        exact_data_len: None,
        min_data_len: None,
        max_data_len: None,
        match_data_slice: None,
    };

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

    /// If provided, the next account's data length must equal this value.
    pub exact_data_len: Option<usize>,

    /// If provided, the next account's data length must be at least this value.
    pub min_data_len: Option<usize>,

    /// If provided, the next account's data length must be at most this value.
    pub max_data_len: Option<usize>,

    /// If provided, the next account's data must match this slice at the given offset.
    pub match_data_slice: Option<MatchDataSlice<'a>>,
}

#[derive(Debug, Default)]
pub struct MatchDataSlice<'a> {
    pub offset: usize,
    pub data: &'a [u8],
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
        exact_data_len,
        min_data_len,
        max_data_len,
        match_data_slice,
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
                format!("Account index {}: Key mismatch...", index).as_str(),
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
                format!("Account index {}: Key mismatch...", index).as_str(),
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
                format!("Account index {}: Owner mismatch...", index).as_str(),
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
                format!("Account index {}: Owner mismatch...", index).as_str(),
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
                format!("Account index {}: PDA key mismatch...", index).as_str(),
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
                    "Account index {}: Expected signer",
                    index
                )
                .as_str()])
                .into())
            } else {
                Err(SealevelToolsError::AccountInfo(&[format!(
                    "Account index {}: Did not expect signer",
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
                    "Account index {}: Expected writable",
                    index
                )
                .as_str()])
                .into())
            } else {
                Err(SealevelToolsError::AccountInfo(&[format!(
                    "Account index {}: Expected read-only",
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
                    "Account index {}: Expected executable",
                    index
                )
                .as_str()])
                .into())
            } else {
                Err(SealevelToolsError::AccountInfo(&[format!(
                    "Account index {}: Did not expect executable",
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

    if let Some(exact_data_len) = exact_data_len {
        let data_len = account.data_len();

        if data_len != exact_data_len {
            #[cfg(feature = "alloc")]
            return Err(SealevelToolsError::AccountInfo(&[
                format!("Account index {}: Data length mismatch...", index).as_str(),
                format!("  Found: {}", data_len).as_str(),
                format!("  Expected: {}", exact_data_len).as_str(),
            ])
            .into());
            #[cfg(not(feature = "alloc"))]
            return Err(SealevelToolsError::AccountInfo(&[
                "Account does not match expected data length",
            ])
            .into());
        }
    }

    if let Some(min_data_len) = min_data_len {
        let data_len = account.data_len();

        if data_len < min_data_len {
            #[cfg(feature = "alloc")]
            return Err(SealevelToolsError::AccountInfo(&[
                format!("Account index {}: Data length mismatch...", index).as_str(),
                format!("  Found: {}", data_len).as_str(),
                format!("  Expected at least: {}", min_data_len).as_str(),
            ])
            .into());
            #[cfg(not(feature = "alloc"))]
            return Err(SealevelToolsError::AccountInfo(&[
                "Account does not match minimum data length",
            ])
            .into());
        }
    }

    if let Some(max_data_len) = max_data_len {
        let data_len = account.data_len();

        if data_len > max_data_len {
            #[cfg(feature = "alloc")]
            return Err(SealevelToolsError::AccountInfo(&[
                format!("Account index {}: Data length mismatch...", index).as_str(),
                format!("  Found: {}", data_len).as_str(),
                format!("  Expected at most: {}", max_data_len).as_str(),
            ])
            .into());
            #[cfg(not(feature = "alloc"))]
            return Err(SealevelToolsError::AccountInfo(&[
                "Account does not match maximum data length",
            ])
            .into());
        }
    }

    if let Some(MatchDataSlice { offset, data }) = match_data_slice {
        let account_data = account.try_borrow_data()?;
        let end: usize = offset
            .checked_add(data.len())
            .ok_or(ProgramError::AccountDataTooSmall)?;

        if account_data.len() < end {
            #[cfg(feature = "alloc")]
            return Err(SealevelToolsError::AccountInfo(&[
                format!("Account index {}: Data slice mismatch...", index).as_str(),
                format!("  Found: {} bytes", account_data.len()).as_str(),
                format!("  Expected at least: {} bytes", end).as_str(),
            ])
            .into());
            #[cfg(not(feature = "alloc"))]
            return Err(SealevelToolsError::AccountInfo(&[
                "Account data slice does not match expected length",
            ])
            .into());
        }

        if &account_data[offset..end] != data {
            #[cfg(feature = "alloc")]
            {
                return Err(SealevelToolsError::AccountInfo(&[
                    format!(
                        "Account index {}: Data slice mismatch at offset {}...",
                        index, offset
                    )
                    .as_str(),
                    format!(
                        "  Found: {}",
                        bs58::encode(&account_data[offset..offset + data.len()]).into_string()
                    )
                    .as_str(),
                    format!("  Expected: {}", bs58::encode(data).into_string()).as_str(),
                ])
                .into());
            }
            #[cfg(not(feature = "alloc"))]
            return Err(SealevelToolsError::AccountInfo(&[
                "Account data slice does not match expected data",
            ])
            .into());
        }
    }

    Ok((index, account))
}
