use pinocchio::{
    AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError
};
use pinocchio_pubkey::derive_address;

use pinocchio_token::instructions::Transfer;

use crate::state::Escrow;

pub fn process_take_instruction(
    accounts: &[AccountView],
    _data: &[u8],
) -> ProgramResult {

    let [
        taker,
        maker,
        mint_a,
        mint_b,
        escrow_account,
        taker_ata_a,
        taker_ata_b,
        maker_ata_b,
        escrow_ata,
        token_program,
        _associated_token_program@ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !taker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Check maker_ata_b and drop the borrow
    {
        let maker_ata_b_state = pinocchio_token::state::TokenAccount::from_account_view(&maker_ata_b)?;

        if maker_ata_b_state.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if maker_ata_b_state.mint() != mint_b.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    // Check taker_ata_a and drop the borrow
    {
        let taker_ata_a_state = pinocchio_token::state::TokenAccount::from_account_view(&taker_ata_a)?;

        if taker_ata_a_state.owner() != taker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if taker_ata_a_state.mint() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    // Read escrow state and extract needed data, then drop the borrow
    let (bump, amount_to_receive, amount_to_give) = {
        let escrow_state = Escrow::from_account_info_readonly(&escrow_account)?;
        (escrow_state.bump, escrow_state.amount_to_receive(), escrow_state.amount_to_give())
    };

    let seed = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];
    let seeds = &seed[..];

    let escrow_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(escrow_account_pda, *escrow_account.address().as_array());


    let bump = [bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(maker.address().as_array()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);
    {
        Transfer {
            from: escrow_ata,
            to: taker_ata_a,
            authority: escrow_account,
            amount: amount_to_receive,
        }.invoke_signed(&[seeds])?;
    }

    {
        Transfer {
            from: taker_ata_b,
            to: maker_ata_b,
            authority: taker,
            amount: amount_to_give,
        }.invoke()?
    }

    // Update escrow state
    {
        let escrow_state = Escrow::from_account_info(&escrow_account)?;
        escrow_state.set_amount_to_receive(0);
        escrow_state.set_amount_to_give(0);
    }

    Ok(())
}