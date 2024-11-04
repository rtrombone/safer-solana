use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::pubkey::Pubkey;

use crate::cpi::{CpiAuthority, CpiInstruction};

/// Arguments for the mint to instruction on the specified Token program, which mints a specified
/// amount to a token account. Only the mint's authority can invoke this instruction.
///
/// ### Example
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, NextEnumeratedAccountOptions, ReadonlyAccount,
///         TokenProgramWritableAccount, WritableAccount,
///     },
///     cpi::token_program as token_program_cpi,
/// };
/// use solana_nostd_entrypoint::NoStdAccountInfo;
/// use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
///
/// solana_program::declare_id!("Examp1eTokenManagement1111111111111111111111");
///
/// pub fn mint_to(accounts: &[NoStdAccountInfo], amount: u64) -> ProgramResult {
///    let mut accounts_iter = accounts.iter().enumerate();
///
///     // First account is the mint. Disregard checking the mint PDA (but in a real program, you
///     // probably should check). We don't care to deserialize the mint account.
///     let (_, mint_account) = try_next_enumerated_account::<TokenProgramWritableAccount>(
///         &mut accounts_iter,
///         Default::default(),
///     )?;
///
///     let token_program_id = mint_account.owner();
///
///     // Second account is the destination token account. We don't care to deserialize the token
///     // account. No need to check whether this account belongs to a Token program because we
///     // enforce the Token program ID from the mint account.
///     let (_, destination_account) =
///         try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;
///
///     // Third account is the mint authority.
///     //
///     // Since we need the bump for the mint authority's signer seeds, we will find the mint
///     // authority's address. But if this bump were cached, we could disregard the pubkey check
///     // since the signer seeds would be "wrong" for any account that is not the actual mint
///     // authority.
///     let (mint_authority_addr, mint_authority_bump) =
///         Pubkey::find_program_address(&[b"authority"], &ID);
///
///     let (_, mint_authority) = try_next_enumerated_account::<ReadonlyAccount>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&mint_authority_addr),
///             ..Default::default()
///         },
///     )?;
///
///     token_program_cpi::MintTo {
///         token_program_id,
///         mint: &mint_account,
///         destination: &destination_account,
///         mint_authority: mint_authority
///             .as_cpi_authority(Some(&[b"authority", &[mint_authority_bump]])),
///         amount,
///     }
///     .into_invoke();
///
///     Ok(())
/// }
/// ```
pub struct MintTo<'a, 'b> {
    pub token_program_id: &'b Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub destination: &'a NoStdAccountInfo,
    pub mint_authority: CpiAuthority<'a, 'b>,
    pub amount: u64,
}

impl<'a, 'b> MintTo<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            destination,
            mint_authority,
            amount,
        } = self;

        // Mint to selector == 7.
        let instruction_data = super::serialize_amount_instruction_data(7, amount);

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                mint.to_meta_c(),
                destination.to_meta_c(),
                mint_authority.to_meta_c_signer(),
            ],
            data: &instruction_data,
        }
        .invoke_possibly_signed(
            &[
                mint.to_info_c(),
                destination.to_info_c(),
                mint_authority.to_info_c(),
            ],
            &[mint_authority.signer_seeds],
        );
    }
}
