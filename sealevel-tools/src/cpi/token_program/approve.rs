use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::pubkey::Pubkey;

use crate::cpi::{CpiAuthority, CpiInstruction};

/// Arguments for the approve instruction on the specified Token program, which allows a delegated
/// authority to move a specified amount from a token account. Only the token account's owner can
/// approve an amount to a delegated authority.
pub struct Approve<'a, 'b> {
    pub token_program_id: &'b Pubkey,
    pub source: &'a NoStdAccountInfo,
    pub delegate: &'a NoStdAccountInfo,
    pub authority: CpiAuthority<'a, 'b>,
    pub amount: u64,
}

impl<'a, 'b> Approve<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            source,
            delegate,
            authority,
            amount,
        } = self;

        // Approve selector == 4.
        let instruction_data = super::serialize_amount_instruction_data(4, amount);

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                delegate.to_meta_c(),
                authority.to_meta_c_signer(),
            ],
            data: &instruction_data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                delegate.to_info_c(),
                authority.to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }
}
