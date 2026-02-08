use anchor_lang::prelude::*;

#[account]
pub struct UserWhitelist {
    pub vault: Pubkey,
    pub user_limit: u64,
    pub last_updated: i64,
    pub bump: u8,
}