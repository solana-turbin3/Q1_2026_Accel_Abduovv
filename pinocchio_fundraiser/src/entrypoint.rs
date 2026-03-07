#![allow(unexpected_cfgs)]

use crate::instructions::{self, ProgramInstruction};
use pinocchio::{
    default_panic_handler, no_allocator, program_entrypoint,
    error::ProgramError, Address, ProgramResult, AccountView,
};

// This is the entrypoint for the program.
program_entrypoint!(process_instruction);
//Do not allocate memory.
no_allocator!();
// Use the no_std panic handler.
default_panic_handler!();

#[inline(always)]
fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let (ix_disc, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match ProgramInstruction::try_from(ix_disc)? {
        ProgramInstruction::InitializeState => {
            instructions::initialize(accounts, instruction_data)
        }
        ProgramInstruction::Contribute => {
            instructions::contribute::contribute(accounts, instruction_data)
        }
        ProgramInstruction::Checker => {
            instructions::checker::checker(accounts, instruction_data)
        }
        ProgramInstruction::Refund => {
            instructions::refund::refund(accounts, instruction_data)
        }
    }
}