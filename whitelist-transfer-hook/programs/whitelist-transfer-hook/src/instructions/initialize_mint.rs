use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    Mint,
    TokenInterface,
};

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = authority,
        extensions::transfer_hook::authority = authority.key(),
        extensions::transfer_hook::program_id = crate::ID,
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeMint<'info> {
    pub fn initialize_mint(&self) -> Result<()> {
        
        msg!("Initializing Mint with Transfer Hook...");

        Ok(())
    }
}