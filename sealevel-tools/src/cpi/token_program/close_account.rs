use crate::{
    cpi::{CpiAuthority, CpiInstruction},
    entrypoint::NoStdAccountInfo,
    pubkey::Pubkey,
};

/// Arguments for the close account instruction, which closes a token account or mint account (if
/// the mint close authority extension exists). For token accounts, only the token account's owner
/// can invoke this instruction. For mint accounts with the mint close authority extension, only
/// the mint's close authority can invoke this instruction.
#[derive(Clone, PartialEq, Eq)]
pub struct CloseAccount<'a, 'b: 'a> {
    pub token_program_id: &'a Pubkey,
    pub account: &'b NoStdAccountInfo,
    pub beneficiary: &'b NoStdAccountInfo,
    pub authority: CpiAuthority<'a, 'b>,
}

impl<'a, 'b: 'a> CloseAccount<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            account,
            beneficiary,
            authority,
        } = self;

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                account.to_meta_c(),
                beneficiary.to_meta_c(),
                authority.to_meta_c_signer(),
            ],
            // Close account selector == 9.
            data: &[9],
        }
        .invoke_possibly_signed(
            &[
                account.to_info_c(),
                beneficiary.to_info_c(),
                authority.to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }
}
