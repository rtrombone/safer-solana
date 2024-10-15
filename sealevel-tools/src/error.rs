use solana_program::{msg, program_error::ProgramError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SealevelToolsError {
    #[error("Account at index {0}: {1}")]
    NextEnumeratedAccount(usize, String),

    #[error("Create account: {0}")]
    CpiSystemProgramCreateAccount(String),
}

impl SealevelToolsError {
    pub const ACCOUNT_INFO_NEXT_ENUMERATED_ACCOUNT: u32 = u32::from_be_bytes(
        crate::discriminator::Discriminator::Sha2(b"account_info::try_next_enumerated_account")
            .to_bytes(),
    );
    pub const CPI_SYSTEM_PROGRAM_CREATE_ACCOUNT: u32 = u32::from_be_bytes(
        crate::discriminator::Discriminator::Sha2(b"cpi::system_program::try_create_account")
            .to_bytes(),
    );
}

impl From<SealevelToolsError> for ProgramError {
    fn from(e: SealevelToolsError) -> ProgramError {
        msg!("Custom error: {}", e);

        match e {
            SealevelToolsError::NextEnumeratedAccount(_, _) => {
                ProgramError::Custom(SealevelToolsError::ACCOUNT_INFO_NEXT_ENUMERATED_ACCOUNT)
            }
            SealevelToolsError::CpiSystemProgramCreateAccount(_) => {
                ProgramError::Custom(SealevelToolsError::CPI_SYSTEM_PROGRAM_CREATE_ACCOUNT)
            }
        }
    }
}
