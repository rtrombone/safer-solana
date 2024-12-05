use crate::{
    cpi::{CpiAuthority, CpiInstruction},
    entrypoint::NoStdAccountInfo,
    pubkey::Pubkey,
};

/// Arguments for the freeze account instruction on the specified Token program, which prevents a
/// token account from moving tokens. Only the mint's freeze authority can invoke this instruction.
#[derive(Clone, PartialEq, Eq)]
pub struct FreezeAccount<'a, 'b: 'a> {
    pub token_program_id: &'a Pubkey,
    pub account: &'b NoStdAccountInfo,
    pub mint: &'b NoStdAccountInfo,
    pub freeze_authority: CpiAuthority<'a, 'b>,
}

impl<'a, 'b: 'a> FreezeAccount<'a, 'b> {
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
            // Freeze account selector == 10.
            data: &[10],
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
