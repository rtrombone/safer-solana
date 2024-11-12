use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize non-transferable mint instruction on the specified Token program,
/// which only allows minting to token accounts with immutable owners (these tokens cannot be
/// transferred between token accounts). This instruction must be called before a mint is
/// initialized.
pub struct InitializeNonTransferable<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
}

impl<'a> InitializeNonTransferable<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
        } = self;

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            // Initialize non-transferable extension selector == 32.
            data: &[32],
        }
        .invoke_signed(&[mint.to_info_c()], &[]);
    }
}
