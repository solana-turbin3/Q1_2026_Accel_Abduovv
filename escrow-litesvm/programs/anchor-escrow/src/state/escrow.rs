use anchor_lang::prelude::*;

pub const AFTER_FIVE_DAYS: i64 = 432_000; // 5 days in seconds

#[account]
#[derive(InitSpace, Debug)]
pub struct Escrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub created_at: i64,
    pub bump: u8,
}