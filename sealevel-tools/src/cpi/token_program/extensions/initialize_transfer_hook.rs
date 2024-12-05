use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize transfer hook instruction on the specified Token program, which
/// establishes a program for executing external logic associated with a token transfer. This
/// instruction must be called before a mint is initialized.
#[derive(Clone, PartialEq, Eq)]
pub struct InitializeTransferHook<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub authority: Option<&'a Pubkey>,
    pub program_id: Option<&'a Pubkey>,
}

impl<'a> InitializeTransferHook<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            authority,
            program_id,
        } = self;

        // Transfer hook extension selector == 36.
        let instruction_data = super::serialize_initialize_pointer_instruction_data(
            36,
            super::unwrap_or_default_pubkey(authority),
            super::unwrap_or_default_pubkey(program_id),
        );

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            data: &instruction_data,
        }
        .invoke_signed(&[mint.to_info_c()], &[]);
    }
}
