//! Error types for Sealevel tools.

use solana_program::{msg, program_error::ProgramError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SealevelToolsError {
    /// Error relating to the [account_info](crate::account_info) module. Custom program error code
    /// reflected by [ACCOUNT_INFO](Self::ACCOUNT_INFO).
    #[error("Account info: {0}")]
    AccountInfo(String),

    /// Error relating to [cpi](crate::cpi) module. Custom program error code reflected by
    /// [CPI](Self::CPI).
    #[error("CPI: {0}: {1}")]
    Cpi(&'static str, String),
}

impl SealevelToolsError {
    pub const ACCOUNT_INFO: u32 = u32::from_be_bytes(
        crate::discriminator::Discriminator::Sha2(b"sealevel_tools::account_info").to_bytes(),
    );
    pub const CPI: u32 = u32::from_be_bytes(
        crate::discriminator::Discriminator::Sha2(b"sealevel_tools::cpi").to_bytes(),
    );
}

impl From<SealevelToolsError> for ProgramError {
    fn from(e: SealevelToolsError) -> ProgramError {
        msg!("Custom error: {}", e);

        match e {
            SealevelToolsError::AccountInfo(_) => {
                ProgramError::Custom(SealevelToolsError::ACCOUNT_INFO)
            }
            SealevelToolsError::Cpi(_, _) => ProgramError::Custom(SealevelToolsError::CPI),
        }
    }
}
