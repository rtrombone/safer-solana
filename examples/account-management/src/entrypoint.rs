use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{instruction::ProgramInstruction, processor};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if program_id != &crate::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    match BorshDeserialize::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?
    {
        ProgramInstruction::InitThing(data) => processor::init_thing(accounts, data),
        ProgramInstruction::UpdateThing(data) => processor::update_thing(accounts, data),
        ProgramInstruction::CloseThing => processor::close_thing(accounts),
    }
}

solana_program::entrypoint!(process_instruction);
