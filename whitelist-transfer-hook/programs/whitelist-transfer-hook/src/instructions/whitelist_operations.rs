use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::state::Whitelist;

#[derive(Accounts)]
pub struct InitializeWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: It's safe because we don't read or write from this account
    pub user: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + Whitelist::INIT_SPACE,
        seeds = [b"whitelist", user.key().as_ref()],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}


impl<'info> InitializeWhitelist<'info> {
    pub fn initialize_whitelist(&mut self, bumps: InitializeWhitelistBumps) -> Result<()> {
        // Initialize the whitelist with is_whitelisted set to false by default
        self.whitelist.set_inner(Whitelist { 
            is_whitelisted: true,
            user: self.user.key(),
            bump: bumps.whitelist,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct UpdateWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account()]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"whitelist", whitelist.user.as_ref()],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
}

impl<'info> UpdateWhitelist<'info> {
    pub fn update_whitelist(&mut self, is_whitelisted: bool) -> Result<()> {
        self.whitelist.is_whitelisted = is_whitelisted;
        msg!("Whitelist updated: {} is_whitelisted = {}", self.whitelist.user, is_whitelisted);
        Ok(())
    }
}