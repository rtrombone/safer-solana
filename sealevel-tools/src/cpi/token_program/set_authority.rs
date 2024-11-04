use core::mem::size_of;

use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::pubkey::Pubkey;

use crate::cpi::{CpiAuthority, CpiInstruction};

use super::AuthorityType;

/// Arguments for the set authority instruction on the specified Token program, which sets a new
/// authority for either mint or token account (depending on the [AuthorityType]). Only the current
/// authority of the given account can invoke this instruction.
pub struct SetAuthority<'a, 'b> {
    pub token_program_id: &'b Pubkey,
    pub account: &'a NoStdAccountInfo,
    pub authority: CpiAuthority<'a, 'b>,
    pub authority_type: AuthorityType,
    pub new_authority: Option<&'b Pubkey>,
}

impl<'a, 'b> SetAuthority<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            account,
            authority,
            authority_type,
            new_authority,
        } = self;

        const IX_DATA_LEN: usize = 1 // selector
        + size_of::<u8>() // authority_type
        + size_of::<u8>() + size_of::<Pubkey>(); // new_authority

        let mut instruction_data = [0; IX_DATA_LEN];

        // Set authority selector == 6.
        instruction_data[0] = 6;
        instruction_data[1] = authority_type as u8;

        if let Some(new_authority) = new_authority {
            instruction_data[2] = 1;
            instruction_data[3..35].copy_from_slice(&new_authority.to_bytes());
        }

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[account.to_meta_c(), authority.to_meta_c_signer()],
            data: &instruction_data,
        }
        .invoke_possibly_signed(
            &[account.to_info_c(), authority.to_info_c()],
            &[authority.signer_seeds],
        );
    }
}
