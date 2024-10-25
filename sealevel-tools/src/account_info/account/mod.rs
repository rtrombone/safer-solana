#[cfg(feature = "borsh")]
mod borsh;
mod bpf_loader_upgradeable;
mod pack;
#[cfg(feature = "token")]
mod token;

#[cfg(feature = "borsh")]
pub use borsh::*;
pub use bpf_loader_upgradeable::*;
pub use pack::*;
#[cfg(feature = "token")]
pub use token::*;

use std::ops::Deref;

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
};

use crate::{
    cpi::{CpiAccount, CpiAuthority},
    error::SealevelToolsError,
};

use super::{close_account, try_next_enumerated_account_info, NextEnumeratedAccountOptions};

/// Trait for processing the next enumerated [AccountInfo] with default options. These options can
/// be overridden in the [try_next_enumerated_account] method (like checking for a specific key or
/// owner).
pub trait ProcessNextEnumeratedAccount<'a, 'b>: Sized {
    /// Default options for processing the next enumerated account.
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static>;

    /// Only return `Some(Self)` if the account meets the criteria specified by the struct
    /// implementing this trait.
    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self>;
}

/// Generic wrapper for a data account that can be read from or written to (specified by `WRITE`
/// const parameter).
#[derive(Debug)]
pub struct DataAccount<'a, 'b, const WRITE: bool>(pub(crate) &'b AccountInfo<'a>);

impl<'a, 'b, const WRITE: bool> DataAccount<'a, 'b, WRITE> {
    /// Read data serialized with the [Pack] trait from the account.
    pub fn try_read_pack_data<T: Pack + IsInitialized>(&self) -> Result<T, ProgramError> {
        let data = self.try_borrow_data()?;
        T::unpack(&data)
    }

    /// Read data serialized with the [BorshDeserialize](::borsh::BorshDeserialize) trait from the
    /// account.
    #[cfg(feature = "borsh")]
    pub fn try_read_borsh_data<const N: usize, T: ::borsh::BorshDeserialize>(
        &self,
        discriminator: Option<&[u8; N]>,
    ) -> Result<T, ProgramError> {
        let data = self.try_borrow_data()?;
        crate::account::try_deserialize_borsh_data(&mut &data[..], discriminator)
            .map_err(Into::into)
    }
}

impl<'a, 'b> DataAccount<'a, 'b, true> {
    pub fn close(&self, beneficiary: &DataAccount<'a, 'b, true>) -> ProgramResult {
        close_account(super::CloseAccount {
            account: self.0,
            beneficiary: beneficiary.0,
        })
    }
}

impl<'a, 'b, const WRITE: bool> ProcessNextEnumeratedAccount<'a, 'b>
    for DataAccount<'a, 'b, WRITE>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: None,
            any_of_keys: None,
            owner: None,
            any_of_owners: None,
            seeds: None,
            is_signer: None,
            is_writable: Some(WRITE),
            executable: None,
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if account.is_writable == WRITE {
            Some(Self(account))
        } else {
            None
        }
    }
}

impl<'a, 'b, const WRITE: bool> Deref for DataAccount<'a, 'b, WRITE> {
    type Target = AccountInfo<'a>;

    fn deref(&self) -> &'b Self::Target {
        self.0
    }
}

impl<'a, 'c, const WRITE: bool> DataAccount<'a, 'c, WRITE> {
    pub fn as_cpi_account(&'c self) -> CpiAccount<'a, 'c> {
        CpiAccount::Info(self.deref())
    }

    pub fn as_cpi_authority<'b>(
        &'c self,
        signer_seeds: Option<&'c [&'b [u8]]>,
    ) -> CpiAuthority<'a, 'b, 'c> {
        CpiAuthority {
            account: self.deref().into(),
            signer_seeds,
        }
    }
}

/// Generic wrapper for a program account.
#[derive(Debug)]
pub struct Program<'a, 'b>(pub(crate) &'b AccountInfo<'a>);

impl<'a, 'b> ProcessNextEnumeratedAccount<'a, 'b> for Program<'a, 'b> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: None,
            any_of_keys: None,
            owner: None,
            any_of_owners: None,
            seeds: None,
            is_signer: None,
            is_writable: None,
            executable: Some(true),
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if account.executable {
            Some(Self(account))
        } else {
            None
        }
    }
}

impl<'a, 'b> Deref for Program<'a, 'b> {
    type Target = AccountInfo<'a>;

    fn deref(&self) -> &'b Self::Target {
        self.0
    }
}

impl<'a, 'b> Program<'a, 'b> {
    pub fn as_cpi_account(&'b self) -> CpiAccount<'a, 'b> {
        CpiAccount::Info(self.deref())
    }
}

/// Generic wrapper for a signer account that can be read from or written to (specified by `WRITE`
/// const parameter).
#[derive(Debug)]
pub struct Signer<'a, 'b, const WRITE: bool>(pub(crate) &'b AccountInfo<'a>);

impl<'a, 'b, const WRITE: bool> ProcessNextEnumeratedAccount<'a, 'b> for Signer<'a, 'b, WRITE> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: None,
            any_of_keys: None,
            owner: None,
            any_of_owners: None,
            seeds: None,
            is_signer: Some(true),
            is_writable: Some(WRITE),
            // Can a deployed program's keypair still be used as a signer?
            executable: Some(false),
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if account.is_signer && account.is_writable == WRITE {
            Some(Self(account))
        } else {
            None
        }
    }
}

impl<'a, 'b, const WRITE: bool> Deref for Signer<'a, 'b, WRITE> {
    type Target = AccountInfo<'a>;

    fn deref(&self) -> &'b Self::Target {
        self.0
    }
}

impl<'a, 'c, const WRITE: bool> Signer<'a, 'c, WRITE> {
    pub fn as_cpi_account(&'c self) -> CpiAccount<'a, 'c> {
        CpiAccount::Info(self.deref())
    }

    pub fn as_cpi_authority<'b>(&'c self) -> CpiAuthority<'a, 'b, 'c> {
        CpiAuthority {
            account: self.deref().into(),
            signer_seeds: None,
        }
    }
}

/// Like [try_next_enumerated_account_info], but processes the account as a specific type implementing
/// [ProcessNextEnumeratedAccount].
///
/// ### Example
///
/// ```
/// use sealevel_tools::account_info::{
///     try_next_enumerated_account, NextEnumeratedAccountOptions, DataAccount, Program, Signer,
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
///         try_next_enumerated_account::<Signer<true>>(&mut accounts_iter, Default::default())?;
///
///     // Next account must be read-only data account.
///     let (index, readonly_account) = try_next_enumerated_account::<DataAccount<false>>(
///         &mut accounts_iter,
///         Default::default()
///     )?;
///
///     // Next account must be System program.
///     let (index, system_program) = try_next_enumerated_account::<Program>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&solana_program::system_program::ID),
///             ..Default::default()
///         })?;
///
///     Ok(())
/// }
/// ```
pub fn try_next_enumerated_account<'a, 'b, 'c, T>(
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
    let (index, account) = try_next_enumerated_account_info(
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
        SealevelToolsError::AccountInfo(format!(
            "index: {}. Cannot process account as {}",
            index,
            std::any::type_name::<T>()
        ))
    })?;

    Ok((index, processed))
}
