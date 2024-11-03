use example_token_management::instruction::ProgramInstruction;
use solana_banks_interface::TransactionMetadata;
use solana_program_test::{tokio, BanksClient, ProgramTest};
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
};
use spl_token_2022::state::Mint;

#[tokio::test]
async fn test_init_mint_token_program() {
    let decimals = 9;
    let mint_authority = Pubkey::new_unique();

    let tx_meta = init_mint_for_test(
        set_up(
            spl_token::ID,
            decimals,
            mint_authority,
            None, // freeze_authority
        )
        .await,
    )
    .await;
    assert!(!program_failed(&tx_meta.log_messages));
    // NOTE: Mint bump is 252, which requires 3 iterations to find a mint key. Each iteration costs
    // 1,500 CU. So, the total cost is 4,500 CU for this PDA.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 3 * 1_500;
    assert_eq!(adjusted_compute_units_consumed, 8_266);
}

#[tokio::test]
async fn test_init_mint_token_2022_program() {
    let decimals = 9;
    let mint_authority = Pubkey::new_unique();

    let tx_meta = init_mint_for_test(
        set_up(
            spl_token_2022::ID,
            decimals,
            mint_authority,
            None, // freeze_authority
        )
        .await,
    )
    .await;
    assert!(!program_failed(&tx_meta.log_messages));
    // NOTE: Mint bump is 252, which requires 3 iterations to find a mint key. Each iteration costs
    // 1,500 CU. So, the total cost is 4,500 CU for this PDA.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 3 * 1_500;
    assert_eq!(adjusted_compute_units_consumed, 8_636);
}

#[tokio::test]
async fn test_init_mint_token_program_and_freeze_authority() {
    let decimals = 9;
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let tx_meta = init_mint_for_test(
        set_up(
            spl_token::ID,
            decimals,
            mint_authority,
            freeze_authority.into(),
        )
        .await,
    )
    .await;
    assert!(!program_failed(&tx_meta.log_messages));
    // NOTE: Mint bump is 252, which requires 3 iterations to find a mint key. Each iteration costs
    // 1,500 CU. So, the total cost is 4,500 CU for this PDA.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 3 * 1_500;
    assert_eq!(adjusted_compute_units_consumed, 8_582);
}

#[tokio::test]
async fn test_init_mint_token_2022_program_and_freeze_authority() {
    let decimals = 9;
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let tx_meta = init_mint_for_test(
        set_up(
            spl_token_2022::ID,
            decimals,
            mint_authority,
            freeze_authority.into(),
        )
        .await,
    )
    .await;
    assert!(!program_failed(&tx_meta.log_messages));
    // NOTE: Mint bump is 252, which requires 3 iterations to find a mint key. Each iteration costs
    // 1,500 CU. So, the total cost is 4,500 CU for this PDA.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 3 * 1_500;
    assert_eq!(adjusted_compute_units_consumed, 8_938);
}

async fn set_up(
    token_program_id: Pubkey,
    decimals: u8,
    mint_authority: Pubkey,
    freeze_authority: Option<Pubkey>,
) -> InitMintTest {
    let (banks_client, payer, recent_blockhash) = ProgramTest::new(
        "example_token_management",
        example_token_management::ID,
        None,
    )
    .start()
    .await;

    InitMintTest {
        banks_client,
        payer,
        recent_blockhash,
        token_program_id,
        decimals,
        mint_authority,
        freeze_authority,
    }
}

fn program_failed(log_messages: &Vec<String>) -> bool {
    log_messages
        .iter()
        .filter(|line| line.contains(&format!("Program {} failed", example_token_management::ID)))
        .peekable()
        .peek()
        .is_some()
}

pub struct InitMintTest {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub token_program_id: Pubkey,
    pub decimals: u8,
    pub mint_authority: Pubkey,
    pub freeze_authority: Option<Pubkey>,
}

pub async fn init_mint_for_test(
    InitMintTest {
        mut banks_client,
        payer,
        recent_blockhash,
        token_program_id,
        decimals,
        mint_authority,
        freeze_authority,
    }: InitMintTest,
) -> TransactionMetadata {
    let (new_mint_addr, mint_bump) =
        Pubkey::find_program_address(&[b"mint"], &example_token_management::ID);
    assert_eq!(mint_bump, 252);

    let instruction = Instruction {
        program_id: example_token_management::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new(new_mint_addr, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: borsh::to_vec(&ProgramInstruction::InitMint {
            decimals,
            mint_authority,
            freeze_authority,
        })
        .unwrap(),
    };
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let tx_meta = banks_client
        .process_transaction_with_metadata(transaction)
        .await
        .unwrap()
        .metadata
        .unwrap();

    // Check that mint exists.
    let mint_account = banks_client
        .get_account(new_mint_addr)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(mint_account.owner, token_program_id);

    let mint_data = Mint::unpack(&mint_account.data).unwrap();
    assert_eq!(
        mint_data,
        Mint {
            mint_authority: mint_authority.into(),
            freeze_authority: freeze_authority.into(),
            decimals,
            is_initialized: true,
            supply: 0,
        }
    );

    tx_meta
}
