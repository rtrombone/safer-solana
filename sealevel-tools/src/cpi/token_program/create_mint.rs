use solana_program::{
    account_info::AccountInfo, program::invoke, program_error::ProgramError, program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token_2022::{instruction::initialize_mint2, state::Mint};

use crate::{
    account_info::DataAccount,
    cpi::{
        system_program::{try_create_account, CreateAccount},
        CpiAccount, CpiAuthority,
    },
};

/// Arguments for [try_create_mint].
#[derive(Debug)]
pub struct CreateMint<'a, 'b, 'c> {
    pub token_program_id: &'c Pubkey,
    pub payer: CpiAuthority<'a, 'b, 'c>,
    pub mint: CpiAuthority<'a, 'b, 'c>,
    pub mint_authority: CpiAccount<'a, 'c>,
    pub decimals: u8,
    pub account_infos: &'c [AccountInfo<'a>],
    pub opts: CreateMintOptions<'a, 'c>,
}

/// Optional arguments for [try_create_mint].
#[derive(Debug, Default)]
pub struct CreateMintOptions<'a, 'b> {
    pub freeze_authority: Option<CpiAccount<'a, 'b>>,
}

/// Create a mint account. This method creates an account for one of the Token programs and
/// initializes it as a mint.
///
/// ### Example
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, try_next_enumerated_account_info, AnyTokenProgram,
///         NextEnumeratedAccountOptions, DataAccount, Signer,
///     },
///     cpi::{
///         token_program::{try_create_mint, CreateMint},
///         CpiAccount,
///     },
/// };
/// use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
///
/// fn process_instruction(
///      program_id: &Pubkey,
///      accounts: &[AccountInfo],
///      instruction_data: &[u8],
/// ) -> ProgramResult {
///     let mut accounts_iter = accounts.iter().enumerate();
///
///     // Next account must writable signer (A.K.A. our payer).
///     let (_, payer) =
///         try_next_enumerated_account::<Signer<true>>(&mut accounts_iter, Default::default())?;
///
///     let (new_mint_addr, new_mint_bump) =
///         Pubkey::find_program_address(&[b"mint"], program_id);
///
///     // Next account must be writable data account matching PDA address.
///     let (_, new_mint_account) = try_next_enumerated_account::<DataAccount<true>>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&new_mint_addr),
///             ..Default::default()
///         },
///     )?;
///
///     // Next account is the mint authority.
///     let (_, mint_authority) =
///         try_next_enumerated_account_info(&mut accounts_iter, Default::default())?;
///
///     // Next account is which token program to use.
///     let (_, token_program) =
///         try_next_enumerated_account::<AnyTokenProgram>(&mut accounts_iter, Default::default())?;
///
///     try_create_mint(CreateMint {
///         token_program_id: token_program.key,
///         payer: payer.as_cpi_authority(),
///         mint: new_mint_account.as_cpi_authority(Some(&[b"mint", &[new_mint_bump]])),
///         mint_authority: CpiAccount::Info(&mint_authority),
///         decimals: 9,
///         account_infos: accounts,
///         opts: Default::default(), // No freeze authority specified.
///     })?;
///
///     Ok(())
/// }
/// ```
pub fn try_create_mint<'a, 'c>(
    CreateMint {
        token_program_id,
        payer,
        mint,
        mint_authority,
        decimals,
        account_infos,
        opts: CreateMintOptions { freeze_authority },
    }: CreateMint<'a, '_, 'c>,
) -> Result<DataAccount<'a, 'c, true>, ProgramError> {
    // First create the mint account by assigning it to the token program.
    let mint_account = try_create_account(CreateAccount {
        payer,
        to: mint,
        space: Mint::LEN as u64,
        program_id: token_program_id,
        account_infos,
    })?;

    let instruction = initialize_mint2(
        token_program_id,
        mint_account.key,
        mint_authority.key(),
        freeze_authority.as_ref().map(|account| account.key()),
        decimals,
    )?;

    invoke(&instruction, account_infos)?;

    Ok(mint_account)
}
