//! Error types for this crate.

use crate::{log::sol_log, program_error::ProgramError};

#[derive(Debug)]
pub enum SealevelToolsError<'a> {
    PassThrough(ProgramError),

    /// Error relating to the [`account_info` module]. Custom program error code
    /// reflected by [ACCOUNT_INFO].
    ///
    /// [`account_info` module]: crate::account_info
    /// [ACCOUNT_INFO]: Self::ACCOUNT_INFO
    AccountInfo(&'a [&'a str]),

    /// Error relating to [`cpi` module]. Custom program error code reflected by
    /// [CPI].
    ///
    /// [`cpi` module]: crate::cpi
    /// [CPI]: Self::CPI
    Cpi(&'a [&'a str]),
}

impl<'a> SealevelToolsError<'a> {
    pub const ACCOUNT_INFO: u32 = u32::from_be_bytes(
        crate::discriminator::Discriminator::Sha2(b"sealevel_tools::account_info").to_bytes(),
    );
    pub const CPI: u32 = u32::from_be_bytes(
        crate::discriminator::Discriminator::Sha2(b"sealevel_tools::cpi").to_bytes(),
    );
}

impl<'a> From<SealevelToolsError<'a>> for ProgramError {
    fn from(e: SealevelToolsError) -> ProgramError {
        let (msgs, code) = match e {
            SealevelToolsError::PassThrough(err) => return err,
            SealevelToolsError::AccountInfo(err) => {
                sol_log("Custom error: AccountInfo");
                (err, SealevelToolsError::ACCOUNT_INFO)
            }
            SealevelToolsError::Cpi(err) => {
                sol_log("Custom error: CPI");
                (err, SealevelToolsError::CPI)
            }
        };

        msgs.iter().for_each(|err| sol_log(err));

        ProgramError::Custom(code)
    }
}

impl<'a> From<ProgramError> for SealevelToolsError<'a> {
    fn from(e: ProgramError) -> SealevelToolsError<'a> {
        SealevelToolsError::PassThrough(e)
    }
}
