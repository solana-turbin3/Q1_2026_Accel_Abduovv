use anchor_lang::{prelude::*};

use crate::state::whitelist::UserWhitelist;

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct WhitelistOperations<'info> {
    #[account(
        mut,
        //address = 
    )]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"whitelist", user.key().as_ref()],
        close = admin,
        bump,
    )]
    pub whitelist: Account<'info, UserWhitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> WhitelistOperations<'info> {
    pub fn close_whitelist(&mut self) -> Result<()> {
        Ok(())
    }

}