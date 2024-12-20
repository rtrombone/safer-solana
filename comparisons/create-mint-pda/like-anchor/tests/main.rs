use create_mint_pda_common::{InitMintTest, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::pubkey::Pubkey;

#[tokio::test]
async fn test_init_mint_token_program() {
    let decimals = 9;
    let mint_authority = Pubkey::new_unique();

    let tx_meta = create_mint_pda_common::init_mint_for_test(
        set_up(
            TOKEN_PROGRAM_ID,
            decimals,
            mint_authority,
            None, // freeze_authority
        )
        .await,
    )
    .await;
    assert!(did_not_fail(&tx_meta.log_messages));
    assert_eq!(tx_meta.compute_units_consumed, 10_903);
}

#[tokio::test]
async fn test_init_mint_token_2022_program() {
    let decimals = 9;
    let mint_authority = Pubkey::new_unique();

    let tx_meta = create_mint_pda_common::init_mint_for_test(
        set_up(
            TOKEN_2022_PROGRAM_ID,
            decimals,
            mint_authority,
            None, // freeze_authority
        )
        .await,
    )
    .await;
    assert!(did_not_fail(&tx_meta.log_messages));
    assert_eq!(tx_meta.compute_units_consumed, 11_267);
}

#[tokio::test]
async fn test_init_mint_token_program_and_freeze_authority() {
    let decimals = 9;
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let tx_meta = create_mint_pda_common::init_mint_for_test(
        set_up(
            TOKEN_PROGRAM_ID,
            decimals,
            mint_authority,
            freeze_authority.into(),
        )
        .await,
    )
    .await;
    assert!(did_not_fail(&tx_meta.log_messages));
    assert_eq!(tx_meta.compute_units_consumed, 11_209);
}

#[tokio::test]
async fn test_init_mint_token_2022_program_and_freeze_authority() {
    let decimals = 9;
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let tx_meta = create_mint_pda_common::init_mint_for_test(
        set_up(
            TOKEN_2022_PROGRAM_ID,
            decimals,
            mint_authority,
            freeze_authority.into(),
        )
        .await,
    )
    .await;
    assert!(did_not_fail(&tx_meta.log_messages));
    assert_eq!(tx_meta.compute_units_consumed, 11_559);
}

async fn set_up(
    token_program_id: Pubkey,
    decimals: u8,
    mint_authority: Pubkey,
    freeze_authority: Option<Pubkey>,
) -> InitMintTest {
    let (banks_client, payer, recent_blockhash) = ProgramTest::new(
        "create_mint_pda_like_anchor",
        create_mint_pda_like_anchor::ID,
        processor!(create_mint_pda_like_anchor::process_instruction),
    )
    .start()
    .await;

    InitMintTest {
        banks_client,
        payer,
        recent_blockhash,
        program_id: create_mint_pda_like_anchor::ID,
        token_program_id,
        decimals,
        mint_authority,
        freeze_authority,
    }
}

fn did_not_fail(log_messages: &Vec<String>) -> bool {
    log_messages
        .iter()
        .filter(|line| {
            line.contains(&format!(
                "Program {} failed",
                create_mint_pda_like_anchor::ID
            ))
        })
        .peekable()
        .peek()
        .is_none()
}
