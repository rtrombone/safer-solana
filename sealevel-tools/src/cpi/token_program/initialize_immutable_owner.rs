use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::pubkey::Pubkey;

use crate::cpi::CpiInstruction;

/// Arguments for the initialize immutable owner instruction on the specified Token program, which
/// initializes the immutable owner of a token account. This instruction must be called before a
/// token account is initialized.
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

        _invoke_initialize_immutable_owner(token_program_id, account);
    }
}

#[inline(always)]
pub(super) fn _invoke_initialize_immutable_owner(
    token_program_id: &Pubkey,
    account: &NoStdAccountInfo,
) {
    CpiInstruction {
        program_id: token_program_id,
        accounts: &[account.to_meta_c()],
        // Initialize immutable owner selector == 22.
        data: &[22],
    }
    .invoke_signed(&[account.to_info_c()], &[]);
}
