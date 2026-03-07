use pinocchio::error::ProgramError;

pub mod initialize;
pub mod contribute;
pub mod checker;
pub mod refund;

pub use initialize::*;

#[repr(u8)]
pub enum ProgramInstruction {
    InitializeState,
    Contribute,
    Checker,
    Refund
}

impl TryFrom<&u8> for ProgramInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(ProgramInstruction::InitializeState),
            1 => Ok(ProgramInstruction::Contribute),
            2 => Ok(ProgramInstruction::Checker),
            3 => Ok(ProgramInstruction::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}