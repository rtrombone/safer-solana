use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::pubkey::Pubkey;

use crate::cpi::{CpiAuthority, CpiInstruction};

/// Arguments for the sync native instruction on the specified Token program, which synchronizes the
/// amount (balance) on the token account with the number of excess lamports on the token account.
/// Performing this call is effectively "wrapping" SOL as the mint representation of SOL.
pub struct SyncNative<'a, 'b> {
    pub token_program_id: &'b Pubkey,
    pub source: &'a NoStdAccountInfo,
    pub authority: CpiAuthority<'a, 'b>,
}

impl<'a, 'b> SyncNative<'a, 'b> {
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
            // Sync native selector == 17.
            data: &[17],
        }
        .invoke_possibly_signed(
            &[source.to_info_c(), authority.to_info_c()],
            &[authority.signer_seeds],
        );
    }
}
