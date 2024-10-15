use std::ops::Deref;

use borsh::{BorshDeserialize, BorshSerialize};
use sealevel_tools::{
    account_info::{
        try_next_enumerated_account_as, DataAccount, NextEnumeratedAccountOptions, Signer,
    },
    cpi::system_program::create_account::{try_create_borsh_data_account, CreateAccount},
    discriminator::Discriminator,
};
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

declare_id!("Examp1eCreateAccountLikeAnchor1111111111111");

const INITIALIZE_SELECTOR: [u8; 8] = Discriminator::Sha2(b"global:init_thing").to_bytes();

solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if instruction_data != INITIALIZE_SELECTOR {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut accounts_iter = accounts.iter().enumerate();

    // First account will be paying the rent.
    let from_pubkey =
        try_next_enumerated_account_as::<Signer<true>>(&mut accounts_iter, Default::default())
            .map(|(_, signer)| signer.key)?;

    let (new_thing_addr, new_thing_bump) = Pubkey::find_program_address(&[b"thing"], program_id);

    // Second account is the new Thing.
    let (_, new_thing_account) = try_next_enumerated_account_as::<DataAccount<true>>(
        &mut accounts_iter,
        NextEnumeratedAccountOptions {
            key: Some(&new_thing_addr),
            ..Default::default()
        },
    )?;

    try_create_borsh_data_account(
        CreateAccount {
            from_pubkey,
            to: new_thing_account.deref().into(),
            space: 16,
            program_id,
            account_infos: accounts,
            from_signer_seeds: None,
            to_signer_seeds: Some(&[b"thing", &[new_thing_bump]]),
        },
        &Thing { data: 69 },
        Some(&Thing::DISCRIMINATOR),
    )?;

    Ok(())
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Thing {
    pub data: u64,
}

impl Thing {
    pub const DISCRIMINATOR: [u8; 8] = Discriminator::Sha2(b"account:Thing").to_bytes();
}
