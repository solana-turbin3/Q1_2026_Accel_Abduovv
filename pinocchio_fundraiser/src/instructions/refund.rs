use pinocchio::{AccountView, ProgramResult, error::ProgramError};
use pinocchio::sysvars::{Sysvar, clock::Clock};
use pinocchio_pubkey::derive_address;
use crate::errors::FundraiserError;
use crate::states::{ContributeState, Fundraiser};
use pinocchio_token::instructions::TransferChecked;

/// Refund instruction - allows contributors to get their tokens back if fundraiser duration is over without
/// raising enough funds
pub fn refund(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [
        contributor,
        contribute_account,
        mint_to_raise,
        contributor_ata,
        fundraiser_acc,
        vault,
        maker,
        _token_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Either contributor or maker must sign
    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let fundraiser_state = Fundraiser::from_account_info(fundraiser_acc)?;

    // Verify vault belongs to this fundraiser
    let bump = fundraiser_state.bump;
    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];
    let fundraiser_pda = derive_address(&seed, None, &crate::ID);
    if fundraiser_pda != *fundraiser_acc.address().as_array() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let clock = Clock::get()?;

    let current_time = clock.unix_timestamp;

    // Check if fundraiser has expired (elapsed time > duration)
    let elapsed_time = current_time - fundraiser_state.current_time();
    if elapsed_time < fundraiser_state.duration() as i64 {
        return Err(FundraiserError::FundraiserExpired.into());
    }

    // Get contributor's contribution amount
    let contribute_state = ContributeState::from_account_info(contribute_account)?;
    let refund_amount = contribute_state.amount();

    if refund_amount == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Verify contributor account PDA
    let contribute_seed = [
        b"contribute".as_ref(),
        contributor.address().as_ref(),
        fundraiser_acc.address().as_ref(),
    ];
    let contribute_pda = derive_address(&contribute_seed, None, &crate::ID);
    if contribute_pda != *contribute_account.address().as_array() {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Get mint decimals for transfer
    let mint_state = pinocchio_token::state::Mint::from_account_view(mint_to_raise)?;

    // Transfer tokens from vault back to contributor
    let pda_bump_bytes = [bump];
    let signer_seeds = [
        pinocchio::cpi::Seed::from(b"fundraiser".as_ref()),
        pinocchio::cpi::Seed::from(maker.address().as_ref()),
        pinocchio::cpi::Seed::from(&pda_bump_bytes[..]),
    ];
    let signers = [pinocchio::cpi::Signer::from(&signer_seeds[..])];

    TransferChecked {
        from: vault,
        mint: mint_to_raise,
        to: contributor_ata,
        authority: fundraiser_acc,
        amount: refund_amount,
        decimals: mint_state.decimals(),
    }
    .invoke_signed(&signers)?;


    fundraiser_state.set_current_amount(fundraiser_state.current_amount() - refund_amount);

    // Reset contribution amount to zero
    contribute_state.set_amount(0);

    // Close contribute account and return lamports to contributor
    let bump_bytes = [contribute_state.bump];
    let signer_seeds = [
        pinocchio::cpi::Seed::from(b"contribute".as_ref()),
        pinocchio::cpi::Seed::from(contributor.address().as_ref()),
        pinocchio::cpi::Seed::from(fundraiser_acc.address().as_ref()),
        pinocchio::cpi::Seed::from(&bump_bytes[..]),
    ];
    let signers = [pinocchio::cpi::Signer::from(&signer_seeds[..])];

    pinocchio_system::instructions::Transfer {
        from: contribute_account,
        to: contributor,
        lamports: contribute_account.lamports(),
    }
    .invoke_signed(&signers)?;

    Ok(())
}
