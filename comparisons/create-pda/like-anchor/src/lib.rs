use borsh::{BorshDeserialize, BorshSerialize};
use sealevel_tools::{
    account_info::{
        try_next_enumerated_account, DataAccount, NextEnumeratedAccountOptions, Signer,
    },
    cpi::system_program::{try_create_borsh_data_account, CreateAccount},
    discriminator::{Discriminate, Discriminator},
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
    let (_, payer) =
        try_next_enumerated_account::<Signer<true>>(&mut accounts_iter, Default::default())?;

    let (new_thing_addr, new_thing_bump) = Pubkey::find_program_address(&[b"thing"], program_id);

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
            program_id,
            account_infos: accounts,
        },
        &Thing { data: 69 },
    )?;

    Ok(())
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Thing {
    pub data: u64,
}

impl Discriminate<8> for Thing {
    const DISCRIMINATOR: [u8; 8] = Discriminator::Sha2(b"account:Thing").to_bytes();
}
