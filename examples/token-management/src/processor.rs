use sealevel_tools::{
    account_info::{
        try_next_enumerated_account, AnyTokenProgram, DataAccount, NextEnumeratedAccountOptions,
        Signer,
    },
    cpi::{
        token_program::{try_create_mint, CreateMint, CreateMintOptions},
        CpiAccount,
    },
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::ID;

pub fn init_mint(
    accounts: &[AccountInfo],
    decimals: u8,
    mint_authority: Pubkey,
    freeze_authority: Option<Pubkey>,
) -> ProgramResult {
    let mut accounts_iter = accounts.iter().enumerate();

    // First account will be paying the rent.
    let (_, payer) =
        try_next_enumerated_account::<Signer<true>>(&mut accounts_iter, Default::default())?;

    let (new_mint_addr, new_mint_bump) = Pubkey::find_program_address(&[b"mint"], &ID);

    // Second account is which token program to use.
    let (_, token_program) =
        try_next_enumerated_account::<AnyTokenProgram>(&mut accounts_iter, Default::default())?;

    // Third account is the new mint.
    let (_, new_mint_account) = try_next_enumerated_account::<DataAccount<true>>(
        &mut accounts_iter,
        NextEnumeratedAccountOptions {
            key: Some(&new_mint_addr),
            ..Default::default()
        },
    )?;

    try_create_mint(CreateMint {
        token_program_id: token_program.key,
        payer: payer.as_cpi_authority(),
        mint: new_mint_account.as_cpi_authority(Some(&[b"mint", &[new_mint_bump]])),
        mint_authority: CpiAccount::Key(&mint_authority),
        decimals,
        account_infos: accounts,
        opts: CreateMintOptions {
            freeze_authority: freeze_authority.as_ref().map(|key| CpiAccount::Key(key)),
        },
    })?;

    Ok(())
}
