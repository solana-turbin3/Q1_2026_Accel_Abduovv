use anchor_lang::prelude::*;
use ephemeral_vrf_sdk::anchor::vrf;
use ephemeral_vrf_sdk::instructions::{create_request_randomness_ix, RequestRandomnessParams};
use ephemeral_vrf_sdk::types::SerializableAccountMeta;
use ephemeral_rollups_sdk::anchor::ephemeral;

pub mod instructions;
pub mod state;

use crate::instructions::*;
use crate::state::*;

const SEED_PREFIX: &[u8; 32] = b"er-state-account-seed-prefix-123";


declare_id!("D74Ho1cWBHgZNpVG4FnBBA4JtjX4HFZ5QqqRXXVKA8gM");


pub const USER_SEED: &[u8] = b"userd2";

#[ephemeral]
#[program]
pub mod er_state_account {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!(
            "Initializing user account: {:?}",
            ctx.accounts.user.key()
        );
        let user = &mut ctx.accounts.user;
        user.data = 0;
        Ok(())
    }


    pub fn roll_dice_delegated(ctx: Context<DoRollDiceDelegatedCtx>) -> Result<()> {
        msg!("Requesting randomness...");
        let ix = create_request_randomness_ix(RequestRandomnessParams {
            payer: ctx.accounts.payer.key(),
            oracle_queue: ctx.accounts.oracle_queue.key(),
            callback_program_id: ID,
            callback_discriminator: instruction::CallbackRollDiceSimple::DISCRIMINATOR.to_vec(),
            caller_seed: *SEED_PREFIX,
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: ctx.accounts.user.key(),
                is_signer: false,
                is_writable: true,
            }]),
            ..Default::default()
        });
        ctx.accounts
            .invoke_signed_vrf(&ctx.accounts.payer.to_account_info(), &ix)?;
        Ok(())
    }

    pub fn callback_roll_dice_simple(
        ctx: Context<CallbackRollDiceSimpleCtx>,
        randomness: [u8; 32],
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let rnd_u8 = ephemeral_vrf_sdk::rnd::random_u8(&randomness);
        msg!("Consuming random number: {:?}", rnd_u8);
        user.data = rnd_u8 as u64;
        Ok(())
    }

    pub fn delegate(ctx: Context<Delegate>) -> Result<()> {
        ctx.accounts.delegate()?;
        
        Ok(())
    }

    pub fn undelegate(ctx: Context<Undelegate>) -> Result<()> {
        ctx.accounts.undelegate()?;
        
        Ok(())
    }

    pub fn close(ctx: Context<CloseUser>) -> Result<()> {
        ctx.accounts.close()?;
        
        Ok(())
    }

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed, 
        payer = payer, 
        space = 8 + 8, // discriminator + data
        seeds = [USER_SEED, payer.key().to_bytes().as_slice()], 
        bump
    )]
    pub user: Account<'info, UserAccount>,
    pub system_program: Program<'info, System>,
}

#[vrf]
#[derive(Accounts)]
pub struct DoRollDiceCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [USER_SEED, payer.key().to_bytes().as_slice()], bump)]
    pub user: Account<'info, UserAccount>,
    /// CHECK: The oracle queue
    #[account(mut, address = ephemeral_vrf_sdk::consts::DEFAULT_QUEUE)]
    pub oracle_queue: AccountInfo<'info>,
}

#[vrf]
#[derive(Accounts)]
pub struct DoRollDiceDelegatedCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [USER_SEED, payer.key().to_bytes().as_slice()], bump)]
    pub user: Account<'info, UserAccount>,
    /// CHECK: The oracle queue
    #[account(mut, address = ephemeral_vrf_sdk::consts::DEFAULT_EPHEMERAL_QUEUE)]
    pub oracle_queue: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CallbackRollDiceCtx<'info> {
    /// This check ensure that the vrf_program_identity (which is a PDA) is a singer
    /// enforcing the callback is executed by the VRF program trough CPI
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, UserAccount>,
}

#[derive(Accounts)]
pub struct CallbackRollDiceSimpleCtx<'info> {
    /// This check ensure that the vrf_program_identity (which is a PDA) is a singer
    /// enforcing the callback is executed by the VRF program trough CPI
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, UserAccount>,
}

}
