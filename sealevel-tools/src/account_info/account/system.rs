use core::ops::Deref;

use crate::{account::system::ID, entrypoint::NoStdAccountInfo, error::SealevelToolsError};

use super::{Account, Program};

/// Wrapper for [Program] for the System program.
#[derive(Clone, PartialEq, Eq)]
pub struct SystemProgram<'a>(pub(crate) Program<'a>);

impl<'a> TryFrom<&'a NoStdAccountInfo> for SystemProgram<'a> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.key() == &ID {
            Ok(Self(Program::try_from(account)?))
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected System program",
            ]))
        }
    }
}

impl<'a> Deref for SystemProgram<'a> {
    type Target = Program<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Account must be owned by the System program.
#[derive(Clone, PartialEq, Eq)]
pub struct SystemAccount<'a, const WRITE: bool>(pub(crate) Account<'a, WRITE>);

/// Read-only account for the System program.
pub type ReadonlySystemAccount<'a> = SystemAccount<'a, false>;

/// Writable account for the System program.
pub type WritableSystemAccount<'a> = SystemAccount<'a, true>;

impl<'a, const WRITE: bool> TryFrom<&'a NoStdAccountInfo> for SystemAccount<'a, WRITE> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.owner() == &ID {
            Account::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected System program account",
            ]))
        }
    }
}

impl<'a, const WRITE: bool> Deref for SystemAccount<'a, WRITE> {
    type Target = Account<'a, WRITE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
