use anchor_lang::prelude::*;

pub const CONFIG_SEED: &[u8] = b"config";
pub const WHITELIST_SEED: &[u8] = b"whitelist";
pub const ONE_MONTH_SECONDS: i64 = 30 * 24 * 60 * 60; // Approximate number of seconds in one month

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub vault: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}