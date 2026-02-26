use pinocchio::{
    AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError, sysvars::{Sysvar, rent::Rent}
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use wincode;
use crate::state::EscrowV2;


pub fn process_make_instruction_v2(
    accounts: &[AccountView],
    data: &[u8],
) -> ProgramResult {

    let [
        maker,
        mint_a,
        mint_b,
        escrow_account,
        maker_ata_a,
        escrow_ata,
        system_program,
        token_program,
        _associated_token_program@ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Check maker_ata_a and drop the borrow
    {
        let maker_ata_a_state = pinocchio_token::state::TokenAccount::from_account_view(&maker_ata_a)?;
        if maker_ata_a_state.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if maker_ata_a_state.mint() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let bump = data[0];
    let seed = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];
    let seeds = &seed[..];

    let escrow_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(escrow_account_pda, *escrow_account.address().as_array());

    let amount_to_receive = unsafe{ *(data.as_ptr().add(1) as *const u64) };
    let amount_to_give = unsafe{ *(data.as_ptr().add(9) as *const u64) };

    let bump = [bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(maker.address().as_array()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);

    unsafe {
        if escrow_account.owner() != &crate::ID {
            CreateAccount {
                from: maker,
                to: escrow_account,
                lamports: Rent::get()?.try_minimum_balance(EscrowV2::LEN)?,
                space: EscrowV2::LEN as u64,
                owner: &crate::ID,
            }.invoke_signed(&[seeds.clone()])?;

            // Create and serialize the escrow state
            let escrow_state = EscrowV2::new(
                *maker.address().as_array(),
                *mint_a.address().as_array(),
                *mint_b.address().as_array(),
                amount_to_receive,
                amount_to_give,
                data[0],
            );

            // Serialize and save to account using direct memory access
            let serialized = wincode::serialize(&escrow_state)
                .map_err(|_| ProgramError::InvalidAccountData)?;
            
            let account_data_ptr = escrow_account.data_ptr();
            core::ptr::copy_nonoverlapping(
                serialized.as_ptr(),
                account_data_ptr,
                serialized.len(),
            );

            pinocchio_associated_token_account::instructions::Create {
                funding_account: maker,
                account: escrow_ata,
                wallet: escrow_account,
                mint: mint_a,
                token_program: token_program,
                system_program: system_program,
            }.invoke()?;

            pinocchio_token::instructions::Transfer {
                from: maker_ata_a,
                to: escrow_ata,
                authority: maker,
                amount: amount_to_give,
            }.invoke()?;

            return Ok(());
        }
        else {
            return Err(ProgramError::IllegalOwner);
        }
    }
}