use crate::{
    account::ata::ID,
    cpi::{CpiAuthority, CpiInstruction},
    entrypoint::NoStdAccountInfo,
    pubkey::Pubkey,
};

/// Arguments for the create instruction on an Associated Token Account program, which creates a
/// token account with an address seeded by its owner and mint.
#[derive(Clone, PartialEq, Eq)]
pub struct Create<'a, 'b: 'a> {
    /// If your program interacts with a fork of the official ATA program, provide the ID here.
    /// Otherwise, the official ATA program ID will be used by default for CPI.
    pub ata_program_id: Option<&'a Pubkey>,

    pub payer: CpiAuthority<'a, 'b>,
    pub associated_account: &'b NoStdAccountInfo,
    pub account_owner: &'b NoStdAccountInfo,
    pub mint: &'b NoStdAccountInfo,
    pub system_program: &'b NoStdAccountInfo,
    pub token_program: &'b NoStdAccountInfo,
    pub idempotent: bool,
}

impl<'a, 'b: 'a> Create<'a, 'b> {
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
            ata_program_id,
            payer,
            associated_account,
            account_owner,
            mint,
            system_program,
            token_program,
            idempotent: _,
        } = self;

        CpiInstruction {
            program_id: ata_program_id.unwrap_or(&ID),
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
