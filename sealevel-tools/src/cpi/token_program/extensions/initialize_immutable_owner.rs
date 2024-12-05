use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize immutable owner instruction on the specified Token program, which
/// prevents the owner of a token account from being updated. This instruction must be called
/// before a token account is initialized.
#[derive(Clone, PartialEq, Eq)]
pub struct InitializeImmutableOwner<'a> {
    pub token_program_id: &'a Pubkey,
    pub account: &'a NoStdAccountInfo,
}

impl<'a> InitializeImmutableOwner<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            account,
        } = self;

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[account.to_meta_c()],
            // Initialize immutable owner selector == 22.
            data: &[22],
        }
        .invoke_signed(&[account.to_info_c()], &[]);
    }
}
