use pinocchio::{AccountView, ProgramResult, error::ProgramError, Address};
use pinocchio::sysvars::{Sysvar, rent::Rent, clock::Clock};
use pinocchio_pubkey::derive_address;
use crate::{constants::{MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER}, states::{ContributeState, Fundraiser}};
use crate::errors::FundraiserError;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::TransferChecked;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contribute {
    pub bump: u8,
    pub amount: u64,
}

pub fn contribute(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [
        contributor,
        contributer_account,
        mint_to_raise,
        contributer_ata,
        fundraiser_acc,
        vault,
        maker,
        _token_program,
        _system_program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !contributor.is_signer() {
        return Err(FundraiserError::MissingRequiredSignature.into());
    }

    let fundraiser_state = Fundraiser::from_account_info(fundraiser_acc)?;

    let clock = Clock::get()?;

    let bump = fundraiser_state.bump;
    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];

    let fundraiser_pda = derive_address(&seed, None, &crate::ID);
    if fundraiser_pda != *fundraiser_acc.address().as_array() {
        return Err(FundraiserError::PdaMismatch.into());
    }

    // Parse contribution amount from instruction data
    if data.is_empty() {
        return Err(FundraiserError::InvalidContributionAmount.into());
    }
    let bump_contribute = data[0];
    let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());


    let mint_state = pinocchio_token::state::Mint::from_account_view(mint_to_raise)?;
    let max_contribution = (fundraiser_state.amount_to_raise() * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER;

    // Check if contribution amount exceeds max allowed (10% of total raise)
    if amount > max_contribution {
        return Err(FundraiserError::ContributionExceedsMax.into());
    }

    // Check if fundraiser is still active (within duration)
    let fundraiser_end_time = fundraiser_state.current_time() + (fundraiser_state.duration() as i64);
    if clock.unix_timestamp > fundraiser_end_time {
        return Err(FundraiserError::FundraiserExpired.into());
    }

    // Check if contribution would exceed the fundraising goal
    if fundraiser_state.current_amount() + amount > fundraiser_state.amount_to_raise() {
        return Err(FundraiserError::ContributionExceedsMax.into());
    }

    // Create or update contributor account
    if contributer_account.data_len() == 0 {

        let rent = Rent::get()?;
        let signer_seeds = [
            pinocchio::cpi::Seed::from(b"contribute".as_ref()),
            pinocchio::cpi::Seed::from(contributor.address().as_ref()),
            pinocchio::cpi::Seed::from(fundraiser_acc.address().as_ref()),
        ];
        let signers = [pinocchio::cpi::Signer::from(&signer_seeds[..])];

        CreateAccount {
            from: contributor,
            to: contributer_account,
            lamports: rent.try_minimum_balance(ContributeState::LEN)?,
            space: ContributeState::LEN as u64,
            owner: &Address::from(crate::ID),
        }
        .invoke_signed(&signers)?;

        // Initialize the contribute state
        let contribute_state = ContributeState::from_account_info(contributer_account)?;
        contribute_state.set_inner(amount, bump_contribute);
    } else {
        // Update existing contributor account
        let contribute_state = ContributeState::from_account_info(contributer_account)?;
        let new_amount = contribute_state.amount() + amount;

        // Check if new total exceeds max contribution limit
        if new_amount > max_contribution {
            return Err(FundraiserError::ContributionExceedsMax.into());
        }
        
        contribute_state.set_amount(new_amount);
    }

    // Transfer tokens from contributor to vault
    TransferChecked {
        from: contributer_ata,
        mint: mint_to_raise,
        to: vault,
        authority: contributor,
        amount,
        decimals: mint_state.decimals(),
    }
    .invoke()?;

    // Update fundraiser current amount
    fundraiser_state.set_current_amount(fundraiser_state.current_amount() + amount);

    Ok(())
}
