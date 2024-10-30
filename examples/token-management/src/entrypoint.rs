use borsh::BorshDeserialize;
use solana_nostd_entrypoint::{basic_panic_impl, entrypoint_nostd, NoStdAccountInfo};
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{instruction::ProgramInstruction, processor};

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[NoStdAccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if program_id != &crate::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    match BorshDeserialize::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?
    {
        ProgramInstruction::InitMint {
            decimals,
            mint_authority,
            freeze_authority,
        } => processor::init_mint(accounts, decimals, mint_authority, freeze_authority),
    }
}

entrypoint_nostd!(process_instruction, 32);

//noalloc_allocator!();
basic_panic_impl!();
