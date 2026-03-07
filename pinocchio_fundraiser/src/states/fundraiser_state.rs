use pinocchio::{AccountView, Address, error::ProgramError, sysvars::clock::Clock, sysvars::Sysvar};
use wincode::{SchemaRead, SchemaWrite};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, SchemaRead, SchemaWrite)]
pub struct Fundraiser {
    maker: [u8; 32],
    mint_to_raise: [u8; 32],
    amount_to_raise: [u8; 8],
    current_amount: [u8; 8],
    duration: [u8; 8],
    current_time: [u8; 8],
    pub bump: u8,
}

impl Fundraiser {
    pub const LEN: usize = core::mem::size_of::<Self>();

    pub fn from_account_info(account_info: &AccountView) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut()?;
        if data.len() != Fundraiser::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    pub fn maker(&self) -> Address {
        Address::from(self.maker)
    }

    pub fn set_maker(&mut self, maker: &Address) {
        self.maker.copy_from_slice(maker.as_ref());
    }

    pub fn mint_to_raise(&self) -> Address {
        Address::from(self.mint_to_raise)
    }

    pub fn set_mint_to_raise(&mut self, mint_a: &Address) {
        self.mint_to_raise.copy_from_slice(mint_a.as_ref());
    }

    pub fn amount_to_raise(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_raise)
    }

    pub fn set_amount_to_raise(&mut self, amount: u64) {
        self.amount_to_raise = amount.to_le_bytes();
    }

    pub fn current_amount(&self) -> u64 {
        u64::from_le_bytes(self.current_amount)
    }

    pub fn set_current_amount(&mut self, amount: u64) {
        self.current_amount = amount.to_le_bytes();
    }

    pub fn duration(&self) -> u64 {
        u64::from_le_bytes(self.duration)
    }

    pub fn set_duration(&mut self, amount: u64) {
        self.duration = amount.to_le_bytes();
    }

    pub fn current_time(&self) -> i64 {
        i64::from_le_bytes(self.current_time)
    }

    pub fn set_current_time(&mut self) -> Result<(), ProgramError> {
        self.current_time = Clock::get()?.unix_timestamp.to_le_bytes();
        Ok(())
    }

    pub fn set_inner(&mut self, maker: &Address, mint_to_raise: &Address, amount_to_raise: u64, duration: u64, bump: u8) -> Result<(), ProgramError> {
        self.maker.copy_from_slice(maker.as_ref());
        self.mint_to_raise.copy_from_slice(mint_to_raise.as_ref());
        self.amount_to_raise = amount_to_raise.to_le_bytes();
        self.current_amount = [0; 8];
        self.duration = duration.to_le_bytes();
        self.current_time = Clock::get()?.unix_timestamp.to_le_bytes();
        self.bump = bump;
        Ok(())
    }
}