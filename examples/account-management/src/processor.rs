use sealevel_tools::{
    account::{AccountSerde, BorshAccountSchema},
    account_info::{
        try_next_enumerated_account, BorshWritableAccount, NextEnumeratedAccountOptions, Payer,
        WritableAccount,
    },
    cpi::system_program::{try_create_serialized_account, FailsafeCreateAccount},
    pda::DeriveAddress,
};
use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::entrypoint::ProgramResult;

use crate::{state::Thing, ID};

pub fn init_thing(accounts: &[NoStdAccountInfo], value: u64) -> ProgramResult {
    let mut accounts_iter = accounts.iter().enumerate();

    // First account will be paying the rent.
    let (_, payer) = try_next_enumerated_account::<Payer>(&mut accounts_iter, Default::default())?;

    let (new_thing_addr, new_thing_bump) = Thing::find_program_address(());

    // Second account is the new Thing.
    let (_, new_thing_account) = try_next_enumerated_account::<WritableAccount>(
        &mut accounts_iter,
        NextEnumeratedAccountOptions {
            key: Some(&new_thing_addr),
            ..Default::default()
        },
    )?;

    let thing = BorshAccountSchema(Thing { value });

    try_create_serialized_account(
        FailsafeCreateAccount {
            payer: payer.as_cpi_authority(),
            to: new_thing_account.as_cpi_authority(Some(&[Thing::SEED, &[new_thing_bump]])),
            space: thing.try_account_space()?,
            program_id: &ID,
        },
        &thing,
    )?;

    Ok(())
}

pub fn update_thing(accounts: &[NoStdAccountInfo], value: u64) -> ProgramResult {
    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the Thing.
    let (_, mut thing_account) = try_next_enumerated_account::<BorshWritableAccount<8, Thing>>(
        &mut accounts_iter,
        Default::default(),
    )?;

    thing_account.data.value = value;
    thing_account.try_write_data()?;

    Ok(())
}

pub fn close_thing(accounts: &[NoStdAccountInfo]) -> ProgramResult {
    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the Thing.
    let (_, thing_account) = try_next_enumerated_account::<BorshWritableAccount<8, Thing>>(
        &mut accounts_iter,
        Default::default(),
    )?;

    // Second account is the beneficiary.
    let (_, beneficiary) =
        try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;

    thing_account.account.try_close(&beneficiary)?;

    Ok(())
}
