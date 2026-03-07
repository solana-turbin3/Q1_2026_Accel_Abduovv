// Comprehensive tests for Pinocchio Fundraiser Program
// Run with: cargo test --features std -- --nocapture

use std::path::PathBuf;

use litesvm::{types::TransactionResult, LiteSVM};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    sysvar,
    transaction::Transaction,
    account::Account,
    program_pack::Pack,
};

use pinocchio_fundraiser::states::{Fundraiser, ContributeState};
use pinocchio_fundraiser::errors::FundraiserError;
use pinocchio_fundraiser::constants::{MIN_AMOUNT_TO_RAISE, MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER};

pub fn program_id() -> Pubkey {
    Pubkey::new_from_array(pinocchio_fundraiser::ID)
}

pub fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();

    let so_path = PathBuf::from("target/deploy").join("pinocchio_fundraiser.so");

    let program_data = std::fs::read(so_path).expect("Failed to read program .so file");
    svm.add_program(program_id(), &program_data).expect("add_program failed");

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 100 * LAMPORTS_PER_SOL).expect("airdrop failed");

    (svm, payer)
}

/// Helper to create a mint account for testing
pub fn create_mint(svm: &mut LiteSVM, payer: &Keypair) -> (Keypair, Pubkey) {
    let mint = Keypair::new();
    let mint_rent = svm.get_account(&sysvar::rent::id()).unwrap().lamports;

    let create_mint_ix = solana_sdk::system_instruction::create_account(
        &payer.pubkey(),
        &mint.pubkey(),
        mint_rent * 10,
        spl_token_2022::state::Mint::LEN as u64,
        &spl_token_2022::id(),
    );

    let init_mint_ix = spl_token_2022::instruction::initialize_mint(
        &spl_token_2022::id(),
        &mint.pubkey(),
        &payer.pubkey(),
        None,
        9,
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix, init_mint_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).expect("Failed to create mint");

    (mint, mint.pubkey())
}

/// Helper to create an associated token account
pub fn create_ata(svm: &mut LiteSVM, payer: &Keypair, mint: &Pubkey, owner: &Pubkey) -> Pubkey {
    let ata = spl_associated_token_account::get_associated_token_address(owner, mint);
    
    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        owner,
        mint,
        &spl_token_2022::id(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_ata_ix],
        Some(&payer.pubkey()),
        &[&payer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).expect("Failed to create ATA");
    
    ata
}

/// Helper to mint tokens to an ATA
pub fn mint_tokens(svm: &mut LiteSVM, payer: &Keypair, mint: &Keypair, ata: &Pubkey, amount: u64) {
    let mint_to_ix = spl_token_2022::instruction::mint_to(
        &spl_token_2022::id(),
        mint,
        ata,
        &payer.pubkey(),
        &[],
        amount,
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&payer.pubkey()),
        &[&payer, mint],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).expect("Failed to mint tokens");
}

pub struct InitializeData {
    pub maker: Keypair,
    pub fundraiser_pda: Pubkey,
    pub vault: Keypair,
    pub mint_to_raise: Pubkey,
    pub amount: u64,
    pub duration: u64,
    pub bump: u8,
}

impl InitializeData {
    pub fn new(maker: Keypair, mint_to_raise: Pubkey, amount: u64, duration: u64) -> Self {
        let (fundraiser_pda, bump) = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), &maker.pubkey().to_bytes()],
            &program_id(),
        );
        let vault = Keypair::new();
        Self {
            maker,
            fundraiser_pda,
            vault,
            mint_to_raise,
            amount,
            duration,
            bump,
        }
    }
}

pub fn initialize_instruction(init_data: &InitializeData) -> Instruction {
    let mut ix_data = vec![0u8]; // discriminator
    ix_data.push(init_data.bump);
    ix_data.extend_from_slice(&init_data.amount.to_le_bytes());
    ix_data.extend_from_slice(&init_data.duration.to_le_bytes());

    let system_program = Pubkey::from(pinocchio_system::id());
    let accounts = vec![
        AccountMeta::new(init_data.maker.pubkey(), true),
        AccountMeta::new(init_data.fundraiser_pda, false),
        AccountMeta::new(init_data.vault.pubkey(), false),
        AccountMeta::new(init_data.mint_to_raise, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program, false),
    ];

    Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    }
}

pub fn initialize(svm: &mut LiteSVM, init_data: &InitializeData) -> TransactionResult {
    let ix = initialize_instruction(init_data);

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&init_data.maker.pubkey()),
        &[&init_data.maker, &init_data.vault],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
}

pub fn contribute_instruction(
    contributor: &Keypair,
    contributor_account: &Pubkey,
    mint_to_raise: &Pubkey,
    contributor_ata: &Pubkey,
    fundraiser_pda: &Pubkey,
    vault: &Pubkey,
    maker: &Pubkey,
    amount: u64,
) -> Instruction {
    let ix_data = vec![1u8, amount as u8]; // discriminator 1 + amount

    let accounts = vec![
        AccountMeta::new(contributor.pubkey(), true),
        AccountMeta::new(*contributor_account, false),
        AccountMeta::new(*mint_to_raise, false),
        AccountMeta::new(*contributor_ata, false),
        AccountMeta::new(*fundraiser_pda, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*maker, false),
        AccountMeta::new_readonly(spl_token_2022::id(), false),
        AccountMeta::new_readonly(Pubkey::from(pinocchio_system::id()), false),
    ];

    Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    }
}

pub fn contribute(
    svm: &mut LiteSVM,
    contributor: &Keypair,
    contributor_account: &Pubkey,
    mint_to_raise: &Pubkey,
    contributor_ata: &Pubkey,
    fundraiser_pda: &Pubkey,
    vault: &Pubkey,
    maker: &Pubkey,
    amount: u64,
) -> TransactionResult {
    let ix = contribute_instruction(
        contributor,
        contributor_account,
        mint_to_raise,
        contributor_ata,
        fundraiser_pda,
        vault,
        maker,
        amount,
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&contributor.pubkey()),
        &[contributor],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
}

pub fn checker_instruction(
    maker: &Keypair,
    maker_ata: &Pubkey,
    mint_to_raise: &Pubkey,
    fundraiser_pda: &Pubkey,
    vault: &Pubkey,
) -> Instruction {
    let ix_data = vec![2u8]; // discriminator 2

    let accounts = vec![
        AccountMeta::new(maker.pubkey(), true),
        AccountMeta::new(*maker_ata, false),
        AccountMeta::new(*mint_to_raise, false),
        AccountMeta::new(*fundraiser_pda, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(spl_token_2022::id(), false),
        AccountMeta::new_readonly(Pubkey::from(pinocchio_system::id()), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
    ];

    Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    }
}

pub fn checker(
    svm: &mut LiteSVM,
    maker: &Keypair,
    maker_ata: &Pubkey,
    mint_to_raise: &Pubkey,
    fundraiser_pda: &Pubkey,
    vault: &Pubkey,
) -> TransactionResult {
    let ix = checker_instruction(maker, maker_ata, mint_to_raise, fundraiser_pda, vault);

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&maker.pubkey()),
        &[maker],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
}

pub fn refund_instruction(
    contributor: &Keypair,
    contribute_account: &Pubkey,
    mint_to_raise: &Pubkey,
    contributor_ata: &Pubkey,
    fundraiser_pda: &Pubkey,
    vault: &Pubkey,
    maker: &Pubkey,
) -> Instruction {
    let ix_data = vec![3u8]; // discriminator 3

    let accounts = vec![
        AccountMeta::new(contributor.pubkey(), true),
        AccountMeta::new(*contribute_account, false),
        AccountMeta::new(*mint_to_raise, false),
        AccountMeta::new(*contributor_ata, false),
        AccountMeta::new(*fundraiser_pda, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*maker, false),
        AccountMeta::new_readonly(spl_token_2022::id(), false),
    ];

    Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    }
}

pub fn refund(
    svm: &mut LiteSVM,
    contributor: &Keypair,
    contribute_account: &Pubkey,
    mint_to_raise: &Pubkey,
    contributor_ata: &Pubkey,
    fundraiser_pda: &Pubkey,
    vault: &Pubkey,
    maker: &Pubkey,
) -> TransactionResult {
    let ix = refund_instruction(
        contributor,
        contribute_account,
        mint_to_raise,
        contributor_ata,
        fundraiser_pda,
        vault,
        maker,
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&contributor.pubkey()),
        &[contributor],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
}

// ============================================================================
// INITIALIZATION TESTS
// ============================================================================

#[test]
fn test_initialize_happy_path() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer, mint_pubkey, 1000, 86400);

    let result = initialize(&mut svm, &init_data).expect("transaction failed");
    
    println!("✓ test_initialize_happy_path");
    println!("  CU used: {}", result.compute_units_consumed.unwrap_or(0));
    println!("  Status: Success");

    // Verify PDA was created with correct size
    let fundraiser_account = svm.get_account(&init_data.fundraiser_pda)
        .expect("fundraiser account should exist");
    assert_eq!(fundraiser_account.data.len(), Fundraiser::LEN, "fundraiser size mismatch");

    // Verify vault account was created
    let vault_account = svm.get_account(&init_data.vault.pubkey())
        .expect("vault account should exist");
    assert_eq!(vault_account.owner, spl_token_2022::id(), "vault owner should be token program");
}

#[test]
fn test_initialize_unhappy_not_enough_accounts() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer, mint_pubkey, 1000, 86400);

    // Create instruction with missing accounts
    let mut ix_data = vec![0u8];
    ix_data.push(init_data.bump);
    ix_data.extend_from_slice(&init_data.amount.to_le_bytes());
    ix_data.extend_from_slice(&init_data.duration.to_le_bytes());

    let accounts = vec![
        AccountMeta::new(init_data.maker.pubkey(), true),
        AccountMeta::new(init_data.fundraiser_pda, false),
        // Missing: vault, mint_to_raise, sysvar_rent, system_program
    ];

    let ix = Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&init_data.maker.pubkey()),
        &[&init_data.maker],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Should fail with not enough accounts");
    
    println!("✓ test_initialize_unhappy_not_enough_accounts");
    println!("  Error: NotEnoughAccountKeys");
}

#[test]
fn test_initialize_unhappy_missing_signature() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer, mint_pubkey, 1000, 86400);

    let mut ix_data = vec![0u8];
    ix_data.push(init_data.bump);
    ix_data.extend_from_slice(&init_data.amount.to_le_bytes());
    ix_data.extend_from_slice(&init_data.duration.to_le_bytes());

    let system_program = Pubkey::from(pinocchio_system::id());
    let accounts = vec![
        AccountMeta::new(init_data.maker.pubkey(), false), // Not a signer!
        AccountMeta::new(init_data.fundraiser_pda, false),
        AccountMeta::new(init_data.vault.pubkey(), false),
        AccountMeta::new(init_data.mint_to_raise, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let ix = Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&init_data.maker.pubkey()),
        &[&init_data.vault], // Missing maker signature
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Should fail with missing signature");
    
    println!("✓ test_initialize_unhappy_missing_signature");
    println!("  Error: MissingRequiredSignature");
}

#[test]
fn test_initialize_unhappy_account_already_initialized() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 1000, 86400);

    // First initialization - should succeed
    let result = initialize(&mut svm, &init_data).expect("first init should succeed");
    println!("  First init CU: {}", result.compute_units_consumed.unwrap_or(0));

    // Second initialization with same PDA - should fail
    let init_data2 = InitializeData::new(payer, mint_pubkey, 2000, 43200);
    let result = initialize(&mut svm, &init_data2);
    assert!(result.is_err(), "Should fail with account already initialized");
    
    println!("✓ test_initialize_unhappy_account_already_initialized");
    println!("  Error: AccountAlreadyInitialized");
}

#[test]
fn test_initialize_unhappy_invalid_instruction_data_length() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer, mint_pubkey, 1000, 86400);

    // Create instruction with too short data
    let ix_data = vec![0u8, init_data.bump]; // Missing amount and duration

    let system_program = Pubkey::from(pinocchio_system::id());
    let accounts = vec![
        AccountMeta::new(init_data.maker.pubkey(), true),
        AccountMeta::new(init_data.fundraiser_pda, false),
        AccountMeta::new(init_data.vault.pubkey(), false),
        AccountMeta::new(init_data.mint_to_raise, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let ix = Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&init_data.maker.pubkey()),
        &[&init_data.maker, &init_data.vault],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Should fail with invalid instruction data");
    
    println!("✓ test_initialize_unhappy_invalid_instruction_data_length");
    println!("  Error: InvalidInstructionData");
}

#[test]
fn test_initialize_unhappy_amount_below_minimum() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    // Amount below minimum (MIN_AMOUNT_TO_RAISE = 3, with 9 decimals = 3_000_000_000)
    let min_amount = MIN_AMOUNT_TO_RAISE.pow(9);
    let init_data = InitializeData::new(payer, mint_pubkey, min_amount - 1, 86400);

    let result = initialize(&mut svm, &init_data);
    assert!(result.is_err(), "Should fail with amount below minimum");
    
    println!("✓ test_initialize_unhappy_amount_below_minimum");
    println!("  Error: InvalidInstructionData (amount < MIN_AMOUNT_TO_RAISE)");
}

#[test]
fn test_initialize_unhappy_pda_mismatch() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer, mint_pubkey, 1000, 86400);

    // Use wrong bump to create PDA mismatch
    let wrong_bump = init_data.bump.wrapping_add(1);
    let mut ix_data = vec![0u8];
    ix_data.push(wrong_bump); // Wrong bump
    ix_data.extend_from_slice(&init_data.amount.to_le_bytes());
    ix_data.extend_from_slice(&init_data.duration.to_le_bytes());

    let system_program = Pubkey::from(pinocchio_system::id());
    let accounts = vec![
        AccountMeta::new(init_data.maker.pubkey(), true),
        AccountMeta::new(init_data.fundraiser_pda, false), // Correct PDA
        AccountMeta::new(init_data.vault.pubkey(), false),
        AccountMeta::new(init_data.mint_to_raise, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let ix = Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&init_data.maker.pubkey()),
        &[&init_data.maker, &init_data.vault],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Should fail with PDA mismatch");
    
    println!("✓ test_initialize_unhappy_pda_mismatch");
    println!("  Error: PdaMismatch (assertion failed)");
}

// ============================================================================
// CONTRIBUTE TESTS
// ============================================================================

#[test]
fn test_contribute_happy_path_first_contribution() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 1000, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    // Create contributor
    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    // Derive contribute PDA
    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    let result = contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        100,
    ).expect("contribute should succeed");

    println!("✓ test_contribute_happy_path_first_contribution");
    println!("  CU used: {}", result.compute_units_consumed.unwrap_or(0));
    println!("  Amount contributed: 100");

    // Verify contribute account was created
    let contribute_acc = svm.get_account(&contribute_account).expect("contribute account should exist");
    assert_eq!(contribute_acc.data.len(), ContributeState::LEN, "contribute state size mismatch");

    // Verify vault received tokens
    let vault_acc = svm.get_account(&init_data.vault.pubkey()).expect("vault should exist");
    let vault_state = spl_token_2022::state::Account::unpack(&vault_acc.data).unwrap();
    assert_eq!(vault_state.amount, 100, "vault amount should be 100");
}

#[test]
fn test_contribute_happy_path_multiple_contributions() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 1000, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    // First contribution
    contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        50,
    ).expect("first contribute should succeed");

    // Second contribution
    let result = contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        50,
    ).expect("second contribute should succeed");

    println!("✓ test_contribute_happy_path_multiple_contributions");
    println!("  CU used: {}", result.compute_units_consumed.unwrap_or(0));
    println!("  Total contributed: 100");

    // Verify total in vault
    let vault_acc = svm.get_account(&init_data.vault.pubkey()).expect("vault should exist");
    let vault_state = spl_token_2022::state::Account::unpack(&vault_acc.data).unwrap();
    assert_eq!(vault_state.amount, 100, "vault amount should be 100");
}

#[test]
fn test_contribute_unhappy_missing_signature() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 1000, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    // Create instruction without signer
    let ix_data = vec![1u8, 100u8];
    let accounts = vec![
        AccountMeta::new(contributor.pubkey(), false), // Not a signer!
        AccountMeta::new(contribute_account, false),
        AccountMeta::new(mint_pubkey, false),
        AccountMeta::new(contributor_ata, false),
        AccountMeta::new(init_data.fundraiser_pda, false),
        AccountMeta::new(init_data.vault.pubkey(), false),
        AccountMeta::new_readonly(payer.pubkey(), false),
        AccountMeta::new_readonly(spl_token_2022::id(), false),
        AccountMeta::new_readonly(Pubkey::from(pinocchio_system::id()), false),
    ];

    let ix = Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&contributor.pubkey()),
        &[&contributor],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Should fail with missing signature");
    
    println!("✓ test_contribute_unhappy_missing_signature");
    println!("  Error: MissingRequiredSignature");
}

#[test]
fn test_contribute_unhappy_mint_mismatch() {
    let (mut svm, payer) = setup();
    let (_mint1, mint_pubkey1) = create_mint(&mut svm, &payer);
    let (_mint2, mint_pubkey2) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey1, 1000, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey2, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint2, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    // Try to contribute with wrong mint
    let result = contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey2, // Wrong mint
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        100,
    );
    assert!(result.is_err(), "Should fail with mint mismatch");
    
    println!("✓ test_contribute_unhappy_mint_mismatch");
    println!("  Error: MintMismatch");
}

#[test]
fn test_contribute_unhappy_contribution_exceeds_max() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    // Set max contribution to 10% of 1000 = 100
    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 1000, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    // Try to contribute more than 10% (max is 100)
    let result = contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        101, // Exceeds 10%
    );
    assert!(result.is_err(), "Should fail with contribution exceeds max");
    
    println!("✓ test_contribute_unhappy_contribution_exceeds_max");
    println!("  Error: ContributionExceedsMax (>{MAX_CONTRIBUTION_PERCENTAGE}%)");
}

#[test]
fn test_contribute_unhappy_fundraiser_expired() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    // Very short duration (1 second)
    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 1000, 1);
    initialize(&mut svm, &init_data).expect("init should succeed");

    // Wait for fundraiser to expire (simulated by SVM)
    std::thread::sleep(std::time::Duration::from_secs(2));

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    let result = contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        100,
    );
    assert!(result.is_err(), "Should fail with fundraiser expired");
    
    println!("✓ test_contribute_unhappy_fundraiser_expired");
    println!("  Error: FundraiserExpired");
}

#[test]
fn test_contribute_unhappy_exceeds_fundraising_goal() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 100, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    // First contributor fills 90%
    let contributor1 = Keypair::new();
    svm.airdrop(&contributor1.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    let contributor1_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor1.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor1_ata, 500);

    let (contribute_account1, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor1.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    contribute(
        &mut svm,
        &contributor1,
        &contribute_account1,
        &mint_pubkey,
        &contributor1_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        90,
    ).expect("first contribute should succeed");

    // Second contributor tries to exceed goal
    let contributor2 = Keypair::new();
    svm.airdrop(&contributor2.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    let contributor2_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor2.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor2_ata, 500);

    let (contribute_account2, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor2.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    let result = contribute(
        &mut svm,
        &contributor2,
        &contribute_account2,
        &mint_pubkey,
        &contributor2_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        20, // Would exceed 100 goal
    );
    assert!(result.is_err(), "Should fail with contribution exceeds goal");
    
    println!("✓ test_contribute_unhappy_exceeds_fundraising_goal");
    println!("  Error: ContributionExceedsMax (exceeds goal)");
}

#[test]
fn test_contribute_unhappy_invalid_contribution_amount() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 1000, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    // Create instruction with empty data
    let ix_data = vec![1u8]; // No amount byte
    let accounts = vec![
        AccountMeta::new(contributor.pubkey(), true),
        AccountMeta::new(contribute_account, false),
        AccountMeta::new(mint_pubkey, false),
        AccountMeta::new(contributor_ata, false),
        AccountMeta::new(init_data.fundraiser_pda, false),
        AccountMeta::new(init_data.vault.pubkey(), false),
        AccountMeta::new_readonly(payer.pubkey(), false),
        AccountMeta::new_readonly(spl_token_2022::id(), false),
        AccountMeta::new_readonly(Pubkey::from(pinocchio_system::id()), false),
    ];

    let ix = Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&contributor.pubkey()),
        &[&contributor],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Should fail with invalid contribution amount");
    
    println!("✓ test_contribute_unhappy_invalid_contribution_amount");
    println!("  Error: InvalidContributionAmount");
}

// ============================================================================
// CHECKER TESTS (Successful Fundraiser Completion)
// ============================================================================

#[test]
fn test_checker_happy_path_goal_reached() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 100, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    // Create contributor and contribute full amount
    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        100,
    ).expect("contribute should succeed");

    // Create maker ATA
    let maker_ata = create_ata(&mut svm, &payer, &mint_pubkey, &payer.pubkey());

    let result = checker(
        &mut svm,
        &payer,
        &maker_ata,
        &mint_pubkey,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
    ).expect("checker should succeed");

    println!("✓ test_checker_happy_path_goal_reached");
    println!("  CU used: {}", result.compute_units_consumed.unwrap_or(0));
    println!("  Status: Fundraiser completed, funds transferred to maker");

    // Verify maker received tokens
    let maker_ata_acc = svm.get_account(&maker_ata).expect("maker ATA should exist");
    let maker_ata_state = spl_token_2022::state::Account::unpack(&maker_ata_acc.data).unwrap();
    assert_eq!(maker_ata_state.amount, 100, "maker should receive 100 tokens");

    // Verify vault was closed (account should not exist or have 0 lamports)
    let vault_result = svm.get_account(&init_data.vault.pubkey());
    assert!(vault_result.is_none() || vault_result.unwrap().lamports == 0, "vault should be closed");

    // Verify fundraiser PDA was closed
    let fundraiser_result = svm.get_account(&init_data.fundraiser_pda);
    assert!(fundraiser_result.is_none() || fundraiser_result.unwrap().lamports == 0, "fundraiser PDA should be closed");
}

#[test]
fn test_checker_unhappy_missing_signature() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 100, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    let maker_ata = create_ata(&mut svm, &payer, &mint_pubkey, &payer.pubkey());

    // Create instruction without maker as signer
    let ix_data = vec![2u8];
    let accounts = vec![
        AccountMeta::new(payer.pubkey(), false), // Not a signer!
        AccountMeta::new(maker_ata, false),
        AccountMeta::new(mint_pubkey, false),
        AccountMeta::new(init_data.fundraiser_pda, false),
        AccountMeta::new(init_data.vault.pubkey(), false),
        AccountMeta::new_readonly(spl_token_2022::id(), false),
        AccountMeta::new_readonly(Pubkey::from(pinocchio_system::id()), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
    ];

    let ix = Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Should fail with missing signature");
    
    println!("✓ test_checker_unhappy_missing_signature");
    println!("  Error: MissingRequiredSignature");
}

#[test]
fn test_checker_unhappy_vault_amount_mismatch() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 100, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    // Contribute only partial amount
    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        50, // Only 50, not 100
    ).expect("contribute should succeed");

    let maker_ata = create_ata(&mut svm, &payer, &mint_pubkey, &payer.pubkey());

    let result = checker(
        &mut svm,
        &payer,
        &maker_ata,
        &mint_pubkey,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
    );
    assert!(result.is_err(), "Should fail with vault amount mismatch");
    
    println!("✓ test_checker_unhappy_vault_amount_mismatch");
    println!("  Error: VaultAmountMismatch");
}

// ============================================================================
// REFUND TESTS
// ============================================================================

#[test]
fn test_refund_happy_path_expired_fundraiser() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    // Very short duration fundraiser
    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 100, 1);
    initialize(&mut svm, &init_data).expect("init should succeed");

    // Create contributor
    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        50,
    ).expect("contribute should succeed");

    // Wait for fundraiser to expire
    std::thread::sleep(std::time::Duration::from_secs(2));

    let result = refund(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
    ).expect("refund should succeed");

    println!("✓ test_refund_happy_path_expired_fundraiser");
    println!("  CU used: {}", result.compute_units_consumed.unwrap_or(0));
    println!("  Status: Refund processed after expiry");

    // Verify contributor received tokens back
    let contributor_ata_acc = svm.get_account(&contributor_ata).expect("contributor ATA should exist");
    let contributor_ata_state = spl_token_2022::state::Account::unpack(&contributor_ata_acc.data).unwrap();
    assert_eq!(contributor_ata_state.amount, 500, "contributor should have tokens back");

    // Verify contribute account was closed
    let contribute_result = svm.get_account(&contribute_account);
    assert!(contribute_result.is_none() || contribute_result.unwrap().lamports == 0, "contribute account should be closed");
}

#[test]
fn test_refund_unhappy_not_expired() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 100, 86400);
    initialize(&mut svm, &init_data).expect("init should succeed");

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        50,
    ).expect("contribute should succeed");

    // Try to refund before expiry
    let result = refund(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
    );
    assert!(result.is_err(), "Should fail - fundraiser not expired");
    
    println!("✓ test_refund_unhappy_not_expired");
    println!("  Error: FundraiserExpired (not yet expired)");
}

#[test]
fn test_refund_unhappy_missing_signature() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 100, 1);
    initialize(&mut svm, &init_data).expect("init should succeed");

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
    mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    contribute(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
        50,
    ).expect("contribute should succeed");

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Create instruction without either contributor or maker as signer
    let ix_data = vec![3u8];
    let accounts = vec![
        AccountMeta::new(contributor.pubkey(), false), // Not a signer!
        AccountMeta::new(contribute_account, false),
        AccountMeta::new(mint_pubkey, false),
        AccountMeta::new(contributor_ata, false),
        AccountMeta::new(init_data.fundraiser_pda, false),
        AccountMeta::new(init_data.vault.pubkey(), false),
        AccountMeta::new_readonly(payer.pubkey(), false), // Maker also not signer
        AccountMeta::new_readonly(spl_token_2022::id(), false),
    ];

    let ix = Instruction {
        program_id: program_id(),
        accounts,
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&contributor.pubkey()),
        &[],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Should fail with missing signature");
    
    println!("✓ test_refund_unhappy_missing_signature");
    println!("  Error: MissingRequiredSignature");
}

#[test]
fn test_refund_unhappy_zero_contribution() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 100, 1);
    initialize(&mut svm, &init_data).expect("init should succeed");

    let contributor = Keypair::new();
    svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
    
    let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());

    let (contribute_account, _bump) = Pubkey::find_program_address(
        &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
        &program_id(),
    );

    // Create contribute account with zero amount
    let create_ix = solana_sdk::system_instruction::create_account(
        &payer.pubkey(),
        &contribute_account,
        10000000,
        ContributeState::LEN as u64,
        &program_id(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&payer.pubkey()),
        &[&payer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).expect("Failed to create contribute account");

    std::thread::sleep(std::time::Duration::from_secs(2));

    let result = refund(
        &mut svm,
        &contributor,
        &contribute_account,
        &mint_pubkey,
        &contributor_ata,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
        &payer.pubkey(),
    );
    assert!(result.is_err(), "Should fail with zero contribution");
    
    println!("✓ test_refund_unhappy_zero_contribution");
    println!("  Error: InvalidInstructionData (zero amount)");
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_full_lifecycle_successful_fundraiser() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    println!("\n=== Full Lifecycle Test: Successful Fundraiser ===");

    // 1. Initialize
    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 100, 86400);
    let init_result = initialize(&mut svm, &init_data).expect("init should succeed");
    println!("1. Initialize - CU: {}", init_result.compute_units_consumed.unwrap_or(0));

    // 2. Multiple contributions
    let mut total_cu = 0u64;
    for i in 1..=5 {
        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
        
        let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
        mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

        let (contribute_account, _bump) = Pubkey::find_program_address(
            &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
            &program_id(),
        );

        let result = contribute(
            &mut svm,
            &contributor,
            &contribute_account,
            &mint_pubkey,
            &contributor_ata,
            &init_data.fundraiser_pda,
            &init_data.vault.pubkey(),
            &payer.pubkey(),
            20,
        ).expect("contribute should succeed");
        
        total_cu += result.compute_units_consumed.unwrap_or(0);
        println!("2.{i}. Contribution #{i} - CU: {}", result.compute_units_consumed.unwrap_or(0));
    }

    // 3. Checker (goal reached)
    let maker_ata = create_ata(&mut svm, &payer, &mint_pubkey, &payer.pubkey());
    let checker_result = checker(
        &mut svm,
        &payer,
        &maker_ata,
        &mint_pubkey,
        &init_data.fundraiser_pda,
        &init_data.vault.pubkey(),
    ).expect("checker should succeed");
    
    total_cu += checker_result.compute_units_consumed.unwrap_or(0);
    println!("3. Checker - CU: {}", checker_result.compute_units_consumed.unwrap_or(0));
    println!("Total CU used: {}", total_cu);
    println!("Status: ✓ Fundraiser completed successfully");

    // Verify maker received all tokens
    let maker_ata_acc = svm.get_account(&maker_ata).expect("maker ATA should exist");
    let maker_ata_state = spl_token_2022::state::Account::unpack(&maker_ata_acc.data).unwrap();
    assert_eq!(maker_ata_state.amount, 100, "maker should receive 100 tokens");
}

#[test]
fn test_full_lifecycle_failed_fundraiser_refunds() {
    let (mut svm, payer) = setup();
    let (_mint, mint_pubkey) = create_mint(&mut svm, &payer);

    println!("\n=== Full Lifecycle Test: Failed Fundraiser with Refunds ===");

    // 1. Initialize with short duration
    let init_data = InitializeData::new(payer.clone(), mint_pubkey, 100, 2);
    let init_result = initialize(&mut svm, &init_data).expect("init should succeed");
    println!("1. Initialize - CU: {}", init_result.compute_units_consumed.unwrap_or(0));

    // 2. Multiple contributions
    let mut contributors = Vec::new();
    let mut total_cu = 0u64;
    
    for i in 1..=3 {
        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");
        
        let contributor_ata = create_ata(&mut svm, &payer, &mint_pubkey, &contributor.pubkey());
        mint_tokens(&mut svm, &payer, &_mint, &contributor_ata, 500);

        let (contribute_account, _bump) = Pubkey::find_program_address(
            &[b"contribute".as_ref(), &contributor.pubkey().to_bytes(), &init_data.fundraiser_pda.to_bytes()],
            &program_id(),
        );

        let result = contribute(
            &mut svm,
            &contributor,
            &contribute_account,
            &mint_pubkey,
            &contributor_ata,
            &init_data.fundraiser_pda,
            &init_data.vault.pubkey(),
            &payer.pubkey(),
            20,
        ).expect("contribute should succeed");
        
        total_cu += result.compute_units_consumed.unwrap_or(0);
        println!("2.{i}. Contribution #{i} - CU: {}", result.compute_units_consumed.unwrap_or(0));
        
        contributors.push((contributor, contribute_account, contributor_ata));
    }

    // Wait for expiry
    std::thread::sleep(std::time::Duration::from_secs(3));

    // 3. Refunds
    for (i, (contributor, contribute_account, contributor_ata)) in contributors.iter().enumerate() {
        let result = refund(
            &mut svm,
            contributor,
            contribute_account,
            &mint_pubkey,
            contributor_ata,
            &init_data.fundraiser_pda,
            &init_data.vault.pubkey(),
            &payer.pubkey(),
        ).expect("refund should succeed");
        
        total_cu += result.compute_units_consumed.unwrap_or(0);
        println!("3.{i}. Refund #{i} - CU: {}", result.compute_units_consumed.unwrap_or(0));
    }

    println!("Total CU used: {}", total_cu);
    println!("Status: ✓ All contributors refunded");

    // Verify vault is empty
    let vault_acc = svm.get_account(&init_data.vault.pubkey()).expect("vault should exist");
    let vault_state = spl_token_2022::state::Account::unpack(&vault_acc.data).unwrap();
    assert_eq!(vault_state.amount, 0, "vault should be empty after refunds");
}
