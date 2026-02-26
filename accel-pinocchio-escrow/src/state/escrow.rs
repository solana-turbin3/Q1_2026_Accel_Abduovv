use pinocchio::{AccountView, error::ProgramError};
use wincode::{SchemaRead, SchemaWrite};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Escrow {
    maker: [u8; 32],
    mint_a: [u8; 32],
    mint_b: [u8; 32],
    amount_to_receive: [u8; 8],
    amount_to_give: [u8; 8],
    pub bump: u8,
}

#[repr(C)]
#[derive(SchemaRead, SchemaWrite)]
pub struct EscrowV2 {
    maker: [u8; 32],
    mint_a: [u8; 32],
    mint_b: [u8; 32],
    amount_to_receive: [u8; 8],
    amount_to_give: [u8; 8],
    pub bump: u8,
}



impl Escrow  {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + 1;

    pub fn from_account_info(account_info: &AccountView) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut()?;
        if data.len() != Escrow::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    pub fn from_account_info_readonly(account_info: &AccountView) -> Result<&Self, ProgramError> {
        let data = account_info.try_borrow()?;
        if data.len() != Escrow::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }

    pub fn maker(&self) -> pinocchio::Address {
        pinocchio::Address::from(self.maker)
    }

    pub fn set_maker(&mut self, maker: &pinocchio::Address) {
        self.maker.copy_from_slice(maker.as_ref());
    }

    pub fn mint_a(&self) -> pinocchio::Address {
        pinocchio::Address::from(self.mint_a)
    }

    pub fn set_mint_a(&mut self, mint_a: &pinocchio::Address) {
        self.mint_a.copy_from_slice(mint_a.as_ref());
    }

    pub fn mint_b(&self) -> pinocchio::Address {
        pinocchio::Address::from(self.mint_b)
    }

    pub fn set_mint_b(&mut self, mint_b: &pinocchio::Address) {
        self.mint_b.copy_from_slice(mint_b.as_ref());
    }

    pub fn amount_to_receive(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_receive)
    }

    pub fn set_amount_to_receive(&mut self, amount: u64) {
        self.amount_to_receive = amount.to_le_bytes();
    }

    pub fn amount_to_give(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_give)
    }

    pub fn set_amount_to_give(&mut self, amount: u64) {
        self.amount_to_give = amount.to_le_bytes();
    }
}


impl EscrowV2 {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + 1;

    pub fn new(
        maker: [u8; 32],
        mint_a: [u8; 32],
        mint_b: [u8; 32],
        amount_to_receive: u64,
        amount_to_give: u64,
        bump: u8,
    ) -> Self {
        Self {
            maker,
            mint_a,
            mint_b,
            amount_to_receive: amount_to_receive.to_le_bytes(),
            amount_to_give: amount_to_give.to_le_bytes(),
            bump,
        }
    }

    pub fn from_account_info(account_info: &AccountView) -> Result<Self, ProgramError> {
        let data = account_info.try_borrow()?;
        if data.len() != EscrowV2::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        wincode::deserialize(&data).map_err(|_| ProgramError::InvalidAccountData)
    }

    pub fn save_to_account(&self, account_info: &AccountView) -> Result<(), ProgramError> {
        let mut data = account_info.try_borrow_mut()?;
        if data.len() != EscrowV2::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let serialized = wincode::serialize(self).map_err(|_| ProgramError::InvalidAccountData)?;
        data.copy_from_slice(&serialized);
        Ok(())
    }

    pub fn maker(&self) -> pinocchio::Address {
        pinocchio::Address::from(self.maker)
    }

    pub fn set_maker(&mut self, maker: &pinocchio::Address) {
        self.maker.copy_from_slice(maker.as_ref());
    }

    pub fn mint_a(&self) -> pinocchio::Address {
        pinocchio::Address::from(self.mint_a)
    }

    pub fn set_mint_a(&mut self, mint_a: &pinocchio::Address) {
        self.mint_a.copy_from_slice(mint_a.as_ref());
    }

    pub fn mint_b(&self) -> pinocchio::Address {
        pinocchio::Address::from(self.mint_b)
    }

    pub fn set_mint_b(&mut self, mint_b: &pinocchio::Address) {
        self.mint_b.copy_from_slice(mint_b.as_ref());
    }

    pub fn amount_to_receive(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_receive)
    }

    pub fn set_amount_to_receive(&mut self, amount: u64) {
        self.amount_to_receive = amount.to_le_bytes();
    }

    pub fn amount_to_give(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_give)
    }

    pub fn set_amount_to_give(&mut self, amount: u64) {
        self.amount_to_give = amount.to_le_bytes();
    }
}
