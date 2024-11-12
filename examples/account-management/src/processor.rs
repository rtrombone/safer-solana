use sealevel_tools::{
    account::BorshAccountSchema,
    account_info::{
        try_next_enumerated_account, EnumeratedAccountConstraints, MatchDataSlice, Payer,
        WritableAccount,
    },
    cpi::system_program::CreateAccount,
    discriminator::Discriminate,
    entrypoint::{NoStdAccountInfo, ProgramResult},
    pda::DeriveAddress,
};

use crate::{
    state::{Thing, WritableThingAccount},
    ID,
};

#[inline(always)]
pub fn init_thing(accounts: &[NoStdAccountInfo], value: u64) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account will be paying the rent.
    let (_, payer) = try_next_enumerated_account::<Payer>(&mut accounts_iter, Default::default())?;

    let (new_thing_addr, new_thing_bump) = Thing::find_program_address(());

    // Second account is the new Thing.
    let (_, new_thing_account) = try_next_enumerated_account::<WritableAccount>(
        &mut accounts_iter,
        EnumeratedAccountConstraints {
            key: Some(&new_thing_addr),
            ..Default::default()
        },
    )?;

    let thing = BorshAccountSchema(Thing { value });

    // solana_program::log::sol_log_compute_units();

    CreateAccount {
        payer: payer.as_cpi_authority(),
        to: new_thing_account.as_cpi_authority(Some(&[Thing::SEED, &[new_thing_bump]])),
        program_id: &ID,
        space: None,
        lamports: None,
    }
    .try_invoke_and_serialize(&thing)?;

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn update_thing(accounts: &[NoStdAccountInfo], value: u64) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the Thing. We do not need to check that this account is owned by this
    // program because the write will fail if it isn't.
    let (_, mut thing_account) = try_next_enumerated_account::<WritableThingAccount>(
        &mut accounts_iter,
        Default::default(),
    )?;

    // solana_program::log::sol_log_compute_units();

    thing_account.data.value = value;
    thing_account.try_write_data()?;

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn close_thing(accounts: &[NoStdAccountInfo]) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the Thing. We only need to match the discriminator. We do not need to check
    // that this account is owned by this program because the close will fail if it isn't.
    let (_, thing_account) = try_next_enumerated_account::<WritableAccount>(
        &mut accounts_iter,
        EnumeratedAccountConstraints {
            match_data_slice: Some(MatchDataSlice {
                offset: 0,
                data: &Thing::DISCRIMINATOR,
            }),
            ..Default::default()
        },
    )?;

    // Second account is the beneficiary.
    let (_, beneficiary) =
        try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;

    // solana_program::log::sol_log_compute_units();

    thing_account.try_close(&beneficiary)?;

    // solana_program::log::sol_log_compute_units();

    Ok(())
}
