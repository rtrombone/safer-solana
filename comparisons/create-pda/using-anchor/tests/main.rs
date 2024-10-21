use create_pda_common::InitThingTest;
use solana_program_test::{tokio, ProgramTest};

#[tokio::test]
async fn test_init_thing() {
    let tx_meta = create_pda_common::init_thing_for_test(set_up().await).await;
    assert!(did_not_fail(&tx_meta.log_messages));
    assert_eq!(tx_meta.compute_units_consumed, 11_442);
}

#[tokio::test]
async fn test_init_thing_already_having_lamports() {
    let tx_meta =
        create_pda_common::init_thing_already_having_lamports_for_test(set_up().await).await;
    assert!(did_not_fail(&tx_meta.log_messages));

    // This includes lamports transfer.
    assert_eq!(tx_meta.compute_units_consumed, 15_516);
}

async fn set_up() -> InitThingTest {
    let (banks_client, payer, recent_blockhash) = ProgramTest::new(
        "create_pda_using_anchor",
        create_pda_using_anchor::ID,
        anchor_processor!(create_pda_using_anchor),
    )
    .start()
    .await;

    InitThingTest {
        banks_client,
        payer,
        recent_blockhash,
        program_id: create_pda_using_anchor::ID,
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
        .filter(|line| line.contains(&format!("Program {} failed", create_pda_using_anchor::ID)))
        .peekable()
        .peek()
        .is_none()
}
