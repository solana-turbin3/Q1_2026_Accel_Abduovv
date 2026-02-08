use anchor_lang::{ 
    prelude::*, 
};
use anchor_spl::token_interface::{
    Mint, 
    TokenInterface,
};

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(mut)]
    // address: 
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = admin,
        extensions::transfer_hook::authority = admin,
        extensions::transfer_hook::program_id = crate::ID,
        extensions::permanent_delegate::delegate = admin,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> InitializeMint<'info> {
    pub fn init_mint(&mut self) -> Result<()> {
        Ok(())
    }
}