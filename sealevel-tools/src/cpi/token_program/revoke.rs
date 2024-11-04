use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::pubkey::Pubkey;

use crate::cpi::{CpiAuthority, CpiInstruction};

/// Arguments for the revoke instruction on the specified Token program, which revokes the delegated
/// amount on a token account. Only the token account's owner can invoke this instruction.
pub struct Revoke<'a, 'b> {
    pub token_program_id: &'b Pubkey,
    pub source: &'a NoStdAccountInfo,
    pub authority: CpiAuthority<'a, 'b>,
}

impl<'a, 'b> Revoke<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            source,
            authority,
        } = self;

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[source.to_meta_c(), authority.to_meta_c_signer()],
            // Revoke selector == 5.
            data: &[5],
        }
        .invoke_possibly_signed(
            &[source.to_info_c(), authority.to_info_c()],
            &[authority.signer_seeds],
        );
    }
}
