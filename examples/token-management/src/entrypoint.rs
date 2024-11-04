use borsh::BorshDeserialize;
use solana_nostd_entrypoint::{basic_panic_impl, entrypoint_nostd, NoStdAccountInfo};
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{instruction::ProgramInstruction, processor};

#[inline(always)]
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
        ProgramInstruction::Approve { amount } => processor::approve(accounts, amount),
        ProgramInstruction::Burn { amount } => processor::burn(accounts, amount),
        ProgramInstruction::GetAccountDataSize(extensions) => {
            processor::get_account_data_size(accounts, extensions)
        }
        ProgramInstruction::InitMint {
            decimals,
            freeze_authority,
        } => processor::init_mint(accounts, decimals, freeze_authority),
        ProgramInstruction::InitTokenAccount { owner, immutable } => {
            processor::init_token_account(accounts, owner, immutable)
        }
        ProgramInstruction::MintTo { amount } => processor::mint_to(accounts, amount),
        ProgramInstruction::Revoke => processor::revoke(accounts),
        ProgramInstruction::SuboptimalMintTo { amount } => {
            processor::suboptimal_mint_to(accounts, amount)
        }
        ProgramInstruction::Transfer { amount } => processor::transfer(accounts, amount),
        ProgramInstruction::TransferChecked { amount, decimals } => {
            processor::transfer_checked(accounts, amount, decimals)
        }
    }
}

entrypoint_nostd!(process_instruction, 32);

//noalloc_allocator!();
basic_panic_impl!();
