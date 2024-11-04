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

impl<'a, const WRITE: bool> TryFrom<&'a NoStdAccountInfo> for Account<'a, WRITE> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.is_writable() == WRITE {
            Ok(Self(account))
        } else if WRITE {
            Err(SealevelToolsError::AccountInfo(&[
                "Cannot process account as writable",
            ]))
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Cannot process account as read-only",
            ]))
        }
    }
}

impl<'a, const WRITE: bool> Deref for Account<'a, WRITE> {
    type Target = NoStdAccountInfo;

    fn deref(&self) -> &'a Self::Target {
        self.0
    }
}

impl<'a, const WRITE: bool> Account<'a, WRITE> {
    pub fn as_cpi_authority<'b>(
        &'a self,
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

impl<'a> TryFrom<&'a NoStdAccountInfo> for Program<'a> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if !account.is_signer() && account.executable() {
            Ok(Self(account))
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Cannot process account as executable",
            ]))
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

impl<'a, const WRITE: bool> TryFrom<&'a NoStdAccountInfo> for Signer<'a, WRITE> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        // Can a deployed program's keypair still be used as a signer?
        if !account.executable() && account.is_signer() && account.is_writable() == WRITE {
            Ok(Self(account))
        } else if WRITE {
            Err(SealevelToolsError::AccountInfo(&[
                "Cannot process account as writable signer",
            ]))
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Cannot process account as read-only signer",
            ]))
        }
    }
}

impl<'a, const WRITE: bool> Deref for Signer<'a, WRITE> {
    type Target = NoStdAccountInfo;

    fn deref(&self) -> &'a Self::Target {
        self.0
    }
}

impl<'a, const WRITE: bool> Signer<'a, WRITE> {
    pub fn as_cpi_authority<'b>(&'a self) -> CpiAuthority<'a, 'b> {
        CpiAuthority {
            account: self.deref(),
            signer_seeds: None,
        }
    }
}

/// Wrapper for [Account] that deserializes data with [AccountSerde].
pub struct DataAccount<'a, const WRITE: bool, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>> {
    pub account: Account<'a, WRITE>,
    pub data: T,
}

impl<'a, const WRITE: bool, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>>
    TryFrom<Account<'a, WRITE>> for DataAccount<'a, WRITE, DISC_LEN, T>
{
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: Account<'a, WRITE>) -> Result<Self, Self::Error> {
        let data = {
            let data = account.try_borrow_data()?;
            T::try_deserialize_data(&mut &data[..])?
        };

        Ok(Self { account, data })
    }
}

impl<'a, const WRITE: bool, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>>
    TryFrom<&'a NoStdAccountInfo> for DataAccount<'a, WRITE, DISC_LEN, T>
{
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        let account = Account::try_from(account)?;

        account.try_into()
    }
}

impl<'a, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>> DataAccount<'a, true, DISC_LEN, T> {
    /// Write the data to the account.
    pub fn try_write_data(&self) -> ProgramResult {
        let Self { account, data } = self;

        let mut info_data = account.try_borrow_mut_data()?;
        data.try_serialize_data(&mut info_data)
    }
}

impl<'a, const WRITE: bool, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>> Deref
    for DataAccount<'a, WRITE, DISC_LEN, T>
{
    type Target = Account<'a, WRITE>;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

/// Like [try_next_enumerated_account_info], but processes the account as a specific type.
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
pub fn try_next_enumerated_account<'a, T: TryFrom<&'a NoStdAccountInfo>>(
    iter: &mut impl Iterator<Item = (usize, &'a NoStdAccountInfo)>,
    opts: NextEnumeratedAccountOptions,
) -> Result<(usize, T), ProgramError>
where
    ProgramError: From<<T as TryFrom<&'a NoStdAccountInfo>>::Error>,
{
    let (index, account) = try_next_enumerated_account_info(iter, opts)?;

    let processed = T::try_from(account)?;

    Ok((index, processed))
}

/// Trait for composable account structs. This trait is meant to leverage the
/// [try_next_enumerated_account] and [try_next_enumerated_account_info] functions to process an
/// enumerated [NoStdAccountInfo] iterator.
///
/// ### Example
///
/// ```
/// use sealevel_tools::account_info::{
///     Payer, ReadonlyAccount, TakeAccounts, WritableAccount, try_next_enumerated_account,
/// };
/// use solana_nostd_entrypoint::NoStdAccountInfo;
/// use solana_program::program_error::ProgramError;
///
/// struct ComposableAccounts<'a> {
///     thing_one: ReadonlyAccount<'a>,
///     thing_two: WritableAccount<'a>,
/// }
///
/// impl<'a> TakeAccounts<'a> for ComposableAccounts<'a> {
///     fn take_accounts(
///         iter: &mut impl Iterator<Item = (usize, &'a NoStdAccountInfo)>,
///     ) -> Result<Self, ProgramError> {
///         let (_, thing_one) = try_next_enumerated_account(iter, Default::default())?;
///         let (_, thing_two) = try_next_enumerated_account(iter, Default::default())?;
///
///         Ok(Self { thing_one, thing_two })
///     }
/// }
///
/// struct MyAccounts<'a> {
///     payer: Payer<'a>,
///     an_account: ReadonlyAccount<'a>,
///     things: ComposableAccounts<'a>,
/// }
///
/// impl<'a> TakeAccounts<'a> for MyAccounts<'a> {
///     fn take_accounts(
///         iter: &mut impl Iterator<Item = (usize, &'a NoStdAccountInfo)>,
///     ) -> Result<Self, ProgramError> {
///         let (_, payer) = try_next_enumerated_account(iter, Default::default())?;
///         let (_, an_account) = try_next_enumerated_account(iter, Default::default())?;
///
///         Ok(Self {
///             payer,
///             an_account,
///             things: TakeAccounts::take_accounts(iter)?,
///         })
///     }
/// }
/// ```
pub trait TakeAccounts<'a>: Sized {
    fn take_accounts(
        iter: &mut impl Iterator<Item = (usize, &'a NoStdAccountInfo)>,
    ) -> Result<Self, ProgramError>;
}
