use create_mint_pda_common::{InitMintTest, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID};
use solana_program_test::{tokio, ProgramTest};
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
    assert_eq!(tx_meta.compute_units_consumed, 14_824);
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
    assert_eq!(tx_meta.compute_units_consumed, 15_170);
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
    assert_eq!(tx_meta.compute_units_consumed, 15_439);
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
    assert_eq!(tx_meta.compute_units_consumed, 15_785);
}

async fn set_up(
    token_program_id: Pubkey,
    decimals: u8,
    mint_authority: Pubkey,
    freeze_authority: Option<Pubkey>,
) -> InitMintTest {
    let (banks_client, payer, recent_blockhash) = ProgramTest::new(
        "create_mint_pda_using_anchor",
        create_mint_pda_using_anchor::ID,
        anchor_processor!(create_mint_pda_using_anchor),
    )
    .start()
    .await;

    InitMintTest {
        banks_client,
        payer,
        recent_blockhash,
        program_id: create_mint_pda_using_anchor::ID,
        token_program_id,
        decimals,
        mint_authority,
        freeze_authority,
    }
}

/// Borrowed from https://github.com/coral-xyz/anchor/issues/2738#issuecomment-2230683481.
#[macro_export]
macro_rules! anchor_processor {
    ($program:ident) => {{
        fn entry(
            program_id: &::anchor_lang::solana_program::pubkey::Pubkey,
            accounts: &[::anchor_lang::solana_program::account_info::AccountInfo],
            instruction_data: &[u8],
        ) -> ::anchor_lang::solana_program::entrypoint::ProgramResult {
            let accounts = Box::leak(Box::new(accounts.to_vec()));

            $program::entry(program_id, accounts, instruction_data)
        }

        solana_program_test::processor!(entry)
    }};
}

fn did_not_fail(log_messages: &Vec<String>) -> bool {
    log_messages
        .iter()
        .filter(|line| {
            line.contains(&format!(
                "Program {} failed",
                create_mint_pda_using_anchor::ID
            ))
        })
        .peekable()
        .peek()
        .is_none()
}
