use borsh::BorshDeserialize;
use sealevel_tools::{
    account_info::{
        try_next_enumerated_account, AnyTokenProgram, DataAccount, NextEnumeratedAccountOptions,
        Signer,
    },
    cpi::token_program::{try_create_mint, CreateMint},
    discriminator::Discriminator,
};
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

declare_id!("Examp1eCreateMintPdaLikeAnchor1111111111111");

const INITIALIZE_SELECTOR: [u8; 8] = Discriminator::Sha2(b"global:init_mint").to_bytes();

solana_program::entrypoint!(process_instruction);

#[derive(BorshDeserialize)]
struct InstructionData {
    decimals: u8,
    mint_authority: Pubkey,
    freeze_authority: Option<Pubkey>,
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if instruction_data[..INITIALIZE_SELECTOR.len()] != INITIALIZE_SELECTOR {
        return Err(ProgramError::InvalidInstructionData);
    }

    let InstructionData {
        decimals,
        mint_authority,
        freeze_authority,
    } = BorshDeserialize::try_from_slice(&instruction_data[INITIALIZE_SELECTOR.len()..])
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let mut accounts_iter = accounts.iter().enumerate();

    // First account will be paying the rent.
    let (_, payer) =
        try_next_enumerated_account::<Signer<true>>(&mut accounts_iter, Default::default())?;

    let (new_mint_addr, new_mint_bump) = Pubkey::find_program_address(&[b"mint"], program_id);

    // Second account is which token program to use.
    let token_program_id =
        try_next_enumerated_account::<AnyTokenProgram>(&mut accounts_iter, Default::default())
            .map(|(_, token_program)| token_program.key)?;

    // Third account is the new mint.
    let (_, new_mint_account) = try_next_enumerated_account::<DataAccount<true>>(
        &mut accounts_iter,
        NextEnumeratedAccountOptions {
            key: Some(&new_mint_addr),
            ..Default::default()
        },
    )?;

    try_create_mint(CreateMint {
        token_program_id,
        payer: payer.as_input_authority(),
        mint: new_mint_account.as_input_authority(Some(&[b"mint", &[new_mint_bump]])),
        mint_authority_pubkey: &mint_authority,
        freeze_authority_pubkey: freeze_authority.as_ref(),
        decimals,
        account_infos: accounts,
    })?;

    Ok(())
}
