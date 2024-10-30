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

use core::ops::Deref;

use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

use crate::{account::AccountSerde, cpi::CpiAuthority, error::SealevelToolsError};

use super::{try_close_account, try_next_enumerated_account_info, NextEnumeratedAccountOptions};

/// Trait for processing the next enumerated [NoStdAccountInfo] with default options. These options can
/// be overridden in the [try_next_enumerated_account] method (like checking for a specific key or
/// owner).
pub trait ProcessNextEnumeratedAccount<'a>: Sized {
    /// Default options for processing the next enumerated account.
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static>;

    /// Only return `Some(Self)` if the account meets the criteria specified by the struct
    /// implementing this trait.
    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self>;
}

/// Generic wrapper for a data account that can be read from or written to (specified by `WRITE`
/// const parameter).
pub struct Account<'a, const WRITE: bool>(pub(crate) &'a NoStdAccountInfo);

pub type ReadonlyAccount<'a> = Account<'a, false>;
pub type WritableAccount<'a> = Account<'a, true>;

impl<'a> Account<'a, true> {
    pub fn try_close(&self, beneficiary: &Account<'a, true>) -> ProgramResult {
        try_close_account(super::CloseAccount {
            account: self.0,
            beneficiary: beneficiary.0,
        })
    }
}

impl<'a, const WRITE: bool> ProcessNextEnumeratedAccount<'a> for Account<'a, WRITE> {
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

    #[inline(always)]
    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if account.is_writable() == WRITE {
            Some(Self(account))
        } else {
            None
        }
    }
}

impl<'a, const WRITE: bool> Deref for Account<'a, WRITE> {
    type Target = NoStdAccountInfo;

    fn deref(&self) -> &'a Self::Target {
        self.0
    }
}

impl<'b, const WRITE: bool> Account<'b, WRITE> {
    pub fn as_cpi_authority<'a>(
        &'b self,
        signer_seeds: Option<&'b [&'a [u8]]>,
    ) -> CpiAuthority<'a, 'b> {
        CpiAuthority {
            account: self.deref(),
            signer_seeds,
        }
    }
}

/// Generic wrapper for a program account.
pub struct Program<'a>(pub(crate) &'a NoStdAccountInfo);

impl<'a> ProcessNextEnumeratedAccount<'a> for Program<'a> {
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

    #[inline(always)]
    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if account.executable() {
            Some(Self(account))
        } else {
            None
        }
    }
}

impl<'a> Deref for Program<'a> {
    type Target = NoStdAccountInfo;

    fn deref(&self) -> &'a Self::Target {
        self.0
    }
}

/// Generic wrapper for a signer account that can be read from or written to (specified by `WRITE`
/// const parameter).
pub struct Signer<'a, const WRITE: bool>(pub(crate) &'a NoStdAccountInfo);

pub type Authority<'a> = Signer<'a, false>;
pub type Payer<'a> = Signer<'a, true>;

impl<'a, const WRITE: bool> ProcessNextEnumeratedAccount<'a> for Signer<'a, WRITE> {
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

    #[inline(always)]
    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if account.is_signer() && account.is_writable() == WRITE {
            Some(Self(account))
        } else {
            None
        }
    }
}

impl<'a, const WRITE: bool> Deref for Signer<'a, WRITE> {
    type Target = NoStdAccountInfo;

    fn deref(&self) -> &'a Self::Target {
        self.0
    }
}

impl<'b, const WRITE: bool> Signer<'b, WRITE> {
    pub fn as_cpi_authority<'a>(&'b self) -> CpiAuthority<'a, 'b> {
        CpiAuthority {
            account: self.deref(),
            signer_seeds: None,
        }
    }
}

/// Wrapper for [Account] that deserializes data with [Pack]. This type warehouses the
/// deserialized data implementing [IsInitialized] and [Pack].
pub struct DataAccount<'a, const WRITE: bool, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>> {
    pub account: Account<'a, WRITE>,
    pub data: T,
}

impl<'a, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>> DataAccount<'a, true, DISC_LEN, T> {
    /// Write the data to the account.
    pub fn try_write_data(&self) -> ProgramResult {
        let Self { account, data } = self;

        let mut info_data = account.try_borrow_mut_data()?;
        data.try_serialize_data(&mut info_data)
    }
}

impl<'a, const WRITE: bool, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>>
    TryFrom<Account<'a, WRITE>> for DataAccount<'a, WRITE, DISC_LEN, T>
{
    type Error = ProgramError;

    fn try_from(account: Account<'a, WRITE>) -> Result<Self, Self::Error> {
        let data = {
            let data = account.try_borrow_data()?;
            T::try_deserialize_data(&mut &data[..])?
        };

        Ok(Self { account, data })
    }
}

impl<'a, const WRITE: bool, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>>
    ProcessNextEnumeratedAccount<'a> for DataAccount<'a, WRITE, DISC_LEN, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        Account::<'a, WRITE>::NEXT_ACCOUNT_OPTIONS;

    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        let account = Account::checked_new(account)?;

        Self::try_from(account).ok()
    }
}

/// Like [try_next_enumerated_account_info], but processes the account as a specific type
/// implementing [ProcessNextEnumeratedAccount].
///
/// ### Example
///
/// ```
/// use sealevel_tools::account_info::{
///     try_next_enumerated_account, NextEnumeratedAccountOptions, Payer, Program, ReadonlyAccount,
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
///     // Next account must writable signer (A.K.A. our payer).
///     let (index, payer) =
///         try_next_enumerated_account::<Payer>(&mut accounts_iter, Default::default())?;
///
///     // Next account must be read-only data account.
///     let (index, readonly_account) = try_next_enumerated_account::<ReadonlyAccount>(
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
#[inline(always)]
pub fn try_next_enumerated_account<'a, 'b, T>(
    iter: &mut impl Iterator<Item = (usize, &'b NoStdAccountInfo)>,
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
    T: ProcessNextEnumeratedAccount<'b>,
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
        SealevelToolsError::AccountInfo(alloc::format!(
            "index: {}. Cannot process account as {}",
            index,
            core::any::type_name::<T>()
        ))
    })?;

    Ok((index, processed))
}
