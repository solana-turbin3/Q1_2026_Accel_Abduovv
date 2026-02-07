#[cfg(test)]
mod tests {

    use {
        anchor_lang::{
            AccountDeserialize, InstructionData, ToAccountMetas, prelude::{
                Clock, msg
            }, solana_program::program_pack::Pack
        }, anchor_spl::{
            associated_token::{
                self, 
                spl_associated_token_account
            }, 
            token::spl_token
        }, litesvm::LiteSVM, litesvm_token::{
            CreateAssociatedTokenAccount, CreateMint, MintTo, spl_token::ID as TOKEN_PROGRAM_ID
        }, solana_instruction::Instruction, solana_keypair::Keypair, solana_message::Message, solana_native_token::LAMPORTS_PER_SOL, solana_pubkey::Pubkey, solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID, solana_signer::Signer, solana_transaction::Transaction, std::
            path::PathBuf
    };

    static PROGRAM_ID: Pubkey = crate::ID;
    const AFTER_FIVE_DAYS: i64 = 432_000; // 5 days in seconds

    struct Setup {
         program: LiteSVM,
         mint_a: Pubkey,
         mint_b: Pubkey,
         maker: Keypair,
         maker_ata_a: Pubkey,
         maker_ata_b: Pubkey,
         taker: Keypair,
         taker_ata_a: Pubkey,
         taker_ata_b: Pubkey,
         escrow: Pubkey,
         vault: Pubkey,
         associated_token_program: Pubkey,
         token_program: Pubkey,
         system_program: Pubkey,
    }

    // Setup function to initialize LiteSVM and create a payer keypair
    // Also loads an account from devnet into the LiteSVM environment (for testing purposes)
    fn setup() -> Setup {
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let payer = Keypair::new();
        let taker = Keypair::new();
    
        // Airdrop some SOL to the payer keypair
        program
            .airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to maker");

        program
            .airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to taker");
    
        // Load program SO file
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/anchor_escrow.so");
    
        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
    
        program.add_program(PROGRAM_ID, &program_data);

        // // Example on how to Load an account from devnet
        // // LiteSVM does not have access to real Solana network data since it does not have network access,
        // // so we use an RPC client to fetch account data from devnet
        // let rpc_client = RpcClient::new("https://api.devnet.solana.com");
        // let account_address = Address::from_str("DRYvf71cbF2s5wgaJQvAGkghMkRcp5arvsK2w97vXhi2").unwrap();
        // let fetched_account = rpc_client
        //     .get_account(&account_address)
        //     .expect("Failed to fetch account from devnet");

        // // Set the fetched account in the LiteSVM environment
        // // This allows us to simulate interactions with this account during testing
        // program.set_account(payer.pubkey(), Account { 
        //     lamports: fetched_account.lamports, 
        //     data: fetched_account.data, 
        //     owner: Pubkey::from(fetched_account.owner.to_bytes()), 
        //     executable: fetched_account.executable, 
        //     rent_epoch: fetched_account.rent_epoch 
        // }).unwrap();

        // msg!("Lamports of fetched account: {}", fetched_account.lamports);

        // Get the maker's public key from the payer keypair
        let maker = payer.pubkey();
        
        // Create two mints (Mint A and Mint B) with 6 decimal places and the maker as the authority
        // This done using litesvm-token's CreateMint utility which creates the mint in the LiteSVM environment
        let mint_a = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        msg!("Mint A: {}\n", mint_a);

        let mint_b = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        msg!("Mint B: {}\n", mint_b);

        // Create the maker's associated token account for Mint A
        // This is done using litesvm-token's CreateAssociatedTokenAccount utility
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&maker).send().unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
            .owner(&maker).send().unwrap();
        msg!("Maker ATA B: {}\n", maker_ata_b);

        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&taker.pubkey()).send().unwrap();
        msg!("Taker ATA A: {}\n", taker_ata_a);

        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
            .owner(&taker.pubkey()).send().unwrap();
        msg!("Taker ATA B: {}\n", taker_ata_b);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker.as_ref(), &123u64.to_le_bytes()],
            &PROGRAM_ID
        ).0;
        msg!("Escrow PDA: {}\n", escrow);

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
        msg!("Vault PDA: {}\n", vault);

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 2000000000)
            .send()
            .unwrap();

        MintTo::new(&mut program, &payer, &mint_b, &taker_ata_b, 5000000000)
            .send()
            .unwrap();

        // Return all the setup data needed for the tests
        let setup = Setup {
            program,
            mint_a,
            mint_b,
            maker: payer,
            maker_ata_a,
            maker_ata_b,
            taker, 
            taker_ata_a, 
            taker_ata_b,
            escrow,
            vault,
            associated_token_program: spl_associated_token_account::ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        };

        setup
    }

    fn make_operation(setup: &Setup, deposit: u64, seed: u64, receive: u64) -> Instruction {
        Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: setup.maker.pubkey(),
                mint_a: setup.mint_a,
                mint_b: setup.mint_b,
                maker_ata_a: setup.maker_ata_a,
                escrow: setup.escrow,
                vault: setup.vault,
                associated_token_program: setup.associated_token_program,
                token_program: setup.token_program,
                system_program: setup.system_program,
            }.to_account_metas(None),
            data: crate::instruction::Make {deposit, seed, receive }.data(),
        }
        
    }

    #[test]
    fn test_make() {

        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let mut setup = setup();

 
        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = make_operation(&setup, 10, 123u64, 10);

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&setup.maker.pubkey()));
        let recent_blockhash = setup.program.latest_blockhash();

        let transaction = Transaction::new(&[&setup.maker], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = setup.program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);


        // Verify the vault account and escrow account data after the "Make" instruction
        let vault_account = setup.program.get_account(&setup.vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10);
        assert_eq!(vault_data.owner, setup.escrow);
        assert_eq!(vault_data.mint, setup.mint_a);

        let escrow_account = setup.program.get_account(&setup.escrow).unwrap();
        let escrow_data = crate::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
        msg!("Escrow Data: {:?}", escrow_data);
        msg!("Vault Balance: {}", vault_data.amount);
        msg!("Maker ATA A Balance: {}", {
            let maker_ata_a_account = setup.program.get_account(&setup.maker_ata_a).unwrap();
            let maker_ata_a_data = spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();
            maker_ata_a_data.amount
        });
        assert_eq!(escrow_data.seed, 123u64);
        assert_eq!(escrow_data.maker, setup.maker.pubkey());
        assert_eq!(escrow_data.mint_a, setup.mint_a);
        assert_eq!(escrow_data.mint_b, setup.mint_b);
        assert_eq!(escrow_data.receive, 10);
        
    }

    #[test]
    fn test_take() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let mut setup = setup();

        // First, perform the "Make" operation to set up the escrow
        let make_ix = make_operation(&setup, 10, 123u64, 10);
        let message = Message::new(&[make_ix], Some(&setup.maker.pubkey()));
        let recent_blockhash = setup.program.latest_blockhash();
        let transaction = Transaction::new(&[&setup.maker], message, recent_blockhash);
        setup.program.send_transaction(transaction).unwrap();

        // Simulate passage of time by manipulating the Clock sysvar in LiteSVM
        let mut clock: Clock = setup.program.get_sysvar();
        clock.unix_timestamp += AFTER_FIVE_DAYS; // Move time forward beyond 5 days
        setup.program.set_sysvar(&clock);

        // Now, create the "Take" instruction to fulfill the escrow
        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Take {
                taker: setup.taker.pubkey(),
                maker: setup.maker.pubkey(),
                mint_a: setup.mint_a,
                mint_b: setup.mint_b,
                taker_ata_a: setup.taker_ata_a,
                taker_ata_b: setup.taker_ata_b,
                maker_ata_b: setup.maker_ata_b,
                escrow: setup.escrow,
                vault: setup.vault,
                associated_token_program: setup.associated_token_program,
                token_program: setup.token_program,
                system_program: setup.system_program,
            }.to_account_metas(None),
            data: crate::instruction::Take {}.data(),
        };

        // Create and send the transaction containing the "Take" instruction
        let message = Message::new(&[take_ix], Some(&setup.taker.pubkey()));
        let recent_blockhash = setup.program.latest_blockhash();
        let transaction = Transaction::new(&[&setup.taker], message, recent_blockhash);
        let tx = setup.program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nTake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);
        msg!("Taker: {}", setup.taker.pubkey());
        msg!("Taker ATA A: {}", setup.taker_ata_a);
        msg!("Taker ATA A Balance: {}", {
            let taker_ata_a_account = setup.program.get_account(&setup.taker_ata_a).unwrap();
            let taker_ata_a_data = spl_token::state::Account::unpack(&taker_ata_a_account.data).unwrap();
            taker_ata_a_data.amount
        });
        msg!("Taker ATA B: {}", setup.taker_ata_b);
        msg!("Taker ATA B Balance: {}", {
            let taker_ata_b_account = setup.program.get_account(&setup.taker_ata_b).unwrap();
            let taker_ata_b_data = spl_token::state::Account::unpack(&taker_ata_b_account.data).unwrap();
            taker_ata_b_data.amount
        });

        // Verify the final balances of the taker and maker after the "Take" instruction
        let taker_ata_a_account = setup.program.get_account(&setup.taker_ata_a).unwrap();
        let taker_ata_a_data = spl_token::state::Account::unpack(&taker_ata_a_account.data).unwrap();
        assert_eq!(taker_ata_a_data.amount, 10);
    }

    #[test]
    fn test_refund() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let mut setup = setup();

        // First, perform the "Make" operation to set up the escrow
        let make_ix = make_operation(&setup, 10, 123u64, 10);
        let message = Message::new(&[make_ix], Some(&setup.maker.pubkey()));
        let recent_blockhash = setup.program.latest_blockhash();
        let transaction = Transaction::new(&[&setup.maker], message, recent_blockhash);
        setup.program.send_transaction(transaction).unwrap();

       
        let maker_initial_balance = {
            let maker_ata_a_account = setup.program.get_account(&setup.maker_ata_a).unwrap();
            let maker_ata_a_data = spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();
            maker_ata_a_data.amount
        };

        // Now, create the "Refund" instruction to refund the escrow
        let refund_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Refund {
                maker: setup.maker.pubkey(),
                escrow: setup.escrow,
                vault: setup.vault,
                maker_ata_a: setup.maker_ata_a,
                token_program: setup.token_program,
                system_program: setup.system_program,
                mint_a: setup.mint_a,
            }.to_account_metas(None),
            data: crate::instruction::Refund {}.data(),
        };

        // Create and send the transaction containing the "Refund" instruction
        let message = Message::new(&[refund_ix], Some(&setup.maker.pubkey()));
        let recent_blockhash = setup.program.latest_blockhash();
        let transaction = Transaction::new(&[&setup.maker], message, recent_blockhash);
        let tx = setup.program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nRefund transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        let maker_after_refund = {
            let maker_ata_a_account = setup.program.get_account(&setup.maker_ata_a).unwrap();
            let maker_ata_a_data = spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();
            maker_ata_a_data.amount
        };

        let vault_account = setup.program.get_account(&setup.vault).unwrap();
        let escrow_account = setup.program.get_account(&setup.escrow).unwrap();

        // Verify the final balance of the maker after the "Refund" instruction
        msg!("Maker initial balance: {}", maker_initial_balance);
        msg!("Maker balance after refund: {}", maker_after_refund);
        assert_eq!(maker_initial_balance + 10, maker_after_refund); // Original 1,000 + refunded 10
        assert_eq!(vault_account.lamports + vault_account.data.len() as u64, 0); // Vault should be empty after refund
        assert_eq!(escrow_account.lamports + escrow_account.data.len() as u64, 0); // Escrow account should be closed after refund


    }

    #[test]
    fn test_take_before_time_passed() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let mut setup = setup();

        // First, perform the "Make" operation to set up the escrow
        let make_ix = make_operation(&setup, 10, 123u64, 10);
        let message = Message::new(&[make_ix], Some(&setup.maker.pubkey()));
        let recent_blockhash = setup.program.latest_blockhash();
        let transaction = Transaction::new(&[&setup.maker], message, recent_blockhash);
        setup.program.send_transaction(transaction).unwrap();

        // Now, create the "Take" instruction to try to fulfill the escrow before time has passed
        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Take {
                taker: setup.taker.pubkey(),
                maker: setup.maker.pubkey(),
                mint_a: setup.mint_a,
                mint_b: setup.mint_b,
                taker_ata_a: setup.taker_ata_a,
                taker_ata_b: setup.taker_ata_b,
                maker_ata_b: setup.maker_ata_b,
                escrow: setup.escrow,
                vault: setup.vault,
                associated_token_program: setup.associated_token_program,
                token_program: setup.token_program,
                system_program: setup.system_program,
            }.to_account_metas(None),
            data: crate::instruction::Take {}.data(),
        };

        // Create and send the transaction containing the "Take" instruction
        let message = Message::new(&[take_ix], Some(&setup.taker.pubkey()));
        let recent_blockhash = setup.program.latest_blockhash();
        let transaction = Transaction::new(&[&setup.taker], message, recent_blockhash);
        
        // This should fail because time has not passed yet
        let result = setup.program.send_transaction(transaction);
        
        // Assert that the transaction failed (time constraint should prevent taking)
        assert!(result.is_ok(), "Take instruction should fail when time has not passed");
    }

}