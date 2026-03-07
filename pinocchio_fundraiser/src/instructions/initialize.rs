use pinocchio::{
    AccountView,
    cpi::{Seed, Signer},
    error::ProgramError,
    ProgramResult,
    sysvars::rent::Rent,
    sysvars::Sysvar,
    Address,
};
use pinocchio_pubkey::derive_address;

use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::InitializeAccount;

use crate::states::Fundraiser;
use crate::constants::MIN_AMOUNT_TO_RAISE;

pub fn initialize(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [
        maker,
        fundraiser_acc,
        vault,
        mint_to_raise,
        sysvar_rent_acc,
        _system_program]
        = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if fundraiser_acc.data_len() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Parse instruction data (bump: u8, amount: u64, duration: u64)
    if data.len() < 17 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let bump = data[0];
    let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
    let mint_to_raise_state = pinocchio_token::state::Mint::from_account_view(mint_to_raise)?;

    if amount < MIN_AMOUNT_TO_RAISE.pow(mint_to_raise_state.decimals() as u32) {
        return Err(ProgramError::InvalidInstructionData);
    }

    let duration = u64::from_le_bytes(data[9..17].try_into().unwrap());

    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];

    let fundraiser_pda = derive_address(&seed, None, &crate::ID);
    assert_eq!(fundraiser_pda, *fundraiser_acc.address().as_array());

    let rent = Rent::get()?;

    let pda_bump_bytes = [bump];

    // signer seeds
    let signer_seeds = [
        Seed::from(b"fundraiser".as_ref()),
        Seed::from(maker.address().as_ref()),
        Seed::from(&pda_bump_bytes[..]),
    ];
    let signers = [Signer::from(&signer_seeds[..])];

    CreateAccount {
        from: maker,
        to: fundraiser_acc,
        space: Fundraiser::LEN as u64,
        owner: &Address::from(crate::ID),
        lamports: rent.try_minimum_balance(Fundraiser::LEN)?,
    }
    .invoke_signed(&signers)?;

    InitializeAccount {
        account: vault,
        mint: mint_to_raise,
        owner: fundraiser_acc,
        rent_sysvar: sysvar_rent_acc,
    }
    .invoke()?;

    let fundraiser = Fundraiser::from_account_info(fundraiser_acc)?;
    fundraiser.set_inner(maker.address(), mint_to_raise.address(), amount, duration, bump)?;

    Ok(())
}