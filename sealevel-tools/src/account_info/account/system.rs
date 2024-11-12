use core::ops::Deref;

use crate::{entrypoint::NoStdAccountInfo, error::SealevelToolsError};

use super::Program;

/// Wrapper for [Program] for the System program.
pub struct SystemProgram<'a>(pub(crate) Program<'a>);

impl<'a> TryFrom<&'a NoStdAccountInfo> for SystemProgram<'a> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.key() == &solana_program::system_program::ID {
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
