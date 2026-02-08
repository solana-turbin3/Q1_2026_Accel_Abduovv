use anchor_lang::prelude::*;
use anchor_spl::token_2022::{Mint, TokenAccount, TokenInterface, Burn, burn_checked};

#[derive(Accounts)]
pub struct SlashTokens<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(mut)]
    pub user_token_account: AccountInfo<'info, TokenAccount>,


    pub mint: AccountInfo<'info, Mint>,
    
    pub token_program: Program<'info, TokenInterface>,
}



impl<'info> SlashTokens<'info> {
    pub fn slash_tokens(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Burn {
            mint: self.mint.to_account_info(),
            from: self.user_token_account.to_account_info(),
            authority: self.admin.to_account_info(),
        };
        
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        burn_checked(cpi_ctx, amount, 6)?;
        
        Ok(())
    }
}