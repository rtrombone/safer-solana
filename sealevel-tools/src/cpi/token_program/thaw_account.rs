use crate::{
    cpi::{CpiAuthority, CpiInstruction},
    entrypoint::NoStdAccountInfo,
    pubkey::Pubkey,
};

/// Arguments for the thaw account instruction on the specified Token program, which unfreezes a
/// token account. Only the mint's freeze authority can invoke this instruction.
pub struct ThawAccount<'a, 'b: 'a> {
    pub token_program_id: &'a Pubkey,
    pub account: &'b NoStdAccountInfo,
    pub mint: &'b NoStdAccountInfo,
    pub freeze_authority: CpiAuthority<'a, 'b>,
}

impl<'a, 'b: 'a> ThawAccount<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            account,
            mint,
            freeze_authority,
        } = self;

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                account.to_meta_c(),
                mint.to_meta_c(),
                freeze_authority.to_meta_c_signer(),
            ],
            // Thaw account selector == 11.
            data: &[11],
        }
        .invoke_possibly_signed(
            &[
                account.to_info_c(),
                mint.to_info_c(),
                freeze_authority.to_info_c(),
            ],
            &[freeze_authority.signer_seeds],
        );
    }
}
