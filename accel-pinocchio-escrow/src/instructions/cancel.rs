use pinocchio::{
    AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError
};

use crate::state::escrow;

pub fn process_cancel_instruction(
    accounts: &[AccountView],
    _data: &[u8],
) -> ProgramResult {

    let [
        maker,
        escrow_account,
        escrow_ata,
        maker_ata_a,
        _system_program,
        token_program,
        _associated_token_program@ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Read escrow state and extract needed data, then drop the borrow
    let (bump, maker_addr, amount_to_give) = {
        let escrow_state = escrow::Escrow::from_account_info_readonly(&escrow_account)?;
        (escrow_state.bump, escrow_state.maker(), escrow_state.amount_to_give())
    };

    if maker_addr != *maker.address() {
        return Err(ProgramError::IllegalOwner);
    }

    let bump = [bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(maker.address().as_array()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);

    // Transfer tokens from vault back to maker before closing
    pinocchio_token::instructions::Transfer {
        from: escrow_ata,
        to: maker_ata_a,
        authority: escrow_account,
        amount: amount_to_give,
    }.invoke_signed(&[seeds.clone()])?;

    // Now close the vault account
    pinocchio_token::instructions::CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow_account,
    }.invoke_signed(&[seeds])?;


    Ok(())
}