use pinocchio::{AccountView, error::ProgramError};
use wincode::{SchemaRead, SchemaWrite};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, SchemaRead, SchemaWrite)]
pub struct ContributeState {
    amount: [u8; 8],
    pub bump: u8,
}

impl ContributeState {
    pub const LEN: usize = core::mem::size_of::<Self>();

    pub fn from_account_info(account_info: &AccountView) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut()?;
        if data.len() != ContributeState::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

 

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    pub fn set_amount(&mut self, amount: u64) {
        self.amount = amount.to_le_bytes();
    }

    pub fn set_inner(&mut self, amount: u64, bump: u8) {
        self.amount = amount.to_le_bytes();
        self.bump = bump;
    }
}