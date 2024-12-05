use core::mem::size_of;

use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize confidential mint/burn instruction on the specified Token program,
/// which establishes an authority and ElGamal pubkey for confidential mint/burn. This instruction
/// must be called before a mint is initialized.
#[derive(Clone, PartialEq, Eq)]
pub struct InitializeConfidentialMintBurn<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub supply_elgamal: &'a [u8; 32],
    pub decryptable_supply: &'a [u8; 36],
}

impl<'a> InitializeConfidentialMintBurn<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            supply_elgamal,
            decryptable_supply,
        } = self;

        const IX_DATA_LEN: usize = {
            size_of::<u8>() // token instruction selector
            + size_of::<u8>() // pointer instruction selector
            + 32 // supply_elgamal
            + 36 // decryptoable_supply
        };

        let mut instruction_data = [0; IX_DATA_LEN];

        // Initialize confidential mint/burn selector == 42.
        instruction_data[0] = 42;

        // Initialize extension pointer selector == 0, so no need to set it.
        instruction_data[2..34].copy_from_slice(supply_elgamal);
        instruction_data[34..70].copy_from_slice(decryptable_supply);

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            data: &instruction_data,
        }
        .invoke_signed(&[mint.to_info_c()], &[]);
    }
}
