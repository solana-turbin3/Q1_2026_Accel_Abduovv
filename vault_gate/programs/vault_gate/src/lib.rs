use anchor_lang::prelude::*;

mod instructions;
mod state;

use instructions::*;

use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::{
    instruction::{
        ExecuteInstruction, 
        InitializeExtraAccountMetaListInstruction
    },
};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

declare_id!("8Gx91H8N6u2d4pQ3NQzhsJ3b92Zm4iF83mcoyCDRyjoK");

#[program]
pub mod whitelist_transfer_hook {
    use super::*;

    pub fn initialize_mint(ctx: Context<InitializeMint>) -> Result<()> {
        ctx.accounts.init_mint()
    }

    pub fn initialize_whitelist(ctx: Context<InitializeWhitelist>, user_limit: u64) -> Result<()> {
        ctx.accounts.initialize_whitelist(user_limit, ctx.bumps)
    }

    pub fn close_whitelist(ctx: Context<WhitelistOperations>) -> Result<()> {
        ctx.accounts.close_whitelist()
    }

    #[instruction(discriminator = InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn initialize_transfer_hook(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {

        msg!("Initializing Transfer Hook...");

        // Get the extra account metas for the transfer hook
        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;

        msg!("Extra Account Metas: {:?}", extra_account_metas);
        msg!("Extra Account Metas Length: {}", extra_account_metas.len());

        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas
        )?;

        Ok(())
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        // Call the transfer hook logic
        ctx.accounts.transfer_hook(amount)
    }

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        ctx.accounts.init_vault(&ctx.bumps)
    }

    pub fn mint(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        ctx.accounts.mint_token(amount)
    }
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }
}
