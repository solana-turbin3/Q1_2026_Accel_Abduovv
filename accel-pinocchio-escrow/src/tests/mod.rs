#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::str::FromStr;

    use litesvm::LiteSVM;
    use litesvm_token::{spl_token, CreateAssociatedTokenAccount, CreateMint, MintTo};

    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    const PROGRAM_ID: &str = "4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    struct SetupData {
        svm: LiteSVM,
        payer: Keypair,
        taker: Keypair,
        maker: Keypair,
        mint_a: Pubkey,
        mint_b: Pubkey,
        escrow_account: Pubkey,
        escrow_vault: Pubkey,
        taker_ata_a: Pubkey,
        taker_ata_b: Pubkey,
        maker_ata_b: Pubkey,
        maker_ata_a: Pubkey,
        amount_to_give: u64,
        amount_to_receive: u64,
        bump: u8,
        system_program: Pubkey,
        token_program: Pubkey,
        associated_token_program: Pubkey,
    }

    fn program_id() -> Pubkey {
        Pubkey::from_str(PROGRAM_ID).unwrap()
    }

    fn setup() -> SetupData {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        let taker = Keypair::new();
        let maker = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");
        svm.airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");
        svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Load program SO file
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/deploy/escrow.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
        let program_id = program_id();
        svm.add_program(program_id, &program_data)
            .expect("Failed to add program");

        // Create mints
        let mint_a = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        let mint_b = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();

        // Create associated token accounts
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_b)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();

        // Derive escrow PDA
        let (escrow_account, bump) = Pubkey::find_program_address(
            &[b"escrow".as_ref(), maker.pubkey().as_ref()],
            &program_id,
        );

        // Derive vault ATA
        let escrow_vault = spl_associated_token_account::get_associated_token_address(
            &escrow_account,
            &mint_a,
        );

        // Define program IDs
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        // Mint tokens to maker's ATA A
        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1_000_000_000)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100_000_000; // 100 tokens with 6 decimals
        let amount_to_give: u64 = 500_000_000;    // 500 tokens with 6 decimals

        SetupData {
            svm,
            payer,
            taker,
            maker,
            mint_a,
            mint_b,
            escrow_account,
            escrow_vault,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            maker_ata_a,
            amount_to_give,
            amount_to_receive,
            bump,
            system_program,
            token_program,
            associated_token_program,
        }
    }

    #[test]
    fn test_make_instruction() {
        let mut setup = setup();

        let program_id = program_id();
        assert_eq!(program_id.to_string(), PROGRAM_ID);

        // Create the "Make" instruction
        let make_data = [
            vec![0u8], // Discriminator for "Make" instruction
            setup.bump.to_le_bytes().to_vec(),
            setup.amount_to_receive.to_le_bytes().to_vec(),
            setup.amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();

        let make_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(setup.maker.pubkey(), true),
                AccountMeta::new(setup.mint_a, false),
                AccountMeta::new(setup.mint_b, false),
                AccountMeta::new(setup.escrow_account, false),
                AccountMeta::new(setup.maker_ata_a, false),
                AccountMeta::new(setup.escrow_vault, false),
                AccountMeta::new(setup.system_program, false),
                AccountMeta::new(setup.token_program, false),
                AccountMeta::new(setup.associated_token_program, false),
            ],
            data: make_data,
        };

        let message = Message::new(&[make_ix], Some(&setup.payer.pubkey()));
        let recent_blockhash = setup.svm.latest_blockhash();
        let transaction = Transaction::new(&[&setup.payer, &setup.maker], message, recent_blockhash);

        let tx = setup.svm.send_transaction(transaction).unwrap();

        println!("\nMake transaction successful");
        println!("CUs Consumed: {}", tx.compute_units_consumed);
    }

    #[test]
    fn test_make_v2_instruction() {
        let mut setup = setup();

        let program_id = program_id();

        // Create the "MakeV2" instruction (discriminator = 3)
        let make_v2_data = [
            vec![3u8], // Discriminator for "MakeV2" instruction
            setup.bump.to_le_bytes().to_vec(),
            setup.amount_to_receive.to_le_bytes().to_vec(),
            setup.amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();

        let make_v2_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(setup.maker.pubkey(), true),
                AccountMeta::new(setup.mint_a, false),
                AccountMeta::new(setup.mint_b, false),
                AccountMeta::new(setup.escrow_account, false),
                AccountMeta::new(setup.maker_ata_a, false),
                AccountMeta::new(setup.escrow_vault, false),
                AccountMeta::new(setup.system_program, false),
                AccountMeta::new(setup.token_program, false),
                AccountMeta::new(setup.associated_token_program, false),
            ],
            data: make_v2_data,
        };

        let message = Message::new(&[make_v2_ix], Some(&setup.payer.pubkey()));
        let recent_blockhash = setup.svm.latest_blockhash();
        let transaction = Transaction::new(&[&setup.payer, &setup.maker], message, recent_blockhash);

        let tx = setup.svm.send_transaction(transaction).unwrap();

        println!("\nMakeV2 transaction successful");
        println!("CUs Consumed: {}", tx.compute_units_consumed);
    }

    #[test]
    fn test_take_instruction() {
        let mut setup = setup();

        let program_id = program_id();

        // First, create the escrow using Make instruction
        let make_data = [
            vec![0u8],
            setup.bump.to_le_bytes().to_vec(),
            setup.amount_to_receive.to_le_bytes().to_vec(),
            setup.amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();

        let make_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(setup.maker.pubkey(), true),
                AccountMeta::new(setup.mint_a, false),
                AccountMeta::new(setup.mint_b, false),
                AccountMeta::new(setup.escrow_account, false),
                AccountMeta::new(setup.maker_ata_a, false),
                AccountMeta::new(setup.escrow_vault, false),
                AccountMeta::new(setup.system_program, false),
                AccountMeta::new(setup.token_program, false),
                AccountMeta::new(setup.associated_token_program, false),
            ],
            data: make_data,
        };

        let message = Message::new(&[make_ix], Some(&setup.payer.pubkey()));
        let recent_blockhash = setup.svm.latest_blockhash();
        let transaction = Transaction::new(&[&setup.payer, &setup.maker], message, recent_blockhash);
        setup.svm.send_transaction(transaction).unwrap();

        // Mint tokens to taker's ATA B (for the exchange)
        MintTo::new(&mut setup.svm, &setup.payer, &setup.mint_b, &setup.taker_ata_b, setup.amount_to_give)
            .send()
            .unwrap();

        // Now execute the Take instruction (discriminator = 1)
        let take_data = vec![1u8]; // Discriminator for "Take" instruction

        let take_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(setup.taker.pubkey(), true),
                AccountMeta::new(setup.maker.pubkey(), false),
                AccountMeta::new(setup.mint_a, false),
                AccountMeta::new(setup.mint_b, false),
                AccountMeta::new(setup.escrow_account, false),
                AccountMeta::new(setup.taker_ata_a, false),
                AccountMeta::new(setup.taker_ata_b, false),
                AccountMeta::new(setup.maker_ata_b, false),
                AccountMeta::new(setup.escrow_vault, false),
                AccountMeta::new_readonly(setup.token_program, false),
                AccountMeta::new(setup.associated_token_program, false),
            ],
            data: take_data,
        };

        let message = Message::new(&[take_ix], Some(&setup.taker.pubkey()));
        let recent_blockhash = setup.svm.latest_blockhash();
        let transaction = Transaction::new(&[&setup.taker], message, recent_blockhash);

        let tx = setup.svm.send_transaction(transaction).unwrap();

        println!("\nTake transaction successful");
        println!("CUs Consumed: {}", tx.compute_units_consumed);
    }

    #[test]
    fn test_cancel_instruction() {
        let mut setup = setup();

        let program_id = program_id();

        // First, create the escrow using Make instruction
        let make_data = [
            vec![0u8],
            setup.bump.to_le_bytes().to_vec(),
            setup.amount_to_receive.to_le_bytes().to_vec(),
            setup.amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();

        let make_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(setup.maker.pubkey(), true),
                AccountMeta::new(setup.mint_a, false),
                AccountMeta::new(setup.mint_b, false),
                AccountMeta::new(setup.escrow_account, false),
                AccountMeta::new(setup.maker_ata_a, false),
                AccountMeta::new(setup.escrow_vault, false),
                AccountMeta::new(setup.system_program, false),
                AccountMeta::new(setup.token_program, false),
                AccountMeta::new(setup.associated_token_program, false),
            ],
            data: make_data,
        };

        let message = Message::new(&[make_ix], Some(&setup.payer.pubkey()));
        let recent_blockhash = setup.svm.latest_blockhash();
        let transaction = Transaction::new(&[&setup.payer, &setup.maker], message, recent_blockhash);
        setup.svm.send_transaction(transaction).unwrap();

        // Now execute the Cancel instruction (discriminator = 2)
        let cancel_data = vec![2u8]; // Discriminator for "Cancel" instruction

        let cancel_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(setup.maker.pubkey(), true),
                AccountMeta::new_readonly(setup.escrow_account, false),
                AccountMeta::new(setup.escrow_vault, false),
                AccountMeta::new(setup.maker_ata_a, false),
                AccountMeta::new_readonly(setup.system_program, false),
                AccountMeta::new_readonly(setup.token_program, false),
                AccountMeta::new_readonly(setup.associated_token_program, false),
            ],
            data: cancel_data,
        };

        let message = Message::new(&[cancel_ix], Some(&setup.maker.pubkey()));
        let recent_blockhash = setup.svm.latest_blockhash();
        let transaction = Transaction::new(&[&setup.maker], message, recent_blockhash);

        let tx = setup.svm.send_transaction(transaction).unwrap();

        println!("\nCancel transaction successful");
        println!("CUs Consumed: {}", tx.compute_units_consumed);
    }
}
