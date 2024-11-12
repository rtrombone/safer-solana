use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize metadata pointer instruction on the specified Token program, which
/// is used to establish a metadata account for a mint. This instruction must be called before a
/// mint is initialized.
pub struct InitializeMetadataPointer<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub authority: Option<&'a Pubkey>,
    pub metadata: Option<&'a Pubkey>,
}

impl<'a> InitializeMetadataPointer<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            authority,
            metadata,
        } = self;

        // Metadata pointer selector == 39.
        let instruction_data = super::serialize_initialize_pointer_instruction_data(
            39,
            super::unwrap_or_default_pubkey(authority),
            super::unwrap_or_default_pubkey(metadata),
        );

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            data: &instruction_data,
        }
        .invoke_signed(&[mint.to_info_c()], &[]);
    }
}
