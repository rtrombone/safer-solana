use std::ops::Deref;

use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
};

use super::NextEnumeratedAccountOptions;

/// Trait for processing the next enumerated [AccountInfo] with default options. These options can
/// be overridden in the [try_next_enumerated_account_as](super::try_next_enumerated_account_as)
/// method (like checking for a specific key or owner).
pub trait ProcessNextEnumeratedAccount<'a, 'b>: Sized {
    /// Default options for processing the next enumerated account.
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static>;

    /// Only return `Some(Self)` if the account meets the criteria specified by the struct
    /// implementing this trait.
    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self>;
}

/// Generic wrapper for a data account that can be read from or written to (specified by `WRITE`
/// const parameter).
pub struct DataAccount<'a, 'b, const WRITE: bool>(pub(crate) &'b AccountInfo<'a>);

impl<'a, 'b, const WRITE: bool> DataAccount<'a, 'b, WRITE> {
    /// Read data serialized with the [Pack] trait from the account.
    pub fn try_read_pack_data<T: Pack + IsInitialized>(&self) -> Result<T, ProgramError> {
        let data = self.0.try_borrow_data()?;
        T::unpack(&data)
    }

    /// Read data serialized with the [BorshDeserialize](borsh::BorshDeserialize) trait from the
    /// account.
    #[cfg(feature = "borsh")]
    pub fn try_read_borsh_data<const N: usize, T: borsh::BorshDeserialize>(
        &self,
        discriminator: Option<&[u8; N]>,
    ) -> Result<T, ProgramError> {
        let data = self.0.try_borrow_data()?;
        crate::account::try_deserialize_borsh_data(&mut &data[..], discriminator)
            .map_err(Into::into)
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

/// Generic wrapper for a program account.
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

/// Generic wrapper for a signer account that can be read from or written to (specified by `WRITE`
/// const parameter).
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
