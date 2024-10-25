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
        &token_program_id,
        mint_account.key,
        mint_authority.key(),
        freeze_authority.as_ref().map(|account| account.key()),
        decimals,
    )?;

    invoke(&instruction, account_infos)?;

    Ok(mint_account)
}
