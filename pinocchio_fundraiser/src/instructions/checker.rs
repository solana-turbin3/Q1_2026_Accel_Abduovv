use pinocchio::cpi::{Seed, Signer};
use pinocchio::{AccountView, ProgramResult, error::ProgramError};
use pinocchio_pubkey::derive_address;
use crate::errors::FundraiserError;
use crate::states::Fundraiser;





pub fn checker(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [
        maker,
        maker_ata,
        mint_to_raise,
        fundraiser_acc,
        vault,
        _token_program,
        _system_program,
        _associated_token_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let fundraiser_state = Fundraiser::from_account_info(fundraiser_acc)?;

    let bump = fundraiser_state.bump;
    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];

    let fundraiser_pda = derive_address(&seed, None, &crate::ID);
    if fundraiser_pda != *fundraiser_acc.address().as_array() {
        return Err(FundraiserError::PdaMismatch.into());
    }

    let vault_state = pinocchio_token::state::TokenAccount::from_account_view(vault)?;
    let mint_to_raise_state = pinocchio_token::state::Mint::from_account_view(mint_to_raise)?;

    if vault_state.amount() != fundraiser_state.amount_to_raise() {
        return Err(FundraiserError::VaultAmountMismatch.into());
    }

    if maker_ata.data_len() == 0 {
        pinocchio_token::instructions::InitializeAccount3 {
            account: maker_ata,
            mint: mint_to_raise,
            owner: maker.address(),
        }.invoke()?;
    }

    let pda_bump_bytes = [bump];
        // signer seeds
    let signer_seeds = [
        Seed::from(b"fundraiser".as_ref()),
        Seed::from(maker.address().as_ref()),
        Seed::from(&pda_bump_bytes[..]),
    ];
    let signers = [Signer::from(&signer_seeds[..])];

    
 {
       pinocchio_token::instructions::TransferChecked {
        from: vault,
        mint: mint_to_raise,
        to: maker_ata,
        authority: fundraiser_acc,
        amount: fundraiser_state.amount_to_raise(),
        decimals: mint_to_raise_state.decimals(),
    }.invoke_signed(&signers)?;
 }

 {
    pinocchio_token::instructions::CloseAccount {
        account: vault,
        destination: maker,
        authority: fundraiser_acc
    }.invoke_signed(&signers.clone())?;
 }

 {
    pinocchio_system::instructions::Transfer {
        from: fundraiser_acc,
        to: maker,
        lamports: fundraiser_acc.lamports(),
    }.invoke_signed(&signers.clone())?;
 }

    Ok(())
}
