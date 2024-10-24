use sealevel_tools::{
    account_info::{
        try_next_enumerated_account, BorshDataAccount, DataAccount, NextEnumeratedAccountOptions,
        Signer,
    },
    cpi::system_program::{try_create_borsh_data_account, CreateAccount},
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{state::Thing, ID};

pub fn init_thing(accounts: &[AccountInfo], value: u64) -> ProgramResult {
    let mut accounts_iter = accounts.iter().enumerate();

    // First account will be paying the rent.
    let (_, payer) =
        try_next_enumerated_account::<Signer<true>>(&mut accounts_iter, Default::default())?;

    let (new_thing_addr, new_thing_bump) = Pubkey::find_program_address(&[b"thing"], &ID);

    // Second account is the new Thing.
    let (_, new_thing_account) = try_next_enumerated_account::<DataAccount<true>>(
        &mut accounts_iter,
        NextEnumeratedAccountOptions {
            key: Some(&new_thing_addr),
            ..Default::default()
        },
    )?;

    try_create_borsh_data_account(
        CreateAccount {
            payer: payer.as_input_authority(),
            to: new_thing_account.as_input_authority(Some(&[b"thing", &[new_thing_bump]])),
            space: 16,
            program_id: &ID,
            account_infos: accounts,
        },
        &Thing { value },
    )?;

    Ok(())
}

pub fn update_thing(accounts: &[AccountInfo], value: u64) -> ProgramResult {
    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the Thing.
    let (_, mut thing_account) = try_next_enumerated_account::<BorshDataAccount<true, 8, Thing>>(
        &mut accounts_iter,
        Default::default(),
    )?;

    thing_account.data.value = value;
    thing_account.try_write_data()?;

    Ok(())
}

pub fn close_thing(accounts: &[AccountInfo]) -> ProgramResult {
    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the Thing.
    let (_, thing_account) = try_next_enumerated_account::<BorshDataAccount<true, 8, Thing>>(
        &mut accounts_iter,
        Default::default(),
    )?;

    // Second account is the beneficiary.
    let (_, beneficiary) =
        try_next_enumerated_account::<DataAccount<true>>(&mut accounts_iter, Default::default())?;

    thing_account.account.close(&beneficiary)?;

    Ok(())
}
