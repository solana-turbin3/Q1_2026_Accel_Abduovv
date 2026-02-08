use anchor_lang::prelude::*;

use crate::state::UserWhitelist;

#[derive(Accounts)]
#[instruction(user: Pubkey, vault: Pubkey)]
pub struct InitializeWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + 4 + 1, // 8 bytes for discriminator, 4 bytes for vector length, 1 byte for bump
        seeds = [b"whitelist", user.key().as_ref()],
        bump
    )]
    pub whitelist: Account<'info, UserWhitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeWhitelist<'info> {
    pub fn initialize_whitelist(&mut self, user_limit: u64, bumps: InitializeWhitelistBumps) -> Result<()> {
        // Initialize the whitelist with an empty address vector
        self.whitelist.set_inner(UserWhitelist { 
            vault: self.whitelist.vault,
            user_limit,
            last_updated: Clock::get()?.unix_timestamp,
            bump: bumps.whitelist
        });

        Ok(())
    }
}