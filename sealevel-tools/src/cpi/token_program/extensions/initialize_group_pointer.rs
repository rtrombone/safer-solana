use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize group member pointer instruction on the specified Token program,
/// which is used to establish a group (collection) of mints. This instruction must be called before
/// a mint is initialized.
#[derive(Clone, PartialEq, Eq)]
pub struct InitializeGroupPointer<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub authority: Option<&'a Pubkey>,
    pub group: Option<&'a Pubkey>,
}

impl<'a> InitializeGroupPointer<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            authority,
            group,
        } = self;

        // Group pointer selector == 40.
        let instruction_data = super::serialize_initialize_pointer_instruction_data(
            40,
            super::unwrap_or_default_pubkey(authority),
            super::unwrap_or_default_pubkey(group),
        );

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            data: &instruction_data,
        }
        .invoke_signed(&[mint.to_info_c()], &[]);
    }
}
