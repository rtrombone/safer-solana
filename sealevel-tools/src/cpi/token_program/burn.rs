use crate::{
    cpi::{CpiAuthority, CpiInstruction},
    entrypoint::NoStdAccountInfo,
    pubkey::Pubkey,
};

/// Arguments for the burn instruction on the specified Token program, which burns a specified
/// amount from a token account. Only the token account's owner or delegated authority can invoke
/// this instruction.
#[derive(Clone, PartialEq, Eq)]
pub struct Burn<'a, 'b: 'a> {
    pub token_program_id: &'a Pubkey,
    pub source: &'b NoStdAccountInfo,
    pub mint: &'b NoStdAccountInfo,
    pub authority: CpiAuthority<'a, 'b>,
    pub amount: u64,
}

impl<'a, 'b: 'a> Burn<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            source,
            mint,
            authority,
            amount,
        } = self;

        // Burn selector == 8.
        let instruction_data = super::serialize_amount_instruction_data(8, amount);

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                authority.to_meta_c_signer(),
            ],
            data: &instruction_data,
        }
        .invoke_possibly_signed(
            &[source.to_info_c(), mint.to_info_c(), authority.to_info_c()],
            &[authority.signer_seeds],
        );
    }
}
