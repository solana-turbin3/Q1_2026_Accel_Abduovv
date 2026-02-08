use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked}
};

use crate::state::{Config, UserWhitelist, ONE_MONTH_SECONDS};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account( seeds = [b"config"], bump=config.bump)]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint=mint,
        associated_token::authority=user,
        associated_token::token_program=token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = config,
        associated_token::token_program=token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
            seeds = [b"whitelist", user.key().as_ref()],
            bump = whitelist.bump
        )]
    pub whitelist: Account<'info, UserWhitelist>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Deposit<'_> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {

        if self.whitelist.last_updated + ONE_MONTH_SECONDS < Clock::get()?.unix_timestamp {
            self.whitelist.user_limit = 0; // Reset user limit after one month
        }

        let cpi_accounts = TransferChecked {
            from: self.user_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer_checked(cpi_ctx, amount, self.mint.decimals)?;
        Ok(())
    }
}