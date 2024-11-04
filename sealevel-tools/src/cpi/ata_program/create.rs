use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::pubkey::Pubkey;

use crate::cpi::{CpiAuthority, CpiInstruction};

/// Arguments for the create instruction on an Associated Token Account program, which creates a
/// token account with an address seeded by its owner and mint.
pub struct Create<'a, 'b> {
    pub associated_token_account_program_id: &'b Pubkey,
    pub payer: CpiAuthority<'a, 'b>,
    pub associated_account: &'a NoStdAccountInfo,
    pub account_owner: &'a NoStdAccountInfo,
    pub mint: &'a NoStdAccountInfo,
    pub system_program: &'a NoStdAccountInfo,
    pub token_program: &'a NoStdAccountInfo,
    pub idempotent: bool,
}

impl<'a, 'b> Create<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        if self.idempotent {
            // Create idempotent selector == 1.
            self._into_invoke(&[1]);
        } else {
            // Empty data for create.
            self._into_invoke(&[]);
        }
    }

    #[inline(always)]
    fn _into_invoke(self, data: &[u8]) {
        let Self {
            associated_token_account_program_id,
            payer,
            associated_account,
            account_owner,
            mint,
            system_program,
            token_program,
            idempotent: _,
        } = self;

        CpiInstruction {
            program_id: associated_token_account_program_id,
            accounts: &[
                payer.to_meta_c_signer(),
                associated_account.to_meta_c(),
                account_owner.to_meta_c(),
                mint.to_meta_c(),
                system_program.to_meta_c(),
                token_program.to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                payer.to_info_c(),
                associated_account.to_info_c(),
                account_owner.to_info_c(),
                mint.to_info_c(),
                system_program.to_info_c(),
                token_program.to_info_c(),
            ],
            &[payer.signer_seeds],
        );
    }
}
